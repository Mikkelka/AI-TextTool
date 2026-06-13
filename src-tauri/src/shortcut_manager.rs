use enigo::{Enigo, Key, Keyboard, Mouse, Settings};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Runtime};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tokio::time::sleep;

use super::{utils::time, window_manager};

/// Time window to ignore duplicate shortcut triggers (milliseconds)
const DEBOUNCE_MS: u128 = 200;

/// Delay before reading clipboard to let other operations settle (milliseconds)
const INITIAL_DELAY_MS: u64 = 50;

/// Delay after writing clipboard marker to ensure it propagates (milliseconds)
const CLIPBOARD_WRITE_DELAY_MS: u64 = 50;

/// Delay after simulating copy to ensure the OS has captured the selection (milliseconds)
const COPY_COMPLETION_DELAY_MS: u64 = 250;

/// Additional delay before retrying a failed clipboard read (milliseconds)
const CLIPBOARD_RETRY_DELAY_MS: u64 = 100;

/// Shared enigo instance for keyboard/mouse operations
fn get_enigo() -> Result<Enigo, String> {
    Enigo::new(&Settings::default())
        .map_err(|e| format!("Failed to create enigo instance: {:?}", e))
}

/// Create and configure the global shortcut handler with debouncing
pub fn create_shortcut_handler<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    let last_trigger = Arc::new(Mutex::new(
        Instant::now() - std::time::Duration::from_millis(1000),
    ));

    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(move |app, _shortcut, _event| {
            handle_global_shortcut(app.clone(), last_trigger.clone())
        })
        .build()
}

/// Handle the global shortcut trigger with debouncing
fn handle_global_shortcut<R: Runtime>(app: AppHandle<R>, last_trigger: Arc<Mutex<Instant>>) {
    // Debouncing - only handle if 200ms have passed since last trigger
    let now = Instant::now();
    {
        let mut last_time = match last_trigger.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                log::warn!("Shortcut debounce mutex was poisoned; recovering state");
                poisoned.into_inner()
            }
        };
        if now.duration_since(*last_time).as_millis() < DEBOUNCE_MS {
            log::debug!("Debouncing - ignoring duplicate trigger");
            return;
        }
        *last_time = now;
    }

    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        process_shortcut_trigger(app_handle).await;
    });
}

/// Process the shortcut trigger: copy text, analyze clipboard, and show appropriate window
async fn process_shortcut_trigger<R: Runtime>(app_handle: AppHandle<R>) {
    // Small initial delay to let any ongoing operations (like Ctrl+A) complete
    sleep(std::time::Duration::from_millis(CLIPBOARD_WRITE_DELAY_MS)).await;

    // First, get current clipboard content to compare later
    let original_clipboard = app_handle
        .clipboard()
        .read_text()
        .unwrap_or_else(|_| String::new());
    log::debug!(
        "Original clipboard content length: {}",
        original_clipboard.len()
    );

    // Clear clipboard with a unique marker to ensure we can detect any change
    let unique_marker = format!("AI_TOOL_MARKER_{}", time::get_current_timestamp_millis());

    if let Err(e) = app_handle.clipboard().write_text(&unique_marker) {
        log::warn!(
            "Failed to clear clipboard, continuing with original method: {}",
            e
        );
    } else {
        log::debug!("Clipboard cleared with marker: {}", unique_marker);
        // Small delay to ensure clipboard write completes
        sleep(std::time::Duration::from_millis(INITIAL_DELAY_MS)).await;
    }

    // Simulate Ctrl+C to copy any selected text
    if let Err(e) = simulate_copy() {
        log::error!("Failed to simulate Ctrl+C: {e}");
        restore_clipboard(&app_handle, &original_clipboard).await;
        // Fallback to opening chat window
        if let Err(e) = window_manager::create_fallback_chat_window(&app_handle) {
            log::error!("Failed to create fallback chat window: {e:?}");
        }
        return;
    }

    // Longer delay to ensure copy operation completes reliably
    sleep(std::time::Duration::from_millis(COPY_COMPLETION_DELAY_MS)).await;

    // Get clipboard content after Ctrl+C - with retry logic
    let new_clipboard = match app_handle.clipboard().read_text() {
        Ok(content) => content,
        Err(_) => {
            // First retry after additional delay
            sleep(std::time::Duration::from_millis(CLIPBOARD_RETRY_DELAY_MS)).await;
            match app_handle.clipboard().read_text() {
                Ok(content) => content,
                Err(_) => {
                    log::warn!("Failed to read clipboard after retries - opening chat window");
                    restore_clipboard(&app_handle, &original_clipboard).await;
                    if let Err(e) = window_manager::create_fallback_chat_window(&app_handle) {
                        log::error!("Failed to create fallback chat window: {e:?}");
                    }
                    return;
                }
            }
        }
    };

    log::debug!("New clipboard content length: {}", new_clipboard.len());

    // Enhanced detection logic with multiple strategies
    let clipboard_changed = new_clipboard != original_clipboard;
    let has_meaningful_content = !new_clipboard.trim().is_empty();
    let is_not_marker = !new_clipboard.starts_with("AI_TOOL_MARKER_");

    // Strategy 1: Traditional change detection
    let text_was_selected_traditional =
        clipboard_changed && has_meaningful_content && is_not_marker;

    // Strategy 2: If clipboard contains substantial text (likely from Ctrl+A scenarios)
    let substantial_text_threshold = 10; // Characters
    let has_substantial_content =
        new_clipboard.trim().len() >= substantial_text_threshold && is_not_marker;

    // Strategy 3: Check if clipboard is exactly same as original (Ctrl+A duplicate scenario)
    let is_duplicate_but_meaningful = new_clipboard == original_clipboard
        && has_meaningful_content
        && new_clipboard.trim().len() >= 5;

    let text_was_selected =
        text_was_selected_traditional || has_substantial_content || is_duplicate_but_meaningful;

    log::debug!("Detection results:");
    log::debug!(
        "  clipboard_changed={}, has_meaningful_content={}, is_not_marker={}",
        clipboard_changed,
        has_meaningful_content,
        is_not_marker
    );
    log::debug!(
        "  traditional_detection={}, substantial_content={}, duplicate_but_meaningful={}",
        text_was_selected_traditional,
        has_substantial_content,
        is_duplicate_but_meaningful
    );
    log::debug!("  final_result: text_was_selected={}", text_was_selected);

    if text_was_selected {
        log::info!("Text detected - showing popup with operations");
        // Text was selected - show popup with operations
        show_popup_with_text(&app_handle, new_clipboard).await;
    } else {
        log::info!("No text detected - opening chat window directly");
        restore_clipboard(&app_handle, &original_clipboard).await;
        // No text selected - open chat window directly
        if let Err(e) = window_manager::create_direct_chat_window(&app_handle) {
            log::error!("Failed to create direct chat window: {e:?}");
        }
    }
}

/// Restore original clipboard content if we replaced it with a marker.
async fn restore_clipboard<R: Runtime>(app_handle: &AppHandle<R>, original_clipboard: &str) {
    if let Err(e) = app_handle.clipboard().write_text(original_clipboard) {
        log::warn!("Failed to restore original clipboard: {e}");
    }
}

/// Show popup window with text operations
async fn show_popup_with_text<R: Runtime>(app_handle: &AppHandle<R>, clipboard_text: String) {
    let (mouse_x, mouse_y) = match get_enigo() {
        Ok(enigo) => enigo.location().unwrap_or((100, 100)),
        Err(_) => (100, 100),
    };

    if let Err(e) =
        window_manager::create_popup_window(app_handle, mouse_x, mouse_y, clipboard_text)
    {
        log::error!("Failed to create popup window: {e:?}");
    }
}

/// Simulate Ctrl+C to copy selected text
/// Note: Uses thread::sleep instead of tokio::sleep for hardware timing
/// These are very short delays (10ms) for keyboard input registration
fn simulate_copy() -> Result<(), String> {
    log::debug!("Starting Ctrl+C simulation...");

    let mut enigo = get_enigo()?;

    // Press and hold Ctrl
    if let Err(e) = enigo.key(Key::Control, enigo::Direction::Press) {
        return Err(format!("Failed to press Ctrl: {:?}", e));
    }

    // Small delay to ensure key is registered (hardware timing)
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Click C key
    if let Err(e) = enigo.key(Key::Unicode('c'), enigo::Direction::Click) {
        let _ = enigo.key(Key::Control, enigo::Direction::Release);
        return Err(format!("Failed to click C: {:?}", e));
    }

    // Small delay before releasing
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Release Ctrl
    if let Err(e) = enigo.key(Key::Control, enigo::Direction::Release) {
        return Err(format!("Failed to release Ctrl: {:?}", e));
    }

    log::debug!("Ctrl+C simulation completed successfully");
    Ok(())
}

/// Simulate Ctrl+V to paste text (used by the simulate_paste command)
/// Note: Uses thread::sleep instead of tokio::sleep for hardware timing
/// The 100ms delay ensures focus has returned to the original application
pub fn simulate_paste() -> Result<String, String> {
    log::debug!("Simulating Ctrl+V after popup closed...");

    std::thread::sleep(std::time::Duration::from_millis(100));

    let mut enigo = get_enigo()?;

    if let Err(e) = enigo.key(Key::Control, enigo::Direction::Press) {
        return Err(format!("Failed to press Ctrl: {:?}", e));
    }
    if let Err(e) = enigo.key(Key::Unicode('v'), enigo::Direction::Click) {
        return Err(format!("Failed to click V: {:?}", e));
    }
    if let Err(e) = enigo.key(Key::Control, enigo::Direction::Release) {
        return Err(format!("Failed to release Ctrl: {:?}", e));
    }
    log::debug!("Ctrl+V simulation completed");
    Ok("Paste completed".to_string())
}
