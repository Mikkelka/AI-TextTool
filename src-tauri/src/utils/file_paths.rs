use std::path::PathBuf;

/// Get the path to app_data.json
/// Returns the path next to the executable, or current directory as fallback
pub fn get_app_data_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
        .map(|p| p.join("app_data.json"))
        .unwrap_or_else(|| {
            std::env::current_dir()
                .map(|d| d.join("app_data.json"))
                .unwrap_or_else(|_| PathBuf::from("app_data.json"))
        })
}
