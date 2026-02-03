use crate::services::ServiceManager;
use std::sync::Arc;
use tauri::{AppHandle, Manager, Runtime, WebviewWindow};
use tokio::sync::Mutex;

/// Update the system tray icon and tooltip based on current service status
#[tauri::command]
pub async fn update_tray_status<R: Runtime>(
    app: AppHandle<R>,
    service_manager: tauri::State<'_, Arc<Mutex<ServiceManager>>>,
) -> Result<(), String> {
    let manager = service_manager.lock().await;

    // Check if any service is running
    let any_running = manager
        .services
        .values()
        .any(|service| service.is_running());

    // Get the tray icon and update tooltip based on status
    if let Some(tray) = app.tray_by_id("main-tray") {
        let tooltip = if any_running {
            "ClickDevPort - Services Running"
        } else {
            "ClickDevPort - No Services Running"
        };
        tray.set_tooltip(Some(tooltip)).map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Get the current tray status
#[tauri::command]
pub async fn get_tray_status<R: Runtime>(
    app: AppHandle<R>,
    service_manager: tauri::State<'_, Arc<Mutex<ServiceManager>>>,
) -> Result<TrayStatus, String> {
    let manager = service_manager.lock().await;

    let running_count = manager
        .services
        .values()
        .filter(|service| service.is_running())
        .count();

    let total_count = manager.services.len();

    let is_visible = app
        .get_webview_window("main")
        .map(|w: WebviewWindow<R>| w.is_visible().unwrap_or(false))
        .unwrap_or(false);

    Ok(TrayStatus {
        running_services: running_count,
        total_services: total_count,
        is_visible,
    })
}

#[derive(serde::Serialize)]
pub struct TrayStatus {
    pub running_services: usize,
    pub total_services: usize,
    pub is_visible: bool,
}
