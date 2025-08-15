use tauri::{Runtime, WebviewWindowBuilder, Manager, Emitter};

/// Show the onboarding window for first-time setup
pub fn show_onboarding_window<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
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
            if let Err(e) = super::tray_manager::create_tray(&app_handle) {
                eprintln!("Failed to create tray after onboarding: {:?}", e);
            }
            
            // Register global shortcut after onboarding (hardcoded to ctrl+space)
            use tauri_plugin_global_shortcut::GlobalShortcutExt;
            match app_handle.global_shortcut().register("CmdOrCtrl+Space") {
                Ok(()) => println!("Global shortcut registered after onboarding"),
                Err(e) => println!("Failed to register shortcut after onboarding: {:?}", e),
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
    clipboard_text: String
) -> tauri::Result<()> {
    // Close existing popup windows
    if let Some(existing_popup) = app.get_webview_window("popup") {
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
        app,
        &window_label,
        tauri::WebviewUrl::App(format!("popup.html?t={}", timestamp).into())
    )
    .title("AI Text Operations")
    .inner_size(320.0, 350.0)  // Compact 2-column layout without header
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
    .build()?;
    
    println!("Popup window created successfully");
    // Send clipboard text directly to popup window
    let _ = popup_window.emit("set-clipboard-text", clipboard_text);
    
    Ok(())
}

/// Create a direct chat window (when no text is selected)
pub fn create_direct_chat_window<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    println!("No text selected - opening chat window directly");
    
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
    let window_id = format!("chat_direct_{}", timestamp);
    
    // Create chat window centered on screen
    let chat_window = WebviewWindowBuilder::new(
        app,
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
    .build()?;
    
    println!("Chat window opened successfully");
    let _ = chat_window.set_focus();
    
    Ok(())
}

/// Create a fallback chat window (when clipboard fails)
pub fn create_fallback_chat_window<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    println!("Failed to read clipboard - opening chat window as fallback");
    
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
    let window_id = format!("chat_fallback_{}", timestamp);
    
    // Create chat window centered on screen
    let chat_window = WebviewWindowBuilder::new(
        app,
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
    .build()?;
    
    println!("Fallback chat window opened successfully");
    let _ = chat_window.set_focus();
    
    Ok(())
}

/// Create a chat window from tray menu
pub fn create_tray_chat_window<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
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
    let chat_window = WebviewWindowBuilder::new(
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
    .build()?;
    
    println!("Chat window opened successfully from tray");
    let _ = chat_window.set_focus();
    
    Ok(())
}

/// Create a settings window from tray menu
pub fn create_settings_window<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    println!("Opening settings window...");
    
    // Close existing settings window if open
    if let Some(existing_settings) = app.get_webview_window("settings") {
        let _ = existing_settings.close();
    }
    
    // Create settings window
    let settings_window = WebviewWindowBuilder::new(
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
    .build()?;
    
    println!("Settings window opened successfully");
    let _ = settings_window.set_focus();
    
    Ok(())
}

/// Create a chat history window from tray menu
pub fn create_chat_history_window<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    println!("Opening chat history window...");
    
    // Close existing history window if open
    if let Some(existing_history) = app.get_webview_window("chat_history") {
        let _ = existing_history.close();
    }
    
    // Create chat history window
    let history_window = WebviewWindowBuilder::new(
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
    .build()?;
    
    println!("Chat history window opened successfully");
    let _ = history_window.set_focus();
    
    Ok(())
}

/// Create an edit operations window from tray menu
pub fn create_edit_operations_window<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    println!("Opening edit operations window...");
    
    // Close existing edit operations window if open
    if let Some(existing_edit) = app.get_webview_window("edit_operations") {
        let _ = existing_edit.close();
    }
    
    // Create edit operations window
    let edit_window = WebviewWindowBuilder::new(
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
    .build()?;
    
    println!("Edit operations window opened successfully");
    let _ = edit_window.set_focus();
    
    Ok(())
}