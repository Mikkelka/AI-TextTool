use tauri::AppHandle;

use super::super::utils::time;
use super::super::window_manager::{chat_window_config_standard, create_window};

/// Maximum URL length before switching to initialization_script injection
const MAX_URL_TEXT_LENGTH: usize = 1000;

#[tauri::command]
pub async fn reopen_chat_conversation(
    app: AppHandle,
    conversation_id: String,
    operation: String,
    title: String,
) -> Result<(), String> {
    log::info!("Reopening conversation: {}", title);

    let timestamp = time::get_current_timestamp_millis();
    let chat_url = format!(
        "windows/chat.html?operation={}&title={}&conversationId={}&t={}",
        urlencoding::encode(&operation),
        urlencoding::encode(&title),
        urlencoding::encode(&conversation_id),
        timestamp
    );

    let mut config = chat_window_config_standard("reopen", Some(chat_url), Some(title));
    config.window_id = format!("chat_reopen_{}", timestamp);

    match create_window(&app, config) {
        Ok(()) => {
            log::info!("Chat conversation reopened successfully");
            Ok(())
        }
        Err(e) => {
            log::error!("Failed to reopen chat conversation: {e:?}");
            Err(format!("Failed to reopen conversation: {}", e))
        }
    }
}

#[tauri::command]
pub async fn open_chat_window(
    app: AppHandle,
    operation: String,
    text: String,
    instruction: String,
) -> Result<(), String> {
    log::info!(
        "Opening chat window for operation: {} with text length: {}",
        operation,
        text.len()
    );

    let timestamp = time::get_current_timestamp_millis();
    let window_title = format!("{} - AI TextTool", operation);

    // For large text, use initialization_script injection instead of URL params
    // to avoid URL length limits (~2000-8000 chars)
    let (chat_url, init_script) = if text.len() > MAX_URL_TEXT_LENGTH {
        log::debug!(
            "Text too large for URL ({} chars), using initialization_script",
            text.len()
        );
        let url = format!(
            "windows/chat.html?operation={}&title={}&t={}",
            urlencoding::encode(&operation),
            urlencoding::encode(&window_title),
            timestamp
        );
        let script = format!(
            "window.__chatInitData = {{ text: {}, instruction: {} }};",
            serde_json::to_string(&text).unwrap_or_else(|_| "\"\"".to_string()),
            serde_json::to_string(&instruction).unwrap_or_else(|_| "\"\"".to_string())
        );
        (url, Some(script))
    } else {
        let url = format!(
            "windows/chat.html?operation={}&text={}&title={}&instruction={}&t={}",
            urlencoding::encode(&operation),
            urlencoding::encode(&text),
            urlencoding::encode(&window_title),
            urlencoding::encode(&instruction),
            timestamp
        );
        (url, None)
    };

    log::debug!("Creating chat window with URL: {}", chat_url);

    let mut config = chat_window_config_standard(
        &operation.to_lowercase(),
        Some(chat_url),
        Some(window_title.clone()),
    );
    if let Some(script) = init_script {
        config.initialization_script = Some(script);
    }

    match create_window(&app, config) {
        Ok(()) => {
            log::info!(
                "Chat window opened successfully for operation: {}",
                operation
            );
            Ok(())
        }
        Err(e) => {
            log::error!("Failed to open chat window: {e:?}");
            Err(format!("Failed to open chat window: {}", e))
        }
    }
}
