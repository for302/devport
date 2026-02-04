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

/// Opens Command Prompt (cmd.exe) in the specified project directory.
#[tauri::command]
pub async fn open_in_terminal(_app_handle: AppHandle, path: String) -> Result<(), String> {
    use std::process::Command;

    // Try Windows Terminal first
    let wt_result = Command::new("wt")
        .args(["-d", &path])
        .spawn();

    if wt_result.is_ok() {
        return Ok(());
    }

    // Fallback to cmd.exe with start command
    Command::new("cmd")
        .args(["/C", "start", "cmd.exe", "/K", &format!("cd /d {}", &path)])
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
