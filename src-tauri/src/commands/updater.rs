use crate::services::updater::{UpdateCheckResult, UpdateInfo, UpdateManager};

/// Check for available updates
#[tauri::command]
pub async fn check_for_updates() -> Result<UpdateCheckResult, String> {
    let manager = UpdateManager::new();
    Ok(manager.check_for_updates().await)
}

/// Get the current application version
#[tauri::command]
pub fn get_current_version() -> String {
    UpdateManager::new().get_current_version()
}

/// Download the update and return the file path
#[tauri::command]
pub async fn download_update(update_info: UpdateInfo) -> Result<String, String> {
    let manager = UpdateManager::new();
    manager
        .download_update(&update_info)
        .await
        .map_err(|e| e.to_string())
}

/// Get the GitHub releases URL for manual download
#[tauri::command]
pub fn get_releases_url() -> String {
    UpdateManager::new().get_releases_url()
}

/// Open the downloaded update file to install
#[tauri::command]
pub async fn install_update(file_path: String) -> Result<(), String> {
    // On Windows, we can use the shell to open/run the installer
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("cmd")
            .args(["/C", "start", "", &file_path])
            .spawn()
            .map_err(|e| format!("Failed to start installer: {}", e))?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        use std::process::Command;
        Command::new("open")
            .arg(&file_path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }

    Ok(())
}
