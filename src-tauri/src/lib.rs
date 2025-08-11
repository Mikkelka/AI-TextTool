use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime, Emitter, WebviewWindowBuilder,
};
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tauri_plugin_clipboard_manager::ClipboardExt;
use enigo::{Enigo, Key, Keyboard, Settings, Mouse};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use serde::{Serialize, Deserialize};
use ai_provider::{GeminiProvider, ChatMessage};

mod config;
mod ai_provider;

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
    let config = match config::load_config(app.clone()).await {
        Ok(config) => config,
        Err(e) => return Err(format!("Failed to load config: {}", e)),
    };
    
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
    let operation_details = match config::get_operation(app.clone(), operation.clone()).await {
        Ok(Some(op)) => op,
        Ok(None) => return Err(format!("Operation '{}' not found", operation)),
        Err(e) => return Err(format!("Failed to get operation: {}", e)),
    };
    
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
    let config = match config::load_config(app).await {
        Ok(config) => config,
        Err(e) => return Err(format!("Failed to load config: {}", e)),
    };
    
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
    
    let config = match config::load_config(app).await {
        Ok(config) => config,
        Err(e) => return Err(format!("Failed to load config: {}", e)),
    };
    
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

#[derive(Serialize, Deserialize, Clone)]
struct ChatEntry {
    timestamp: String,
    original_text: String,
    ai_option: String,
    processed_text: String,
}

#[tauri::command]
async fn save_chat_entry(app: tauri::AppHandle, original_text: String, ai_option: String, processed_text: String) -> Result<String, String> {
    println!("Saving chat entry: {} -> {}", original_text, processed_text);
    
    // Create chat entry
    let entry = ChatEntry {
        timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        original_text,
        ai_option,
        processed_text,
    };
    
    // Get directory next to executable, fallback to app data
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()));
    
    let history_file = if let Some(exe_path) = exe_dir {
        println!("Saving chat history next to exe: {:?}", exe_path.join("chat_history.json"));
        exe_path.join("chat_history.json")
    } else {
        // Fallback to app data directory
        let app_data_dir = app.path().app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {:?}", e))?;
        let chats_dir = app_data_dir.join("chats");
        std::fs::create_dir_all(&chats_dir)
            .map_err(|e| format!("Failed to create chats directory: {:?}", e))?;
        println!("Saving chat history in app data: {:?}", chats_dir.join("chat_history.json"));
        chats_dir.join("chat_history.json")
    };
    let mut chat_history: Vec<ChatEntry> = if history_file.exists() {
        let content = std::fs::read_to_string(&history_file)
            .map_err(|e| format!("Failed to read chat history: {:?}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse chat history: {:?}", e))?
    } else {
        Vec::new()
    };
    
    // Add new entry
    chat_history.push(entry);
    
    // Keep only last 100 entries
    if chat_history.len() > 100 {
        chat_history.drain(0..chat_history.len()-100);
    }
    
    // Save back to file
    let json_content = serde_json::to_string_pretty(&chat_history)
        .map_err(|e| format!("Failed to serialize chat history: {:?}", e))?;
    
    std::fs::write(&history_file, json_content)
        .map_err(|e| format!("Failed to write chat history: {:?}", e))?;
    
    println!("Chat entry saved successfully to: {:?}", history_file);
    Ok("Chat saved successfully".to_string())
}

#[tauri::command]
async fn load_chat_history(app: tauri::AppHandle) -> Result<Vec<ChatEntry>, String> {
    println!("Loading chat history...");
    
    // Get directory next to executable, fallback to app data
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()));
    
    let history_file = if let Some(exe_path) = exe_dir {
        println!("Loading chat history from exe directory: {:?}", exe_path.join("chat_history.json"));
        exe_path.join("chat_history.json")
    } else {
        // Fallback to app data directory
        let app_data_dir = app.path().app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {:?}", e))?;
        println!("Loading chat history from app data: {:?}", app_data_dir.join("chats").join("chat_history.json"));
        app_data_dir.join("chats").join("chat_history.json")
    };
    
    if !history_file.exists() {
        return Ok(Vec::new());
    }
    
    let content = std::fs::read_to_string(&history_file)
        .map_err(|e| format!("Failed to read chat history: {:?}", e))?;
    
    let chat_history: Vec<ChatEntry> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse chat history: {:?}", e))?;
    
    println!("Loaded {} chat entries", chat_history.len());
    Ok(chat_history)
}

#[tauri::command]
async fn clear_chat_history(app: tauri::AppHandle) -> Result<String, String> {
    println!("Clearing chat history...");
    
    // Get directory next to executable, fallback to app data  
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()));
    
    let history_file = if let Some(exe_path) = exe_dir {
        println!("Clearing chat history from exe directory: {:?}", exe_path.join("chat_history.json"));
        exe_path.join("chat_history.json")
    } else {
        // Fallback to app data directory
        let app_data_dir = app.path().app_data_dir()
            .map_err(|e| format!("Failed to get app data dir: {:?}", e))?;
        println!("Clearing chat history from app data: {:?}", app_data_dir.join("chats").join("chat_history.json"));
        app_data_dir.join("chats").join("chat_history.json")
    };
    
    if history_file.exists() {
        std::fs::write(&history_file, "[]")
            .map_err(|e| format!("Failed to clear chat history: {:?}", e))?;
    }
    
    println!("Chat history cleared successfully");
    Ok("Chat history cleared successfully".to_string())
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
            
            // Register global shortcut
            let shortcuts = app_handle.global_shortcut();
            match shortcuts.register("CmdOrCtrl+Space") {
                Ok(()) => println!("Global shortcut registered after onboarding!"),
                Err(e) => println!("Failed to register shortcut after onboarding: {:?}", e),
            }
        }
    });
    
    Ok(())
}

fn create_tray<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let chat_history_i = MenuItem::with_id(app, "chat_history", "Chat History", true, None::<&str>)?;
    let separator = tauri::menu::PredefinedMenuItem::separator(app)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&settings_i, &chat_history_i, &separator, &quit_i])?;

    // Use the default app icon from the bundle
    let _ = TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin({
            let last_trigger = Arc::new(Mutex::new(Instant::now() - std::time::Duration::from_millis(1000)));
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, _event| {
                    println!("Global shortcut triggered: {}", shortcut.to_string());
                    let shortcut_str = shortcut.to_string().to_lowercase();
                    if shortcut_str == "control+space" || shortcut_str == "cmd+space" {
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
                        
                        println!("Handling Ctrl+Space shortcut");
                        let app_handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            // Automatically copy selected text by simulating Ctrl+C
                            println!("Simulating Ctrl+C...");
                            let mut enigo = Enigo::new(&Settings::default()).unwrap();
                            enigo.key(Key::Control, enigo::Direction::Press).unwrap();
                            enigo.key(Key::Unicode('c'), enigo::Direction::Click).unwrap();
                            enigo.key(Key::Control, enigo::Direction::Release).unwrap();
                            
                            // Small delay to let the copy operation complete
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            
                            // Get clipboard content (this will be the selected text after Ctrl+C)
                            println!("Reading clipboard...");
                            if let Ok(clipboard_text) = app_handle.clipboard().read_text() {
                                println!("Clipboard content: '{}'", clipboard_text);
                                if !clipboard_text.trim().is_empty() {
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
                                }
                            } else {
                                println!("Failed to read clipboard");
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
            
            // Check if config exists - if not, show onboarding
            let config_path = app.path().app_data_dir()
                .map(|dir| dir.join("config.json"))
                .unwrap_or_else(|_| std::env::current_dir().unwrap().join("config.json"));
            
            if !config_path.exists() {
                println!("No config found - showing onboarding window");
                show_onboarding_window(app.handle())?;
            } else {
                println!("Config found - setting up tray and global shortcut");
                create_tray(app.handle())?;
                
                // Register global shortcut Ctrl+Space
                println!("Registering global shortcut: CmdOrCtrl+Space");
                match app.global_shortcut().register("CmdOrCtrl+Space") {
                    Ok(()) => println!("Global shortcut registered successfully!"),
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
            save_chat_entry, 
            load_chat_history,
            clear_chat_history,
            config::load_config,
            config::save_config,
            config::load_operations,
            config::save_operations,
            config::get_operation,
            config::update_api_key,
            config::update_shortcut,
            config::switch_provider,
            config::update_operation,
            config::remove_operation
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
