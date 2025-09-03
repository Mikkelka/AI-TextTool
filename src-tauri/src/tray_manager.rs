use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Runtime,
};

use super::window_manager;

/// Create and configure the system tray with menu
pub fn create_tray<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    let chat_i = MenuItem::with_id(app, "chat", "Chat", true, None::<&str>)?;
    let separator1 = tauri::menu::PredefinedMenuItem::separator(app)?;
    let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let edit_operations_i = MenuItem::with_id(
        app,
        "edit_operations",
        "Edit Operations",
        true,
        None::<&str>,
    )?;
    let chat_history_i =
        MenuItem::with_id(app, "chat_history", "Chat History", true, None::<&str>)?;
    let separator2 = tauri::menu::PredefinedMenuItem::separator(app)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &chat_i,
            &separator1,
            &settings_i,
            &edit_operations_i,
            &chat_history_i,
            &separator2,
            &quit_i,
        ],
    )?;

    // Use the default app icon from the bundle
    let _ = TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(move |app, event| handle_tray_menu_event(app, event.id.as_ref()))
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

/// Handle tray menu events
fn handle_tray_menu_event<R: Runtime>(app: &tauri::AppHandle<R>, menu_id: &str) {
    match menu_id {
        "quit" => {
            app.exit(0);
        }
        "chat" => {
            if let Err(e) = window_manager::create_tray_chat_window(app) {
                eprintln!("Failed to create chat window from tray: {:?}", e);
            }
        }
        "settings" => {
            if let Err(e) = window_manager::create_settings_window(app) {
                eprintln!("Failed to create settings window: {:?}", e);
            }
        }
        "chat_history" => {
            if let Err(e) = window_manager::create_chat_history_window(app) {
                eprintln!("Failed to create chat history window: {:?}", e);
            }
        }
        "edit_operations" => {
            if let Err(e) = window_manager::create_edit_operations_window(app) {
                eprintln!("Failed to create edit operations window: {:?}", e);
            }
        }
        _ => {}
    }
}
