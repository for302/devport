use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;

/// Opens a project directory in VS Code using the `code` command.
#[tauri::command]
pub async fn open_in_vscode(app_handle: AppHandle, path: String) -> Result<(), String> {
    app_handle
        .shell()
        .command("code")
        .arg(&path)
        .spawn()
        .map_err(|e| format!("Failed to open VS Code: {}", e))?;

    Ok(())
}

/// Opens PowerShell in the specified project directory.
#[tauri::command]
pub async fn open_in_terminal(app_handle: AppHandle, path: String) -> Result<(), String> {
    app_handle
        .shell()
        .command("powershell")
        .args(["-NoExit", "-Command", &format!("cd '{}'", path)])
        .spawn()
        .map_err(|e| format!("Failed to open terminal: {}", e))?;

    Ok(())
}

/// Opens a URL in the default system browser.
#[tauri::command]
pub async fn open_in_browser(app_handle: AppHandle, url: String) -> Result<(), String> {
    app_handle
        .shell()
        .command("cmd")
        .args(["/C", "start", "", &url])
        .spawn()
        .map_err(|e| format!("Failed to open browser: {}", e))?;

    Ok(())
}

/// Opens the project folder in Windows Explorer.
#[tauri::command]
pub async fn open_file_explorer(app_handle: AppHandle, path: String) -> Result<(), String> {
    app_handle
        .shell()
        .command("explorer")
        .arg(&path)
        .spawn()
        .map_err(|e| format!("Failed to open file explorer: {}", e))?;

    Ok(())
}
