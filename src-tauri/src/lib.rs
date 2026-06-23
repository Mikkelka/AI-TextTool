use tauri::Manager;

use ai_provider::{GlobalRateLimiter, SharedHttpClient};
use data_manager::SharedDataManager;
use utils::file_paths;

mod ai_provider;
mod commands;
mod data_manager;
mod shortcut_manager;
mod tray_manager;
mod utils;
mod window_manager;

/// Default global shortcut (Tauri v2 format)
const DEFAULT_SHORTCUT: &str = "CmdOrCtrl+Space";

/// Register the global shortcut, reading from config if available
pub(crate) fn register_global_shortcut<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    let shortcut = get_configured_shortcut();
    match app.global_shortcut().register(shortcut.as_str()) {
        Ok(()) => log::info!("Global shortcut '{}' registered successfully", shortcut),
        Err(e) => log::error!("Failed to register global shortcut '{}': {e:?}", shortcut),
    }
}

/// Read the configured shortcut from app_data.json, falling back to default
fn get_configured_shortcut() -> String {
    let config_path = file_paths::get_app_data_path();
    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(shortcut) = data.get("config").and_then(|c| c.get("shortcut")) {
                    if let Some(s) = shortcut.as_str() {
                        if !s.is_empty() {
                            return s.to_string();
                        }
                    }
                }
            }
        }
    }
    DEFAULT_SHORTCUT.to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    utils::logging::init_logging();

    let rate_limiter = GlobalRateLimiter::new(15);
    let http_client = SharedHttpClient::new();

    let run_result = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(shortcut_manager::create_shortcut_handler())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_fs::init())
        .manage(rate_limiter)
        .manage(http_client)
        .setup(|app| {
            // Check if app_data.json exists BEFORE initializing the data manager.
            // initialize() creates the file via save_data() when it's missing, so
            // checking afterwards would always return true and skip onboarding.
            let config_path = utils::file_paths::get_app_data_path();
            let is_first_run = !config_path.exists();

            // Initialize shared data manager (blocking on async init)
            let dm = tauri::async_runtime::block_on(SharedDataManager::new())
                .expect("Failed to initialize data manager");
            app.manage(dm);

            // Hide the main window immediately on startup
            if let Some(window) = app.get_webview_window("main") {
                match window.hide() {
                    Ok(()) => log::info!("Main window hidden on startup"),
                    Err(e) => log::warn!("Failed to hide main window on startup: {e:?}"),
                }
            }

            if is_first_run {
                log::info!("No app_data.json found - showing onboarding window");
                window_manager::show_onboarding_window(app.handle())?;
            } else {
                log::info!("app_data.json found - setting up tray and global shortcut");
                tray_manager::create_tray(app.handle())?;
                register_global_shortcut(app.handle());
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
    }
}
