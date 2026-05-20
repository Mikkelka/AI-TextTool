use super::super::shortcut_manager;

#[tauri::command]
pub fn simulate_paste() -> Result<String, String> {
    shortcut_manager::simulate_paste()
}
