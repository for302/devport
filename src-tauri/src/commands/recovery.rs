use crate::services::mariadb_diagnostics::{DiagnosticReport, MariaDbDiagnostics, RecoveryAction};
use crate::services::process_manager::kill_process_tree;
use crate::services::recovery::{RecoveryManager, SessionState, ServiceState, ProjectState};
use std::collections::HashMap;

/// Initialize recovery on app startup
#[tauri::command]
pub async fn init_recovery() -> Result<Vec<String>, String> {
    let recovery = RecoveryManager::new();

    // Cleanup any stale processes from previous session
    let cleaned = recovery.cleanup_stale_processes();

    Ok(cleaned)
}

/// Save current session state
#[tauri::command]
pub async fn save_session_state(
    services: HashMap<String, ServiceState>,
    projects: HashMap<String, ProjectState>,
) -> Result<(), String> {
    let recovery = RecoveryManager::new();

    let state = SessionState {
        services,
        projects,
        saved_at: chrono::Utc::now().to_rfc3339(),
    };

    recovery.save_session_state(&state)
}

/// Get services that should be auto-started
#[tauri::command]
pub async fn get_auto_start_services() -> Result<Vec<String>, String> {
    let recovery = RecoveryManager::new();
    Ok(recovery.get_auto_start_services())
}

/// Get services that were running before crash/shutdown
#[tauri::command]
pub async fn get_services_to_restore() -> Result<Vec<String>, String> {
    let recovery = RecoveryManager::new();
    Ok(recovery.get_services_to_restore())
}

/// Clear saved session state (called on clean shutdown)
#[tauri::command]
pub async fn clear_session_state() -> Result<(), String> {
    let recovery = RecoveryManager::new();
    recovery.clear_session_state()
}

/// Check if a specific PID is still running
#[tauri::command]
pub async fn is_pid_running(pid: u32) -> Result<bool, String> {
    let recovery = RecoveryManager::new();
    Ok(recovery.is_pid_running(pid))
}

/// Check if a port is in use
#[tauri::command]
pub async fn is_port_in_use(port: u16) -> Result<bool, String> {
    let recovery = RecoveryManager::new();
    Ok(recovery.is_port_in_use(port))
}

/// Get PID using a specific port
#[tauri::command]
pub async fn get_pid_on_port(port: u16) -> Result<Option<u32>, String> {
    let recovery = RecoveryManager::new();
    Ok(recovery.get_pid_on_port(port))
}

/// Kill a process by PID
#[tauri::command]
pub async fn kill_process(pid: u32) -> Result<(), String> {
    let recovery = RecoveryManager::new();
    recovery.kill_stale_process(pid)
}

/// Diagnose MariaDB failure
#[tauri::command]
pub async fn diagnose_mariadb(last_error: String) -> Result<DiagnosticReport, String> {
    Ok(tokio::task::spawn_blocking(move || {
        MariaDbDiagnostics::diagnose(&last_error)
    })
    .await
    .map_err(|e| e.to_string())?)
}

/// Execute a recovery step
#[tauri::command]
pub async fn execute_recovery_step(_step_id: String, action: RecoveryAction) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        match action {
            RecoveryAction::KillProcess { pid } => {
                let result = kill_process_tree(pid);
                if result.success {
                    Ok(format!("Process {} terminated successfully", pid))
                } else {
                    Err(result.error.unwrap_or_else(|| "Failed to kill process".to_string()))
                }
            }
            RecoveryAction::ReinitData => {
                crate::services::mariadb_diagnostics::reinitialize_data_directory(true)
            }
            RecoveryAction::ChangePort { new_port } => {
                Ok(format!("Port change to {} requested. Please update MariaDB configuration.", new_port))
            }
            RecoveryAction::FixConfig { key, value } => {
                Ok(format!("Config fix requested: {} = {}", key, value))
            }
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Check if Python is installed
#[tauri::command]
pub async fn check_python_installed() -> Result<PythonInfo, String> {
    tokio::task::spawn_blocking(|| {
        // Try python --version first
        let python_result = try_python_command("python");
        if let Some(info) = python_result {
            return Ok(info);
        }

        // Try py --version (Windows Python Launcher)
        let py_result = try_python_command("py");
        if let Some(info) = py_result {
            return Ok(info);
        }

        // Try python3 --version
        let python3_result = try_python_command("python3");
        if let Some(info) = python3_result {
            return Ok(info);
        }

        Err("Python is not installed. Please install from python.org".to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PythonInfo {
    pub version: String,
    pub path: String,
    pub has_pip: bool,
}

fn try_python_command(cmd: &str) -> Option<PythonInfo> {
    use std::process::Command;

    let mut command = Command::new(cmd);
    command.args(["--version"]);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let version = if version.is_empty() {
        String::from_utf8_lossy(&output.stderr).trim().to_string()
    } else {
        version
    };

    if version.is_empty() || !version.contains("Python") {
        return None;
    }

    // Check for pip
    let mut pip_cmd = Command::new(cmd);
    pip_cmd.args(["-m", "pip", "--version"]);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        pip_cmd.creation_flags(0x08000000);
    }

    let has_pip = pip_cmd.output().map(|o| o.status.success()).unwrap_or(false);

    Some(PythonInfo {
        version,
        path: cmd.to_string(),
        has_pip,
    })
}
