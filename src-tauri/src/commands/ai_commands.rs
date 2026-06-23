use tauri::Manager;
use tauri_plugin_clipboard_manager::ClipboardExt;

use super::super::ai_provider::gemini::RateLimiter;
use super::super::ai_provider::types::{CHAT_MODEL, TEXT_MODEL};
use super::super::ai_provider::{
    ChatMessage, ChatResponse, Content, GeminiError, GeminiProvider, GenerationConfig,
    GlobalRateLimiter, SharedHttpClient, ThinkingConfig,
};
use super::super::data_manager::types::ProviderConfig;
use super::super::data_manager::SharedDataManager;
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
        GeminiError::RateLimitExceeded {
            retry_after_seconds,
        } => {
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

/// Helper function to get the shared rate limiter from Tauri state
fn get_rate_limiter(app: &tauri::AppHandle) -> std::sync::Arc<tokio::sync::Mutex<RateLimiter>> {
    let global_limiter = app.state::<GlobalRateLimiter>();
    global_limiter.get_limiter()
}

/// Helper function to get the shared HTTP client from Tauri state
fn get_http_client(app: &tauri::AppHandle) -> SharedHttpClient {
    app.state::<SharedHttpClient>().inner().clone()
}

/// Error returned when the active provider or its API key is missing.
const NO_PROVIDER_MSG: &str =
    "No AI provider configured. Please complete setup in onboarding or settings.";
const NO_API_KEY_MSG: &str =
    "API key not configured. Please configure your Gemini API key in settings.";

/// Load the active provider config and construct a `GeminiProvider`.
/// Returns the provider config (borrowed clone) alongside the provider so callers
/// can access `chat_model_name` / `text_model_name` / `chat_system_instruction`.
///
/// Returns `Err` with a user-friendly message when no provider is configured
/// or the API key is empty. Returns `Ok(None)` only from `test_ai_connection`,
/// which prefers `Ok(false)` over an error — use [`try_create_provider`] there.
async fn create_provider(
    app: &tauri::AppHandle,
) -> Result<(GeminiProvider, ProviderConfig), String> {
    let state = app.state::<SharedDataManager>();
    let manager = state.0.lock().await;
    let config = manager.get_config().clone();

    let provider_config = config
        .active_provider()
        .ok_or_else(|| NO_PROVIDER_MSG.to_string())?
        .clone();

    if provider_config.api_key.trim().is_empty() {
        return Err(NO_API_KEY_MSG.to_string());
    }

    let rate_limiter = get_rate_limiter(app);
    let http_client = get_http_client(app);
    let provider = GeminiProvider::new(
        provider_config.api_key.to_string(),
        rate_limiter,
        &http_client,
    )
    .map_err(gemini_error_to_user_message)?;

    Ok((provider, provider_config))
}

/// Like [`create_provider`] but returns `Ok(None)` instead of `Err` when no
/// provider is configured or the API key is empty. Used by `test_ai_connection`,
/// which treats those states as "not connected" rather than errors.
async fn try_create_provider(app: &tauri::AppHandle) -> Result<Option<GeminiProvider>, String> {
    let state = app.state::<SharedDataManager>();
    let manager = state.0.lock().await;
    let config = manager.get_config().clone();

    let provider_config = match config.active_provider() {
        Some(p) => p,
        None => return Ok(None),
    };

    if provider_config.api_key.trim().is_empty() {
        return Ok(None);
    }

    let rate_limiter = get_rate_limiter(app);
    let http_client = get_http_client(app);
    match GeminiProvider::new(
        provider_config.api_key.to_string(),
        rate_limiter,
        &http_client,
    ) {
        Ok(provider) => Ok(Some(provider)),
        Err(_) => Ok(None),
    }
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

    // Construct provider (loads config, checks API key, builds GeminiProvider)
    let (provider, provider_config) = create_provider(&app).await?;

    // Get operation details (needs the data manager)
    let state = app.state::<SharedDataManager>();
    let manager = state.0.lock().await;
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
            &provider_config.text_model_name,
            contents,
            Some(&operation_details.instruction),
            Some(GenerationConfig {
                temperature: Some(0.0),
                ..Default::default()
            }),
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

    // Construct provider (loads config, checks API key, builds GeminiProvider)
    let (provider, provider_config) = create_provider(&app).await?;

    // Prepare messages
    let mut messages = history;
    messages.push(ChatMessage::user(message));

    // Use custom instruction if provided, otherwise use config default
    let default_instruction = provider_config.chat_system_instruction.clone();
    let system_instruction = custom_instruction.as_ref().unwrap_or(&default_instruction);
    let selected_model = selected_model
        .map(|model| model.trim().to_string())
        .filter(|model| !model.is_empty())
        .unwrap_or_else(|| provider_config.chat_model_name.clone());
    let enable_grounding = enable_grounding.unwrap_or(false);

    if enable_grounding && !GeminiProvider::supports_google_search_grounding(&selected_model) {
        return Err(format!(
            "Google Search grounding is not supported for model '{}'. Select {} or {}.",
            selected_model, CHAT_MODEL, TEXT_MODEL
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

    // No provider / empty API key both map to "not connected"
    let provider = match try_create_provider(&app).await? {
        Some(p) => p,
        None => return Ok(false),
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
