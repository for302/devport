use crate::services::uninstaller::{UninstallManager, UninstallMode, UninstallPreview, UninstallResult};
use crate::services::ServiceManager;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

/// Get a preview of what will be deleted for the given uninstall mode
#[tauri::command]
pub async fn get_uninstall_preview(mode: UninstallMode) -> Result<UninstallPreview, String> {
    let manager = UninstallManager::new();
    Ok(manager.get_uninstall_preview(&mode))
}

/// Perform the uninstall operation
#[tauri::command]
pub async fn perform_uninstall(
    mode: UninstallMode,
    service_manager: State<'_, Arc<Mutex<ServiceManager>>>,
) -> Result<UninstallResult, String> {
    let manager = UninstallManager::new();

    // Stop all services first
    {
        let mut sm = service_manager.lock().await;
        let _ = sm.stop_service("apache").await;
        let _ = sm.stop_service("mariadb").await;
    }

    // Also stop via system commands to ensure cleanup
    let _ = manager.stop_all_services();
    let _ = manager.stop_all_projects();

    // Perform uninstall based on mode
    let result = match mode {
        UninstallMode::Basic => manager.uninstall_basic().await,
        UninstallMode::FullData => manager.uninstall_full_data().await,
        UninstallMode::SystemRevert => manager.uninstall_system_revert().await,
    };

    Ok(result)
}

/// Stop all running services and projects for uninstall
#[tauri::command]
pub async fn stop_all_for_uninstall(
    service_manager: State<'_, Arc<Mutex<ServiceManager>>>,
) -> Result<bool, String> {
    let manager = UninstallManager::new();

    // Stop services via service manager
    {
        let mut sm = service_manager.lock().await;
        let _ = sm.stop_service("apache").await;
        let _ = sm.stop_service("mariadb").await;
    }

    // Also stop via system commands
    manager.stop_all_services().map_err(|e| e.to_string())?;
    manager.stop_all_projects().map_err(|e| e.to_string())?;

    Ok(true)
}

/// Check if any services or projects are currently running
#[tauri::command]
pub async fn check_running_processes(
    service_manager: State<'_, Arc<Mutex<ServiceManager>>>,
) -> Result<RunningProcessesInfo, String> {
    let sm = service_manager.lock().await;
    let statuses = sm.get_all_statuses();

    let apache_running = statuses
        .get("apache")
        .map(|s| format!("{:?}", s).to_lowercase() == "running")
        .unwrap_or(false);

    let mariadb_running = statuses
        .get("mariadb")
        .map(|s| format!("{:?}", s).to_lowercase() == "running")
        .unwrap_or(false);

    Ok(RunningProcessesInfo {
        apache_running,
        mariadb_running,
        any_running: apache_running || mariadb_running,
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunningProcessesInfo {
    pub apache_running: bool,
    pub mariadb_running: bool,
    pub any_running: bool,
}
