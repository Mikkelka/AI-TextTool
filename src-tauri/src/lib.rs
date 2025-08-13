use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime, Emitter, WebviewWindowBuilder, AppHandle,
};
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tauri_plugin_clipboard_manager::ClipboardExt;
use enigo::{Enigo, Key, Keyboard, Settings, Mouse};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use ai_provider::{GeminiProvider, ChatMessage};
use data_manager::DataManager;

mod ai_provider;
mod data_manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn process_text_with_ai(
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
async fn chat_with_ai(
    message: String,
    history: Vec<ChatMessage>,
    custom_instruction: Option<String>,
    app: tauri::AppHandle,
) -> Result<String, String> {
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
    
    // Generate response
    match provider.chat_completion(
        messages,
        Some(system_instruction),
        &config.chat_model,
        None,
    ).await {
        Ok(response) => {
            println!("Chat response generated successfully");
            Ok(response)
        },
        Err(e) => Err(format!("Chat failed: {}", e)),
    }
}

#[tauri::command]
async fn test_ai_connection(app: tauri::AppHandle) -> Result<bool, String> {
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
async fn get_ai_models() -> Result<Vec<String>, String> {
    let models = GeminiProvider::get_available_models()
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    Ok(models)
}

#[tauri::command]
fn process_text(text: String, option: String, app: tauri::AppHandle) -> Result<String, String> {
    println!("Processing text: '{}' with option: '{}'", text, option);
    
    let processed_text = match option.as_str() {
        "add_word" => format!("{} [PROCESSED]", text),
        "fix_grammar" => format!("{} [GRAMMAR FIXED]", text), // Placeholder
        "translate" => format!("{} [TRANSLATED]", text), // Placeholder
        _ => text, // Return original if unknown option
    };
    
    println!("Processed result: '{}'", processed_text);
    
    // Put processed text in clipboard
    if let Err(e) = app.clipboard().write_text(processed_text.clone()) {
        return Err(format!("Failed to write to clipboard: {:?}", e));
    }
    
    println!("Text put in clipboard, will auto-paste after popup closes");
    
    Ok(processed_text)
}

#[tauri::command]
fn simulate_paste() -> Result<String, String> {
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



fn show_onboarding_window<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    println!("Creating onboarding window");
    
    let onboarding_window = WebviewWindowBuilder::new(
        app,
        "onboarding",
        tauri::WebviewUrl::App("onboarding.html".into())
    )
    .title("AI Text Tools - Setup")
    .inner_size(700.0, 600.0)
    .min_inner_size(600.0, 500.0)
    .center()
    .resizable(true)
    .maximizable(false)
    .minimizable(false)
    .closable(true)
    .always_on_top(false)
    .skip_taskbar(false)
    .build()?;
    
    println!("Onboarding window created successfully");
    
    // Listen for onboarding window close event
    let app_handle = app.clone();
    onboarding_window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { .. } = event {
            println!("Onboarding window closed - setting up tray and shortcuts");
            
            // After onboarding is closed, set up tray and shortcuts
            if let Err(e) = create_tray(&app_handle) {
                eprintln!("Failed to create tray after onboarding: {:?}", e);
            }
            
            // Register global shortcut after onboarding (hardcoded to ctrl+space)
            match app_handle.global_shortcut().register("CmdOrCtrl+Space") {
                Ok(()) => println!("Global shortcut registered after onboarding"),
                Err(e) => println!("Failed to register shortcut after onboarding: {:?}", e),
            }
        }
    });
    
    Ok(())
}

fn create_tray<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    let chat_i = MenuItem::with_id(app, "chat", "Chat", true, None::<&str>)?;
    let separator1 = tauri::menu::PredefinedMenuItem::separator(app)?;
    let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let edit_operations_i = MenuItem::with_id(app, "edit_operations", "Edit Operations", true, None::<&str>)?;
    let chat_history_i = MenuItem::with_id(app, "chat_history", "Chat History", true, None::<&str>)?;
    let separator2 = tauri::menu::PredefinedMenuItem::separator(app)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&chat_i, &separator1, &settings_i, &edit_operations_i, &chat_history_i, &separator2, &quit_i])?;

    // Use the default app icon from the bundle
    let _ = TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            }
            "chat" => {
                println!("Opening chat window from tray...");
                
                // Close existing chat windows
                let existing_chat_windows = vec!["chat", "chat_direct"];
                for window_name in &existing_chat_windows {
                    if let Some(existing_chat) = app.get_webview_window(window_name) {
                        let _ = existing_chat.close();
                    }
                }
                
                // Create timestamp for unique window ID
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                let window_id = format!("chat_tray_{}", timestamp);
                
                // Create chat window centered on screen
                match WebviewWindowBuilder::new(
                    app,
                    &window_id,
                    tauri::WebviewUrl::App(format!("chat.html?operation=Chat&title=AI Chat&t={}", timestamp).into())
                )
                .title("AI Chat")
                .inner_size(900.0, 700.0)
                .min_inner_size(700.0, 500.0)
                .center()
                .resizable(true)
                .maximizable(true)
                .minimizable(true)
                .closable(true)
                .always_on_top(false)
                .skip_taskbar(false)
                .build() {
                    Ok(chat_window) => {
                        println!("Chat window opened successfully from tray");
                        let _ = chat_window.set_focus();
                    },
                    Err(e) => {
                        eprintln!("Failed to create chat window from tray: {:?}", e);
                    }
                }
            }
            "settings" => {
                println!("Opening settings window...");
                
                // Close existing settings window if open
                if let Some(existing_settings) = app.get_webview_window("settings") {
                    let _ = existing_settings.close();
                }
                
                // Create settings window
                match WebviewWindowBuilder::new(
                    app,
                    "settings",
                    tauri::WebviewUrl::App("settings.html".into())
                )
                .title("Settings - AI Text Tools")
                .inner_size(600.0, 700.0)
                .min_inner_size(500.0, 600.0)
                .center()
                .resizable(true)
                .maximizable(false)
                .minimizable(true)
                .closable(true)
                .always_on_top(false)
                .skip_taskbar(false)
                .build() {
                    Ok(settings_window) => {
                        println!("Settings window opened successfully");
                        let _ = settings_window.set_focus();
                    },
                    Err(e) => {
                        eprintln!("Failed to create settings window: {:?}", e);
                    }
                }
            }
            "chat_history" => {
                println!("Opening chat history window...");
                
                // Close existing history window if open
                if let Some(existing_history) = app.get_webview_window("chat_history") {
                    let _ = existing_history.close();
                }
                
                // Create chat history window
                match WebviewWindowBuilder::new(
                    app,
                    "chat_history",
                    tauri::WebviewUrl::App("history.html".into())
                )
                .title("Chat History - AI Text Tools")
                .inner_size(1000.0, 700.0)
                .min_inner_size(800.0, 600.0)
                .center()
                .resizable(true)
                .maximizable(true)
                .minimizable(true)
                .closable(true)
                .always_on_top(false)
                .skip_taskbar(false)
                .build() {
                    Ok(history_window) => {
                        println!("Chat history window opened successfully");
                        let _ = history_window.set_focus();
                    },
                    Err(e) => {
                        eprintln!("Failed to create chat history window: {:?}", e);
                    }
                }
            }
            "edit_operations" => {
                println!("Opening edit operations window...");
                
                // Close existing edit operations window if open
                if let Some(existing_edit) = app.get_webview_window("edit_operations") {
                    let _ = existing_edit.close();
                }
                
                // Create edit operations window
                match WebviewWindowBuilder::new(
                    app,
                    "edit_operations",
                    tauri::WebviewUrl::App("operation-edit.html".into())
                )
                .title("Edit Operations - AI Text Tools")
                .inner_size(900.0, 700.0)
                .min_inner_size(700.0, 500.0)
                .center()
                .resizable(true)
                .maximizable(true)
                .minimizable(true)
                .closable(true)
                .always_on_top(false)
                .skip_taskbar(false)
                .build() {
                    Ok(edit_window) => {
                        println!("Edit operations window opened successfully");
                        let _ = edit_window.set_focus();
                    },
                    Err(e) => {
                        eprintln!("Failed to create edit operations window: {:?}", e);
                    }
                }
            }
            _ => {}
        })
        .on_tray_icon_event(|_tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                ..
            } = event
            {
                // On left click, do nothing - user should use right-click for menu
                // or use the global hotkey to interact with the app
                println!("Tray icon clicked - use right-click for menu");
            }
        })
        .build(app);

    Ok(())
}

#[tauri::command]
async fn reopen_chat_conversation(app: AppHandle, conversation_id: String, operation: String, title: String) -> Result<(), String> {
    println!("Reopening conversation: {}", title);
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    
    let window_id = format!("chat_reopen_{}", timestamp);
    
    // Create the chat URL with conversation data
    let chat_url = format!(
        "chat.html?operation={}&title={}&conversationId={}&t={}",
        urlencoding::encode(&operation),
        urlencoding::encode(&title),
        urlencoding::encode(&conversation_id),
        timestamp
    );
    
    // Create chat window using backend WebviewWindowBuilder (same as tray chat)
    match WebviewWindowBuilder::new(
        &app,
        &window_id,
        tauri::WebviewUrl::App(chat_url.into())
    )
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
    .build() {
        Ok(chat_window) => {
            println!("Chat conversation reopened successfully: {}", title);
            let _ = chat_window.set_focus();
            Ok(())
        },
        Err(e) => {
            eprintln!("Failed to reopen chat conversation: {:?}", e);
            Err(format!("Failed to reopen conversation: {}", e))
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin({
            let last_trigger = Arc::new(Mutex::new(Instant::now() - std::time::Duration::from_millis(1000)));
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, _shortcut, _event| {
                    {
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
                            // First, get current clipboard content to compare later
                            let original_clipboard = app_handle.clipboard().read_text().unwrap_or_else(|_| String::new());
                            
                            // Simulate Ctrl+C to copy any selected text
                            let mut enigo = Enigo::new(&Settings::default()).unwrap();
                            enigo.key(Key::Control, enigo::Direction::Press).unwrap();
                            enigo.key(Key::Unicode('c'), enigo::Direction::Click).unwrap();
                            enigo.key(Key::Control, enigo::Direction::Release).unwrap();
                            
                            // Small delay to let the copy operation complete
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            
                            // Get clipboard content after Ctrl+C
                            if let Ok(new_clipboard) = app_handle.clipboard().read_text() {
                                // Check if clipboard changed (meaning text was selected and copied)
                                let text_was_selected = new_clipboard != original_clipboard && !new_clipboard.trim().is_empty();
                                
                                if text_was_selected {
                                    let clipboard_text = new_clipboard;
                                    // Get mouse position
                                    let enigo_mouse = Enigo::new(&Settings::default()).unwrap();
                                    let (mouse_x, mouse_y) = enigo_mouse.location().unwrap_or((100, 100));
                                    
                                    // Close existing popup windows
                                    if let Some(existing_popup) = app_handle.get_webview_window("popup") {
                                        let _ = existing_popup.close();
                                    }
                                    
                                    // Create popup window with unique label at mouse position
                                    let timestamp = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap()
                                        .as_millis();
                                    let window_label = format!("popup_{}", timestamp);
                                    
                                    println!("Creating popup window '{}' at mouse position: ({}, {})", window_label, mouse_x, mouse_y);
                                    
                                    let popup_window = WebviewWindowBuilder::new(
                                        &app_handle,
                                        &window_label,
                                        tauri::WebviewUrl::App(format!("popup.html?t={}", timestamp).into())
                                    )
                                    .title("AI Text Operations")
                                    .inner_size(420.0, 280.0)  // Very compact 4-column layout
                                    .position(mouse_x as f64, mouse_y as f64)
                                    .resizable(false)
                                    .decorations(true)  // Enable decorations so window can be closed
                                    .closable(true)     // Explicitly enable closing
                                    .always_on_top(true)
                                    .skip_taskbar(true)
                                    .initialization_script(&format!(
                                        "window.clipboardText = '{}';", 
                                        clipboard_text.replace('\'', "\\'").replace('\n', "\\n").replace('\r', "\\r")
                                    ))
                                    .build();
                                    
                                    if let Ok(window) = popup_window {
                                        println!("Popup window created successfully");
                                        // Send clipboard text directly to popup window
                                        let _ = window.emit("set-clipboard-text", clipboard_text);
                                    }
                                } else {
                                    // No text selected - open chat window directly
                                    println!("No text selected - opening chat window directly");
                                    
                                    // Close existing chat windows
                                    let existing_chat_windows = vec!["chat", "chat_direct"];
                                    for window_name in &existing_chat_windows {
                                        if let Some(existing_chat) = app_handle.get_webview_window(window_name) {
                                            let _ = existing_chat.close();
                                        }
                                    }
                                    
                                    // Create timestamp for unique window ID
                                    let timestamp = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap()
                                        .as_millis();
                                    let window_id = format!("chat_direct_{}", timestamp);
                                    
                                    // Create chat window centered on screen
                                    let chat_window = WebviewWindowBuilder::new(
                                        &app_handle,
                                        &window_id,
                                        tauri::WebviewUrl::App(format!("chat.html?operation=Chat&title=AI Chat&t={}", timestamp).into())
                                    )
                                    .title("AI Chat")
                                    .inner_size(900.0, 700.0)
                                    .min_inner_size(700.0, 500.0)
                                    .center()
                                    .resizable(true)
                                    .decorations(true)
                                    .closable(true)
                                    .always_on_top(false)
                                    .skip_taskbar(false)
                                    .build();
                                    
                                    if let Ok(window) = chat_window {
                                        println!("Chat window opened successfully");
                                        let _ = window.set_focus();
                                    } else {
                                        println!("Failed to create chat window");
                                    }
                                }
                            } else {
                                println!("Failed to read clipboard - opening chat window as fallback");
                                
                                // Close existing chat windows
                                let existing_chat_windows = vec!["chat", "chat_direct"];
                                for window_name in &existing_chat_windows {
                                    if let Some(existing_chat) = app_handle.get_webview_window(window_name) {
                                        let _ = existing_chat.close();
                                    }
                                }
                                
                                // Create timestamp for unique window ID
                                let timestamp = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis();
                                let window_id = format!("chat_fallback_{}", timestamp);
                                
                                // Create chat window centered on screen
                                let chat_window = WebviewWindowBuilder::new(
                                    &app_handle,
                                    &window_id,
                                    tauri::WebviewUrl::App(format!("chat.html?operation=Chat&title=AI Chat&t={}", timestamp).into())
                                )
                                .title("AI Chat")
                                .inner_size(900.0, 700.0)
                                .min_inner_size(700.0, 500.0)
                                .center()
                                .resizable(true)
                                .decorations(true)
                                .closable(true)
                                .always_on_top(false)
                                .skip_taskbar(false)
                                .build();
                                
                                if let Ok(window) = chat_window {
                                    println!("Fallback chat window opened successfully");
                                    let _ = window.set_focus();
                                } else {
                                    println!("Failed to create fallback chat window");
                                }
                            }
                        });
                    }
                })
                .build()
        })
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Hide the main window immediately on startup
            if let Some(window) = app.get_webview_window("main") {
                window.hide().unwrap();
                println!("Main window hidden on startup");
            }
            
            // Check if app_data.json exists - if not, show onboarding
            // Same logic as DataManager uses
            let config_path = if let Ok(exe_path) = std::env::current_exe() {
                exe_path.parent()
                    .map(|parent| parent.join("app_data.json"))
                    .unwrap_or_else(|| std::env::current_dir().unwrap().join("app_data.json"))
            } else {
                std::env::current_dir().unwrap().join("app_data.json")
            };
            
            if !config_path.exists() {
                println!("No app_data.json found - showing onboarding window");
                show_onboarding_window(app.handle())?;
            } else {
                println!("app_data.json found - setting up tray and global shortcut");
                create_tray(app.handle())?;
                
                // Register global shortcut (hardcoded to ctrl+space)
                match app.handle().global_shortcut().register("CmdOrCtrl+Space") {
                    Ok(()) => println!("Global shortcut 'Ctrl+Space' registered successfully!"),
                    Err(e) => println!("Failed to register global shortcut: {:?}", e),
                }
            }
            
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // For main window, hide instead of closing
                if window.label() == "main" {
                    window.hide().unwrap();
                    api.prevent_close();
                } else {
                    // For popup and chat windows, allow normal closing
                    // They will close normally
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            greet, 
            process_text,
            process_text_with_ai,
            chat_with_ai,
            test_ai_connection,
            get_ai_models,
            simulate_paste, 
            reopen_chat_conversation,
            // Data management commands (config, operations, chat, conversations)
            data_manager::dm_load_config,
            data_manager::dm_save_config,
            data_manager::dm_load_operations,
            data_manager::dm_load_operations_sorted,
            data_manager::dm_save_operations,
            data_manager::dm_get_operation,
            data_manager::dm_update_operation,
            data_manager::dm_remove_operation,
            data_manager::dm_reset_operations,
            data_manager::dm_update_api_key,
            data_manager::dm_switch_provider,
            data_manager::save_chat_entry, 
            data_manager::load_chat_history,
            data_manager::clear_chat_history,
            data_manager::save_conversation,
            data_manager::load_saved_conversations,
            data_manager::delete_saved_conversation,
            data_manager::load_conversation_messages
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
