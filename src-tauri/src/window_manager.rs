use tauri::{Emitter, Manager, Runtime, WebviewWindowBuilder};

use crate::utils::time;

/// Configuration for creating a window
#[derive(Clone)]
pub struct WindowConfig {
    pub window_id: String,
    pub url: String,
    pub title: String,
    pub width: f64,
    pub height: f64,
    pub min_width: Option<f64>,
    pub min_height: Option<f64>,
    pub position: WindowPosition,
    pub resizable: bool,
    pub maximizable: bool,
    pub minimizable: bool,
    pub closable: bool,
    pub always_on_top: bool,
    pub skip_taskbar: bool,
    pub decorations: bool,
    pub close_existing: Vec<String>,
    pub initialization_script: Option<String>,
}

/// Window positioning options
#[derive(Clone)]
pub enum WindowPosition {
    Center,
    Coordinates { x: f64, y: f64 },
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            window_id: "default".to_string(),
            url: "windows/index.html".to_string(),
            title: "AI TextTool".to_string(),
            width: 800.0,
            height: 600.0,
            min_width: None,
            min_height: None,
            position: WindowPosition::Center,
            resizable: true,
            maximizable: true,
            minimizable: true,
            closable: true,
            always_on_top: false,
            skip_taskbar: false,
            decorations: false,
            close_existing: vec![],
            initialization_script: None,
        }
    }
}

/// Generic window creation function that handles all common patterns
pub fn create_window<R: Runtime>(
    app: &tauri::AppHandle<R>,
    config: WindowConfig,
) -> tauri::Result<()> {
    // Close existing windows if specified
    for window_name in &config.close_existing {
        if let Some(existing_window) = app.get_webview_window(window_name) {
            let _ = existing_window.destroy();
        }
    }

    // Small delay for popup windows to ensure cleanup
    if config.window_id.contains("popup") {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    log::info!(
        "Creating window '{}' with title '{}'",
        config.window_id, config.title
    );

    // Build the window
    let mut builder = WebviewWindowBuilder::new(
        app,
        &config.window_id,
        tauri::WebviewUrl::App(config.url.into()),
    )
    .title(&config.title)
    .inner_size(config.width, config.height)
    .resizable(config.resizable)
    .maximizable(config.maximizable)
    .minimizable(config.minimizable)
    .closable(config.closable)
    .always_on_top(config.always_on_top)
    .skip_taskbar(config.skip_taskbar)
    .decorations(config.decorations);

    // Set minimum size if specified
    if let (Some(min_width), Some(min_height)) = (config.min_width, config.min_height) {
        builder = builder.min_inner_size(min_width, min_height);
    }

    // Set position
    match config.position {
        WindowPosition::Center => {
            builder = builder.center();
        }
        WindowPosition::Coordinates { x, y } => {
            builder = builder.position(x, y);
        }
    }

    // Add initialization script if provided
    if let Some(script) = &config.initialization_script {
        builder = builder.initialization_script(script);
    }

    // Build and focus the window
    let window = builder.build()?;
    let _ = window.set_focus();

    log::info!("Window '{}' created successfully", config.window_id);
    Ok(())
}

/// Show the onboarding window for first-time setup
pub fn show_onboarding_window<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    log::info!("Creating onboarding window");

    let onboarding_window = WebviewWindowBuilder::new(
        app,
        "onboarding",
        tauri::WebviewUrl::App("windows/onboarding.html".into()),
    )
    .title("AI TextTool - Setup")
    .inner_size(700.0, 900.0)
    .min_inner_size(600.0, 650.0)
    .center()
    .resizable(true)
    .maximizable(false)
    .minimizable(false)
    .closable(true)
    .always_on_top(false)
    .skip_taskbar(false)
    .decorations(false)
    .build()?;

    log::info!("Onboarding window created successfully");

    // Listen for onboarding window close event
    let app_handle = app.clone();
    onboarding_window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { .. } = event {
            log::info!("Onboarding window closed - setting up tray and shortcuts");

            // After onboarding is closed, set up tray and shortcuts
            if let Err(e) = super::tray_manager::create_tray(&app_handle) {
                log::error!("Failed to create tray after onboarding: {e:?}");
            }

            // Register global shortcut after onboarding (hardcoded to ctrl+space)
            use tauri_plugin_global_shortcut::GlobalShortcutExt;
            match app_handle.global_shortcut().register("CmdOrCtrl+Space") {
                Ok(()) => log::info!("Global shortcut registered after onboarding"),
                Err(e) => log::error!("Failed to register shortcut after onboarding: {e:?}"),
            }
        }
    });

    Ok(())
}

/// Create a popup window at the specified position with clipboard text injected
pub fn create_popup_window<R: Runtime>(
    app: &tauri::AppHandle<R>,
    mouse_x: i32,
    mouse_y: i32,
    clipboard_text: String,
) -> tauri::Result<()> {
    // Create timestamp for unique URL
    let timestamp = time::get_current_timestamp_millis();

    log::info!(
        "Creating popup window at mouse position: ({}, {})",
        mouse_x, mouse_y
    );

    let config = WindowConfig {
        window_id: "popup".to_string(),
        url: format!("windows/popup.html?t={}", timestamp),
        title: "AI TextTool - Operations".to_string(),
        width: 300.0,
        height: 290.0,
        min_width: None,
        min_height: None,
        position: WindowPosition::Coordinates {
            x: mouse_x as f64 - 150.0,  // Center horizontally (width / 2)
            y: mouse_y as f64 - 300.0,  // Position above mouse (height + margin)
        },
        resizable: false,
        maximizable: false,
        minimizable: false,
        closable: true,
        always_on_top: true,
        skip_taskbar: true,
        decorations: false,
        close_existing: vec!["popup".to_string()],
        initialization_script: Some(format!(
            "window.clipboardText = {};",
            serde_json::to_string(&clipboard_text).unwrap_or_else(|_| "\"\"".to_string())
        )),
    };

    create_window(app, config)?;

    // Send clipboard text directly to popup window (preserve existing behavior)
    if let Some(popup_window) = app.get_webview_window("popup") {
        let _ = popup_window.emit("set-clipboard-text", clipboard_text);
    }

    Ok(())
}

/// Create a chat window with specified context
fn create_chat_window<R: Runtime>(
    app: &tauri::AppHandle<R>,
    prefix: &str,
    context: &str,
) -> tauri::Result<()> {
    log::info!("{context}");

    // Create timestamp for unique window ID
    let timestamp = time::get_current_timestamp_millis();

    let config = WindowConfig {
        window_id: format!("chat_{}_{}", prefix, timestamp),
        url: format!("windows/chat.html?operation=Chat&title=AI Chat&t={}", timestamp),
        title: "AI TextTool - Chat".to_string(),
        width: 900.0,
        height: 700.0,
        min_width: Some(700.0),
        min_height: Some(500.0),
        position: WindowPosition::Center,
        resizable: true,
        maximizable: prefix == "tray", // Only tray variant is maximizable
        minimizable: prefix == "tray", // Only tray variant is minimizable
        closable: true,
        always_on_top: false,
        skip_taskbar: false,
        decorations: false,
        close_existing: vec!["chat".to_string(), "chat_direct".to_string()],
        initialization_script: None,
    };

    create_window(app, config)
}

/// Create a direct chat window (when no text is selected)
pub fn create_direct_chat_window<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    create_chat_window(
        app,
        "direct",
        "No text selected - opening chat window directly",
    )
}

/// Create a fallback chat window (when clipboard fails)
pub fn create_fallback_chat_window<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    create_chat_window(
        app,
        "fallback",
        "Failed to read clipboard - opening chat window as fallback",
    )
}

/// Create a chat window from tray menu
pub fn create_tray_chat_window<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    create_chat_window(app, "tray", "Opening chat window from tray...")
}

/// Create a settings window from tray menu
pub fn create_settings_window<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    log::info!("Opening settings window...");

    let config = WindowConfig {
        window_id: "settings".to_string(),
        url: "windows/settings.html".to_string(),
        title: "Settings - AI TextTool".to_string(),
        width: 600.0,
        height: 700.0,
        min_width: Some(500.0),
        min_height: Some(600.0),
        position: WindowPosition::Center,
        resizable: true,
        maximizable: false,
        minimizable: true,
        closable: true,
        always_on_top: false,
        skip_taskbar: false,
        decorations: false,
        close_existing: vec!["settings".to_string()],
        initialization_script: None,
    };

    create_window(app, config)
}

/// Create a chat history window from tray menu
pub fn create_chat_history_window<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    log::info!("Opening chat history window...");

    let config = WindowConfig {
        window_id: "chat_history".to_string(),
        url: "windows/history.html".to_string(),
        title: "Chat History - AI TextTool".to_string(),
        width: 1000.0,
        height: 700.0,
        min_width: Some(800.0),
        min_height: Some(600.0),
        position: WindowPosition::Center,
        resizable: true,
        maximizable: true,
        minimizable: true,
        closable: true,
        always_on_top: false,
        skip_taskbar: false,
        decorations: false,
        close_existing: vec!["chat_history".to_string()],
        initialization_script: None,
    };

    create_window(app, config)
}

/// Create an edit operations window from tray menu
pub fn create_edit_operations_window<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    log::info!("Opening edit operations window...");

    let config = WindowConfig {
        window_id: "edit_operations".to_string(),
        url: "windows/operation-edit.html".to_string(),
        title: "Edit Operations - AI TextTool".to_string(),
        width: 900.0,
        height: 700.0,
        min_width: Some(700.0),
        min_height: Some(500.0),
        position: WindowPosition::Center,
        resizable: true,
        maximizable: true,
        minimizable: true,
        closable: true,
        always_on_top: false,
        skip_taskbar: false,
        decorations: false,
        close_existing: vec!["edit_operations".to_string()],
        initialization_script: None,
    };

    create_window(app, config)
}
