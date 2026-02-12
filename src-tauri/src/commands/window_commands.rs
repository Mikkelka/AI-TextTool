use tauri::{AppHandle, WebviewWindowBuilder};

use super::super::utils::time;

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

    // Create the chat URL with conversation data
    let chat_url = format!(
        "windows/chat.html?operation={}&title={}&conversationId={}&t={}",
        urlencoding::encode(&operation),
        urlencoding::encode(&title),
        urlencoding::encode(&conversation_id),
        timestamp
    );

    // Create chat window using backend WebviewWindowBuilder (same as tray chat)
    match WebviewWindowBuilder::new(&app, &window_id, tauri::WebviewUrl::App(chat_url.into()))
        .title(&title)
        .inner_size(900.0, 700.0)
        .min_inner_size(700.0, 500.0)
        .center()
        .resizable(true)
        .maximizable(true)
        .minimizable(true)
        .closable(true)
        .always_on_top(false)
        .skip_taskbar(false)
        .build()
    {
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

    // Create timestamp for unique window ID
    let timestamp = time::get_current_timestamp_millis();
    let window_id = format!("chat_{}_{}", operation.to_lowercase(), timestamp);

    // Create the chat URL with operation data
    let chat_url = format!(
        "windows/chat.html?operation={}&text={}&title={}&instruction={}&t={}",
        urlencoding::encode(&operation),
        urlencoding::encode(&text),
        urlencoding::encode(&format!("{} - AI TextTool", operation)),
        urlencoding::encode(&instruction),
        timestamp
    );

    log::debug!("Creating chat window with URL: {}", chat_url);

    // Create chat window using backend WebviewWindowBuilder (same as tray chat)
    match WebviewWindowBuilder::new(&app, &window_id, tauri::WebviewUrl::App(chat_url.into()))
        .title(format!("{} - AI TextTool", operation))
        .inner_size(900.0, 700.0)
        .min_inner_size(700.0, 500.0)
        .center()
        .resizable(true)
        .maximizable(true)
        .minimizable(true)
        .closable(true)
        .always_on_top(false)
        .skip_taskbar(false)
        .build()
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
