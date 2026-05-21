use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};

use super::super::utils::time;

/// Maximum URL length before switching to initialization_script injection
const MAX_URL_TEXT_LENGTH: usize = 1000;

/// Shared window builder configuration for chat windows
fn create_chat_window_builder<'a, R: tauri::Runtime>(
    app: &'a AppHandle<R>,
    window_id: &'a str,
    title: &'a str,
    url: String,
    initialization_script: Option<String>,
) -> WebviewWindowBuilder<'a, R, AppHandle<R>> {
    let mut builder = WebviewWindowBuilder::new(app, window_id, WebviewUrl::App(url.into()))
        .title(title)
        .inner_size(900.0, 700.0)
        .min_inner_size(700.0, 500.0)
        .center()
        .resizable(true)
        .maximizable(true)
        .minimizable(true)
        .closable(true)
        .always_on_top(false)
        .skip_taskbar(false);

    if let Some(script) = initialization_script {
        builder = builder.initialization_script(script);
    }

    builder
}

#[tauri::command]
pub async fn reopen_chat_conversation(
    app: AppHandle,
    conversation_id: String,
    operation: String,
    title: String,
) -> Result<(), String> {
    log::info!("Reopening conversation: {}", title);

    let timestamp = time::get_current_timestamp_millis();
    let window_id = format!("chat_reopen_{}", timestamp);

    let chat_url = format!(
        "windows/chat.html?operation={}&title={}&conversationId={}&t={}",
        urlencoding::encode(&operation),
        urlencoding::encode(&title),
        urlencoding::encode(&conversation_id),
        timestamp
    );

    match create_chat_window_builder(&app, &window_id, &title, chat_url, None).build() {
        Ok(chat_window) => {
            log::info!("Chat conversation reopened successfully: {}", title);
            let _ = chat_window.set_focus();
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
    let window_id = format!("chat_{}_{}", operation.to_lowercase(), timestamp);
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

    match create_chat_window_builder(&app, &window_id, &window_title, chat_url, init_script).build()
    {
        Ok(chat_window) => {
            log::info!(
                "Chat window opened successfully for operation: {}",
                operation
            );
            let _ = chat_window.set_focus();
            Ok(())
        }
        Err(e) => {
            log::error!("Failed to open chat window: {e:?}");
            Err(format!("Failed to open chat window: {}", e))
        }
    }
}
