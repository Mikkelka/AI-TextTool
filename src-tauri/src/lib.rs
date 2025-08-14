use tauri::Manager;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

mod ai_provider;
mod data_manager;
mod window_manager;
mod tray_manager;
mod shortcut_manager;
mod commands;



#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(shortcut_manager::create_shortcut_handler())
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
                window_manager::show_onboarding_window(app.handle())?;
            } else {
                println!("app_data.json found - setting up tray and global shortcut");
                tray_manager::create_tray(app.handle())?;
                
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
            // Utility commands
            commands::greet, 
            commands::process_text,
            commands::simulate_paste, 
            // AI commands
            commands::process_text_with_ai,
            commands::chat_with_ai,
            commands::test_ai_connection,
            commands::get_ai_models,
            // Window commands
            commands::reopen_chat_conversation,
            commands::open_chat_window,
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
