use crate::models::process_info::ProcessInfo;
use crate::models::ProjectType;
use crate::services::process_manager::{ProcessManager, kill_process_tree_silent};
use crate::services::project_detector::ProjectDetector;
use crate::services::storage::Storage;
use crate::state::AppState;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;

#[tauri::command]
pub async fn start_project(
    project_id: String,
    state: State<'_, Arc<Mutex<AppState>>>,
    app_handle: AppHandle,
) -> Result<ProcessInfo, String> {
    let storage = Storage::new().map_err(|e| e.to_string())?;
    let mut project = storage.get_project(&project_id).map_err(|e| e.to_string())?;

    // Tauri project: sync port from tauri.conf.json devUrl
    if matches!(project.project_type, ProjectType::Tauri) {
        let project_path = std::path::Path::new(&project.path);
        if let Some(actual_port) = ProjectDetector::read_tauri_port(project_path) {
            if project.port != actual_port {
                project.port = actual_port;
                project.updated_at = chrono::Utc::now().to_rfc3339();
                let _ = storage.update_project(project.clone());
            }
        }
    }

    let mut app_state = state.lock().await;
    let mut process_manager = ProcessManager::new();

    let process_info = process_manager
        .start_project(&project, app_handle)
        .map_err(|e| e.to_string())?;

    app_state
        .running_processes
        .insert(project_id, process_info.clone());

    Ok(process_info)
}

#[tauri::command]
pub async fn stop_project(
    project_id: String,
    state: State<'_, Arc<Mutex<AppState>>>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let mut app_state = state.lock().await;

    if let Some(process_info) = app_state.running_processes.remove(&project_id) {
        // Kill the process tree using centralized function
        kill_process_tree_silent(process_info.pid);

        // Emit process stopped event
        let _ = app_handle.emit(
            "process-stopped",
            serde_json::json!({
                "projectId": project_id
            }),
        );

        Ok(())
    } else {
        Err(format!("Process not found for project: {}", project_id))
    }
}

#[tauri::command]
pub async fn restart_project(
    project_id: String,
    state: State<'_, Arc<Mutex<AppState>>>,
    app_handle: AppHandle,
) -> Result<ProcessInfo, String> {
    // Stop the project first
    let _ = stop_project(project_id.clone(), state.clone(), app_handle.clone()).await;

    // Wait a bit for the process to fully stop
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Start the project again
    start_project(project_id, state, app_handle).await
}

#[tauri::command]
pub async fn get_running_processes(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<ProcessInfo>, String> {
    let app_state = state.lock().await;
    Ok(app_state.running_processes.values().cloned().collect())
}
