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
