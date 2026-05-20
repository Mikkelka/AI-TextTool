use tauri::Manager;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

use ai_provider::GlobalRateLimiter;

mod ai_provider;
mod commands;
mod data_manager;
mod shortcut_manager;
mod tray_manager;
mod utils;
mod window_manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    utils::logging::init_logging();

    let rate_limiter = GlobalRateLimiter::new(15);

    let run_result = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(shortcut_manager::create_shortcut_handler())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        .manage(rate_limiter)
        .setup(|app| {
            // Hide the main window immediately on startup
            if let Some(window) = app.get_webview_window("main") {
                match window.hide() {
                    Ok(()) => log::info!("Main window hidden on startup"),
                    Err(e) => log::warn!("Failed to hide main window on startup: {e:?}"),
                }
            }

            // Check if app_data.json exists - if not, show onboarding
            let config_path = utils::file_paths::get_app_data_path();

            if !config_path.exists() {
                log::info!("No app_data.json found - showing onboarding window");
                window_manager::show_onboarding_window(app.handle())?;
            } else {
                log::info!("app_data.json found - setting up tray and global shortcut");
                tray_manager::create_tray(app.handle())?;

                // Register global shortcut (hardcoded to ctrl+space)
                match app.handle().global_shortcut().register("CmdOrCtrl+Space") {
                    Ok(()) => log::info!("Global shortcut 'Ctrl+Space' registered successfully"),
                    Err(e) => log::error!("Failed to register global shortcut: {e:?}"),
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // For main window, hide instead of closing
                if window.label() == "main" {
                    match window.hide() {
                        Ok(()) => api.prevent_close(),
                        Err(e) => log::warn!("Failed to hide main window on close: {e:?}"),
                    }
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
            data_manager::clear_saved_conversations,
            data_manager::save_conversation,
            data_manager::load_saved_conversations,
            data_manager::delete_saved_conversation,
            data_manager::load_conversation_messages
        ])
        .run(tauri::generate_context!());

    if let Err(e) = run_result {
        log::error!("Error while running tauri application: {e:?}");
        std::process::exit(1);
    }
}
