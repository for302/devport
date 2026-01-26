use crate::services::scheduler::SchedulerManager;

/// Register DevPort Manager to auto-start at Windows logon
#[tauri::command]
pub async fn register_auto_start() -> Result<(), String> {
    let scheduler = SchedulerManager::new();
    scheduler.register_auto_start()
}

/// Unregister DevPort Manager from auto-start
#[tauri::command]
pub async fn unregister_auto_start() -> Result<(), String> {
    let scheduler = SchedulerManager::new();
    scheduler.unregister_auto_start()
}

/// Check if auto-start is enabled in Windows Task Scheduler
#[tauri::command]
pub async fn is_auto_start_enabled() -> Result<bool, String> {
    let scheduler = SchedulerManager::new();
    scheduler.is_auto_start_enabled()
}

/// Get the list of services configured for auto-start
#[tauri::command]
pub async fn get_scheduler_auto_start_services() -> Result<Vec<String>, String> {
    let scheduler = SchedulerManager::new();
    Ok(scheduler.get_auto_start_services())
}

/// Set the list of services to auto-start when the app launches
#[tauri::command]
pub async fn set_scheduler_auto_start_services(services: Vec<String>) -> Result<(), String> {
    let scheduler = SchedulerManager::new();
    scheduler.set_auto_start_services(services)
}
