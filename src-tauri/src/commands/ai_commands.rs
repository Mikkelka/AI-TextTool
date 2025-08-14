use tauri_plugin_clipboard_manager::ClipboardExt;

use super::super::ai_provider::{GeminiProvider, ChatMessage, GenerationConfig, ThinkingConfig, ChatResponse};
use super::super::data_manager::DataManager;

#[tauri::command]
pub async fn process_text_with_ai(
    text: String,
    operation: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    println!("Processing text with AI: '{}' using operation: '{}'", text, operation);
    
    // Load configuration to get API key and model settings
    let mut manager = DataManager::new(app.clone());
    let _ = manager.initialize().await.map_err(|e| format!("Failed to initialize data: {}", e))?;
    let config = manager.get_config().clone();
    
    // Check if API key is configured
    if config.api_key.trim().is_empty() {
        return Err("API key not configured. Please configure your Gemini API key in settings.".to_string());
    }
    
    // Create Gemini provider
    let provider = match GeminiProvider::new(config.api_key) {
        Ok(provider) => provider,
        Err(e) => return Err(format!("Failed to create AI provider: {}", e)),
    };
    
    // Get operation details
    let operation_details = manager.get_operation(&operation)
        .cloned()
        .ok_or_else(|| format!("Operation '{}' not found", operation))?;
    
    // Prepare the prompt
    let full_prompt = if operation_details.prefix.is_empty() {
        text.clone()
    } else {
        format!("{}{}", operation_details.prefix, text)
    };
    
    // Process with AI
    let result = match provider.process_text_operation(
        &full_prompt,
        &operation.to_lowercase(),
        Some(&operation_details.instruction),
        &config.text_model,
    ).await {
        Ok(result) => result,
        Err(e) => return Err(format!("AI processing failed: {}", e)),
    };
    
    // Copy result to clipboard for auto-paste
    if let Err(e) = app.clipboard().write_text(result.clone()) {
        return Err(format!("Failed to write to clipboard: {:?}", e));
    }
    
    println!("AI processing completed successfully, result copied to clipboard");
    Ok(result)
}

#[tauri::command]
pub async fn chat_with_ai(
    message: String,
    history: Vec<ChatMessage>,
    custom_instruction: Option<String>,
    enable_thinking: Option<bool>,
    app: tauri::AppHandle,
) -> Result<ChatResponse, String> {
    println!("Chat with AI: '{}'", message);
    
    // Load configuration
    let mut manager = DataManager::new(app);
    let _ = manager.initialize().await.map_err(|e| format!("Failed to initialize data: {}", e))?;
    let config = manager.get_config().clone();
    
    if config.api_key.trim().is_empty() {
        return Err("API key not configured".to_string());
    }
    
    // Create provider
    let provider = match GeminiProvider::new(config.api_key) {
        Ok(provider) => provider,
        Err(e) => return Err(format!("Failed to create AI provider: {}", e)),
    };
    
    // Prepare messages
    let mut messages = history;
    messages.push(ChatMessage::user(message));
    
    // Use custom instruction if provided, otherwise use config default
    let system_instruction = custom_instruction
        .as_ref()
        .unwrap_or(&config.chat_system_instruction);
    
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
    match provider.chat_completion_with_thoughts(
        messages,
        Some(system_instruction),
        &config.chat_model,
        generation_config,
    ).await {
        Ok(response) => {
            println!("Chat response generated successfully");
            Ok(response)
        },
        Err(e) => Err(format!("Chat failed: {}", e)),
    }
}

#[tauri::command]
pub async fn test_ai_connection(app: tauri::AppHandle) -> Result<bool, String> {
    println!("Testing AI connection...");
    
    let mut manager = DataManager::new(app);
    let _ = manager.initialize().await.map_err(|e| format!("Failed to initialize data: {}", e))?;
    let config = manager.get_config().clone();
    
    if config.api_key.trim().is_empty() {
        return Ok(false);
    }
    
    let provider = match GeminiProvider::new(config.api_key) {
        Ok(provider) => provider,
        Err(_) => return Ok(false),
    };
    
    match provider.test_connection().await {
        Ok(connected) => {
            println!("Connection test result: {}", connected);
            Ok(connected)
        },
        Err(e) => {
            println!("Connection test failed: {}", e);
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