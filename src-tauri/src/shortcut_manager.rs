use enigo::{Enigo, Key, Keyboard, Mouse, Settings};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Runtime};
use tauri_plugin_clipboard_manager::ClipboardExt;

use super::window_manager;

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
        let mut last_time = last_trigger.lock().unwrap();
        if now.duration_since(*last_time).as_millis() < 200 {
            println!("Debouncing - ignoring duplicate trigger");
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
    std::thread::sleep(std::time::Duration::from_millis(50));

    // First, get current clipboard content to compare later
    let original_clipboard = app_handle
        .clipboard()
        .read_text()
        .unwrap_or_else(|_| String::new());
    println!(
        "Original clipboard content: '{}' (length: {})",
        original_clipboard.chars().take(50).collect::<String>(),
        original_clipboard.len()
    );

    // Clear clipboard with a unique marker to ensure we can detect any change
    let unique_marker = format!(
        "AI_TOOL_MARKER_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    if let Err(e) = app_handle.clipboard().write_text(&unique_marker) {
        println!(
            "Failed to clear clipboard, continuing with original method: {}",
            e
        );
    } else {
        println!("Clipboard cleared with marker: {}", unique_marker);
        // Small delay to ensure clipboard write completes
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    // Simulate Ctrl+C to copy any selected text
    if let Err(e) = simulate_copy() {
        eprintln!("Failed to simulate Ctrl+C: {}", e);
        // Fallback to opening chat window
        if let Err(e) = window_manager::create_fallback_chat_window(&app_handle) {
            eprintln!("Failed to create fallback chat window: {:?}", e);
        }
        return;
    }

    // Longer delay to ensure copy operation completes reliably
    std::thread::sleep(std::time::Duration::from_millis(250));

    // Get clipboard content after Ctrl+C - with retry logic
    let new_clipboard = match app_handle.clipboard().read_text() {
        Ok(content) => content,
        Err(_) => {
            // First retry after additional delay
            std::thread::sleep(std::time::Duration::from_millis(100));
            match app_handle.clipboard().read_text() {
                Ok(content) => content,
                Err(_) => {
                    println!("Failed to read clipboard after retries - opening chat window");
                    if let Err(e) = window_manager::create_fallback_chat_window(&app_handle) {
                        eprintln!("Failed to create fallback chat window: {:?}", e);
                    }
                    return;
                }
            }
        }
    };

    println!(
        "New clipboard content: '{}' (length: {})",
        new_clipboard.chars().take(50).collect::<String>(),
        new_clipboard.len()
    );

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

    println!("Detection results:");
    println!(
        "  clipboard_changed={}, has_meaningful_content={}, is_not_marker={}",
        clipboard_changed, has_meaningful_content, is_not_marker
    );
    println!(
        "  traditional_detection={}, substantial_content={}, duplicate_but_meaningful={}",
        text_was_selected_traditional, has_substantial_content, is_duplicate_but_meaningful
    );
    println!("  final_result: text_was_selected={}", text_was_selected);

    if text_was_selected {
        println!("Text detected - showing popup with operations");
        // Text was selected - show popup with operations
        show_popup_with_text(&app_handle, new_clipboard).await;
    } else {
        println!("No text detected - opening chat window directly");
        // No text selected - open chat window directly
        if let Err(e) = window_manager::create_direct_chat_window(&app_handle) {
            eprintln!("Failed to create direct chat window: {:?}", e);
        }
    }
}

/// Show popup window with text operations
async fn show_popup_with_text<R: Runtime>(app_handle: &AppHandle<R>, clipboard_text: String) {
    // Get mouse position
    let (mouse_x, mouse_y) = if let Ok(enigo_mouse) = Enigo::new(&Settings::default()) {
        enigo_mouse.location().unwrap_or((100, 100))
    } else {
        (100, 100)
    };

    if let Err(e) =
        window_manager::create_popup_window(app_handle, mouse_x, mouse_y, clipboard_text)
    {
        eprintln!("Failed to create popup window: {:?}", e);
    }
}

/// Simulate Ctrl+C to copy selected text
fn simulate_copy() -> Result<(), String> {
    println!("Starting Ctrl+C simulation...");

    match Enigo::new(&Settings::default()) {
        Ok(mut enigo) => {
            // Press and hold Ctrl
            if let Err(e) = enigo.key(Key::Control, enigo::Direction::Press) {
                return Err(format!("Failed to press Ctrl: {:?}", e));
            }

            // Small delay to ensure key is registered
            std::thread::sleep(std::time::Duration::from_millis(10));

            // Click C key
            if let Err(e) = enigo.key(Key::Unicode('c'), enigo::Direction::Click) {
                // Make sure to release Ctrl even if C fails
                let _ = enigo.key(Key::Control, enigo::Direction::Release);
                return Err(format!("Failed to click C: {:?}", e));
            }

            // Small delay before releasing
            std::thread::sleep(std::time::Duration::from_millis(10));

            // Release Ctrl
            if let Err(e) = enigo.key(Key::Control, enigo::Direction::Release) {
                return Err(format!("Failed to release Ctrl: {:?}", e));
            }

            println!("Ctrl+C simulation completed successfully");
            Ok(())
        }
        Err(e) => Err(format!("Failed to create enigo instance: {:?}", e)),
    }
}

/// Simulate Ctrl+V to paste text (used by the simulate_paste command)
pub fn simulate_paste() -> Result<String, String> {
    println!("Simulating Ctrl+V after popup closed...");

    // Small delay to ensure focus has returned to original application
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Use enigo to simulate Ctrl+V
    match Enigo::new(&Settings::default()) {
        Ok(mut enigo) => {
            if let Err(e) = enigo.key(Key::Control, enigo::Direction::Press) {
                return Err(format!("Failed to press Ctrl: {:?}", e));
            }
            if let Err(e) = enigo.key(Key::Unicode('v'), enigo::Direction::Click) {
                return Err(format!("Failed to click V: {:?}", e));
            }
            if let Err(e) = enigo.key(Key::Control, enigo::Direction::Release) {
                return Err(format!("Failed to release Ctrl: {:?}", e));
            }
            println!("Ctrl+V simulation completed");
            Ok("Paste completed".to_string())
        }
        Err(e) => Err(format!("Failed to create enigo instance: {:?}", e)),
    }
}
