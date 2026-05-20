use tauri::Manager;
use tauri_plugin_clipboard_manager::ClipboardExt;

use super::super::ai_provider::{
    ChatMessage, ChatResponse, Content, GeminiError, GeminiModel, GeminiProvider, GenerationConfig,
    GlobalRateLimiter, ThinkingConfig,
};
use super::super::ai_provider::gemini::RateLimiter;
use super::super::data_manager::DataManager;
use super::super::utils::validation;

/// Convert GeminiError to user-friendly error message
fn gemini_error_to_user_message(error: GeminiError) -> String {
    match error {
        GeminiError::InvalidApiKey => {
            "Invalid API key. Please check your Gemini API key in settings.".to_string()
        }
        GeminiError::Timeout => {
            "Connection timed out. Please check your internet connection and try again.".to_string()
        }
        GeminiError::ServiceUnavailable => {
            "Gemini service is currently unavailable. Please try again later.".to_string()
        }
        GeminiError::RateLimitExceeded { retry_after_seconds } => {
            format!(
                "Rate limit exceeded. Please try again in {} seconds.",
                retry_after_seconds
            )
        }
        GeminiError::ModelNotFound { model } => {
            format!(
                "Model '{}' not found. Please select a different model in settings.",
                model
            )
        }
        GeminiError::ApiError { status, message } => {
            format!("API error ({}): {}", status, message)
        }
        GeminiError::HttpError(e) => {
            format!("Network error: {}. Please check your connection.", e)
        }
        GeminiError::JsonError(e) => {
            format!("Invalid response format: {}. Please try again.", e)
        }
        GeminiError::InvalidRequest { message } => {
            format!("Invalid request: {}", message)
        }
    }
}

/// Helper function to load and initialize DataManager
/// Reduces code duplication across all AI commands
async fn load_data_manager(app: tauri::AppHandle) -> Result<DataManager, String> {
    let mut manager = DataManager::new(app);
    manager
        .initialize()
        .await
        .map_err(|e| format!("Failed to initialize data: {}", e))?;
    Ok(manager)
}

/// Helper function to get the shared rate limiter from Tauri state
fn get_rate_limiter(app: &tauri::AppHandle) -> std::sync::Arc<tokio::sync::Mutex<RateLimiter>> {
    let global_limiter = app.state::<GlobalRateLimiter>();
    global_limiter.get_limiter()
}

#[tauri::command]
pub async fn process_text_with_ai(
    text: String,
    operation: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    log::info!(
        "Processing text with AI (length: {} chars) using operation: '{}'",
        text.len(),
        operation
    );

    // Validate input
    validation::validate_text_input(&text)?;
    validation::validate_operation_name(&operation)?;

    // Load configuration to get API key and model settings
    let manager = load_data_manager(app.clone()).await?;
    let config = manager.get_config().clone();

    // Check if API key is configured
    if config.api_key.trim().is_empty() {
        return Err(
            "API key not configured. Please configure your Gemini API key in settings.".to_string(),
        );
    }

    // Get shared rate limiter and create Gemini provider
    let rate_limiter = get_rate_limiter(&app);
    let provider = GeminiProvider::new(config.api_key, rate_limiter)
        .map_err(gemini_error_to_user_message)?;

    // Get operation details
    let operation_details = manager
        .get_operation(&operation)
        .cloned()
        .ok_or_else(|| format!("Operation '{}' not found", operation))?;

    // Prepare the prompt
    let full_prompt = if operation_details.prefix.is_empty() {
        text.clone()
    } else {
        format!("{}{}", operation_details.prefix, text)
    };

    // Process with AI
    let contents = vec![Content::user(full_prompt)];
    let result = match provider
        .generate_content_with_formatting(
            config.text_model.as_str(),
            contents,
            Some(&operation_details.instruction),
            None,
            false, // Disable formatting for direct text operations
        )
        .await
    {
        Ok(result) => result,
        Err(e) => return Err(format!("AI processing failed: {}", e)),
    };

    // Copy result to clipboard for auto-paste
    if let Err(e) = app.clipboard().write_text(result.clone()) {
        return Err(format!("Failed to write to clipboard: {:?}", e));
    }

    log::info!("AI processing completed successfully, result copied to clipboard");
    Ok(result)
}

#[tauri::command]
pub async fn chat_with_ai(
    message: String,
    history: Vec<ChatMessage>,
    custom_instruction: Option<String>,
    selected_model: Option<String>,
    enable_thinking: Option<bool>,
    enable_grounding: Option<bool>,
    app: tauri::AppHandle,
) -> Result<ChatResponse, String> {
    log::info!("Chat with AI (length: {} chars)", message.len());

    // Validate input
    validation::validate_message_input(&message)?;

    // Load configuration
    let manager = load_data_manager(app.clone()).await?;
    let config = manager.get_config().clone();

    if config.api_key.trim().is_empty() {
        return Err("API key not configured".to_string());
    }

    // Get shared rate limiter and create provider
    let rate_limiter = get_rate_limiter(&app);
    let provider = GeminiProvider::new(config.api_key, rate_limiter)
        .map_err(gemini_error_to_user_message)?;

    // Prepare messages
    let mut messages = history;
    messages.push(ChatMessage::user(message));

    // Use custom instruction if provided, otherwise use config default
    let system_instruction = custom_instruction
        .as_ref()
        .unwrap_or(&config.chat_system_instruction);
    let selected_model = selected_model
        .map(|model| model.trim().to_string())
        .filter(|model| !model.is_empty())
        .unwrap_or_else(|| config.chat_model.to_string());
    let enable_grounding = enable_grounding.unwrap_or(false);

    if enable_grounding && !GeminiProvider::supports_google_search_grounding(&selected_model) {
        return Err(format!(
            "Google Search grounding is not supported for model '{}'. Select {} or {}.",
            selected_model,
            GeminiModel::DEFAULT_CHAT,
            GeminiModel::DEFAULT_TEXT
        ));
    }

    // Create generation config with thinking if enabled
    let generation_config = if enable_thinking.unwrap_or(false) {
        Some(GenerationConfig {
            temperature: Some(0.7),
            top_p: Some(0.8),
            top_k: Some(40),
            max_output_tokens: Some(8192),
            candidate_count: Some(1),
            thinking_config: Some(ThinkingConfig::dynamic_with_thoughts()),
        })
    } else {
        None // Use default config (no thinking)
    };

    // Generate response
    match provider
        .chat_completion_with_thoughts(
            messages,
            Some(system_instruction),
            &selected_model,
            generation_config,
            enable_grounding,
        )
        .await
    {
        Ok(response) => {
            log::info!("Chat response generated successfully");
            Ok(response)
        }
        Err(e) => Err(format!("Chat failed: {}", e)),
    }
}

#[tauri::command]
pub async fn test_ai_connection(app: tauri::AppHandle) -> Result<bool, String> {
    log::info!("Testing AI connection...");

    let manager = load_data_manager(app.clone()).await?;
    let config = manager.get_config().clone();

    if config.api_key.trim().is_empty() {
        return Ok(false);
    }

    let rate_limiter = get_rate_limiter(&app);
    let provider = match GeminiProvider::new(config.api_key, rate_limiter) {
        Ok(provider) => provider,
        Err(_) => return Ok(false),
    };

    match provider.test_connection().await {
        Ok(connected) => {
            log::info!("Connection test result: {}", connected);
            Ok(connected)
        }
        Err(e) => {
            log::warn!("Connection test failed: {}", e);
            Ok(false)
        }
    }
}

#[tauri::command]
pub async fn get_ai_models() -> Result<Vec<String>, String> {
    let models = GeminiProvider::get_available_models()
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    Ok(models)
}
