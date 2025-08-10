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

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn create_tray<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

    // Use the default app icon from the bundle
    let _ = TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "quit" => {
                app.exit(0);
            }
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
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
                                    let mut enigo_mouse = Enigo::new(&Settings::default()).unwrap();
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
                                    .title("Captured Text")
                                    .inner_size(400.0, 200.0)
                                    .position(mouse_x as f64, mouse_y as f64)
                                    .resizable(false)
                                    .decorations(true)
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
        .setup(|app| {
            create_tray(app.handle())?;
            
            // Register global shortcut Ctrl+Space
            println!("Registering global shortcut: CmdOrCtrl+Space");
            match app.global_shortcut().register("CmdOrCtrl+Space") {
                Ok(()) => println!("Global shortcut registered successfully!"),
                Err(e) => println!("Failed to register global shortcut: {:?}", e),
            }
            
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Prevent the window from closing and hide it instead
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
