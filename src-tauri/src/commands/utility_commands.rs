use tauri_plugin_clipboard_manager::ClipboardExt;

use super::super::shortcut_manager;

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub fn process_text(text: String, option: String, app: tauri::AppHandle) -> Result<String, String> {
    log::info!("Processing text with option '{}'", option);

    let processed_text = match option.as_str() {
        "add_word" => format!("{} [PROCESSED]", text),
        "fix_grammar" => format!("{} [GRAMMAR FIXED]", text), // Placeholder
        "translate" => format!("{} [TRANSLATED]", text),      // Placeholder
        _ => text,                                            // Return original if unknown option
    };

    log::debug!("Processed result length: {}", processed_text.len());

    // Put processed text in clipboard
    if let Err(e) = app.clipboard().write_text(processed_text.clone()) {
        return Err(format!("Failed to write to clipboard: {:?}", e));
    }

    log::debug!("Text put in clipboard, will auto-paste after popup closes");

    Ok(processed_text)
}

#[tauri::command]
pub fn simulate_paste() -> Result<String, String> {
    shortcut_manager::simulate_paste()
}
