use tauri::{AppHandle, Runtime};
use tauri_plugin_clipboard_manager::ClipboardExt;
use enigo::{Enigo, Key, Keyboard, Settings, Mouse};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::window_manager;

/// Create and configure the global shortcut handler with debouncing
pub fn create_shortcut_handler<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    let last_trigger = Arc::new(Mutex::new(Instant::now() - std::time::Duration::from_millis(1000)));
    
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(move |app, _shortcut, _event| {
            handle_global_shortcut(app.clone(), last_trigger.clone())
        })
        .build()
}

/// Handle the global shortcut trigger with debouncing
fn handle_global_shortcut<R: Runtime>(
    app: AppHandle<R>, 
    last_trigger: Arc<Mutex<Instant>>
) {
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
    // First, get current clipboard content to compare later
    let original_clipboard = app_handle.clipboard().read_text().unwrap_or_else(|_| String::new());
    
    // Simulate Ctrl+C to copy any selected text
    if let Err(e) = simulate_copy() {
        eprintln!("Failed to simulate Ctrl+C: {}", e);
        // Fallback to opening chat window
        if let Err(e) = window_manager::create_fallback_chat_window(&app_handle) {
            eprintln!("Failed to create fallback chat window: {:?}", e);
        }
        return;
    }
    
    // Small delay to let the copy operation complete
    std::thread::sleep(std::time::Duration::from_millis(100));
    
    // Get clipboard content after Ctrl+C
    if let Ok(new_clipboard) = app_handle.clipboard().read_text() {
        // Check if clipboard changed (meaning text was selected and copied)
        let text_was_selected = new_clipboard != original_clipboard && !new_clipboard.trim().is_empty();
        
        if text_was_selected {
            // Text was selected - show popup with operations
            show_popup_with_text(&app_handle, new_clipboard).await;
        } else {
            // No text selected - open chat window directly
            if let Err(e) = window_manager::create_direct_chat_window(&app_handle) {
                eprintln!("Failed to create direct chat window: {:?}", e);
            }
        }
    } else {
        // Failed to read clipboard - fallback to chat window
        if let Err(e) = window_manager::create_fallback_chat_window(&app_handle) {
            eprintln!("Failed to create fallback chat window: {:?}", e);
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
    
    if let Err(e) = window_manager::create_popup_window(app_handle, mouse_x, mouse_y, clipboard_text) {
        eprintln!("Failed to create popup window: {:?}", e);
    }
}

/// Simulate Ctrl+C to copy selected text
fn simulate_copy() -> Result<(), String> {
    match Enigo::new(&Settings::default()) {
        Ok(mut enigo) => {
            if let Err(e) = enigo.key(Key::Control, enigo::Direction::Press) {
                return Err(format!("Failed to press Ctrl: {:?}", e));
            }
            if let Err(e) = enigo.key(Key::Unicode('c'), enigo::Direction::Click) {
                return Err(format!("Failed to click C: {:?}", e));
            }
            if let Err(e) = enigo.key(Key::Control, enigo::Direction::Release) {
                return Err(format!("Failed to release Ctrl: {:?}", e));
            }
            println!("Ctrl+C simulation completed");
            Ok(())
        },
        Err(e) => {
            Err(format!("Failed to create enigo instance: {:?}", e))
        }
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
        },
        Err(e) => {
            Err(format!("Failed to create enigo instance: {:?}", e))
        }
    }
}