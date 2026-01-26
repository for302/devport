use crate::services::log_manager::{LogEntry, LogManager, LogUpdatePayload, SharedLogStreamManager};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogReadResult {
    pub entries: Vec<LogEntry>,
    pub total_size: u64,
    pub path: String,
}

#[tauri::command]
pub async fn get_service_logs(
    log_manager: State<'_, Arc<Mutex<LogManager>>>,
    service_id: String,
    log_type: String,
    lines: Option<usize>,
) -> Result<LogReadResult, String> {
    let manager = log_manager.lock().await;
    let path = manager.get_log_path(&service_id, &log_type);
    let num_lines = lines.unwrap_or(100);

    let entries = manager
        .read_log_entries(&path, num_lines)
        .map_err(|e| e.to_string())?;

    let total_size = manager.get_log_size(&path).unwrap_or(0);

    Ok(LogReadResult {
        entries,
        total_size,
        path: path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub async fn get_project_logs(
    log_manager: State<'_, Arc<Mutex<LogManager>>>,
    project_name: String,
    log_type: String,
    lines: Option<usize>,
) -> Result<LogReadResult, String> {
    let manager = log_manager.lock().await;
    let path = manager.get_project_log_path(&project_name, &log_type);
    let num_lines = lines.unwrap_or(100);

    let entries = manager
        .read_log_entries(&path, num_lines)
        .map_err(|e| e.to_string())?;

    let total_size = manager.get_log_size(&path).unwrap_or(0);

    Ok(LogReadResult {
        entries,
        total_size,
        path: path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub async fn clear_service_logs(
    log_manager: State<'_, Arc<Mutex<LogManager>>>,
    service_id: String,
    log_type: String,
) -> Result<(), String> {
    let manager = log_manager.lock().await;
    let path = manager.get_log_path(&service_id, &log_type);
    manager.clear_log(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_project_logs(
    log_manager: State<'_, Arc<Mutex<LogManager>>>,
    project_name: String,
    log_type: String,
) -> Result<(), String> {
    let manager = log_manager.lock().await;
    let path = manager.get_project_log_path(&project_name, &log_type);
    manager.clear_log(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cleanup_old_logs(
    log_manager: State<'_, Arc<Mutex<LogManager>>>,
) -> Result<(), String> {
    let manager = log_manager.lock().await;
    manager.cleanup_old_logs().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_log_file_path(
    log_manager: State<'_, Arc<Mutex<LogManager>>>,
    service_id: Option<String>,
    project_name: Option<String>,
    log_type: String,
) -> Result<String, String> {
    let manager = log_manager.lock().await;

    let path = if let Some(sid) = service_id {
        manager.get_log_path(&sid, &log_type)
    } else if let Some(pname) = project_name {
        manager.get_project_log_path(&pname, &log_type)
    } else {
        return Err("Either service_id or project_name must be provided".to_string());
    };

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn write_log_entry(
    log_manager: State<'_, Arc<Mutex<LogManager>>>,
    service_id: Option<String>,
    project_name: Option<String>,
    log_type: String,
    level: String,
    message: String,
    source: String,
) -> Result<(), String> {
    let manager = log_manager.lock().await;

    let path = if let Some(sid) = service_id {
        manager.get_log_path(&sid, &log_type)
    } else if let Some(pname) = project_name {
        manager.get_project_log_path(&pname, &log_type)
    } else {
        return Err("Either service_id or project_name must be provided".to_string());
    };

    let entry = LogEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        level,
        message,
        source,
    };

    manager
        .write_log_entry(&path, &entry)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_log_stream(
    app: AppHandle,
    stream_manager: State<'_, SharedLogStreamManager>,
    service_id: Option<String>,
    project_name: Option<String>,
    log_type: String,
) -> Result<bool, String> {
    let mut manager = stream_manager.write().await;

    // Generate source ID for tracking
    let source_id = if let Some(sid) = &service_id {
        format!("service:{}:{}", sid, log_type)
    } else if let Some(pname) = &project_name {
        format!("project:{}:{}", pname, log_type)
    } else {
        return Err("Either service_id or project_name must be provided".to_string());
    };

    // Get log path
    let log_path = if let Some(sid) = &service_id {
        manager.get_service_log_path(sid, &log_type)
    } else if let Some(pname) = &project_name {
        manager.get_project_log_path(pname, &log_type)
    } else {
        return Err("Either service_id or project_name must be provided".to_string());
    };

    // Source for the event payload
    let event_source = service_id.or(project_name).unwrap();
    let event_source_clone = event_source.clone();

    // Start the stream with callback to emit events
    let started = manager.start_stream(source_id, log_path, move |entries| {
        let payload = LogUpdatePayload {
            source: event_source_clone.clone(),
            entries,
        };
        let _ = app.emit("log-update", payload);
    });

    Ok(started)
}

#[tauri::command]
pub async fn stop_log_stream(
    stream_manager: State<'_, SharedLogStreamManager>,
    service_id: Option<String>,
    project_name: Option<String>,
    log_type: String,
) -> Result<bool, String> {
    let mut manager = stream_manager.write().await;

    // Generate source ID
    let source_id = if let Some(sid) = &service_id {
        format!("service:{}:{}", sid, log_type)
    } else if let Some(pname) = &project_name {
        format!("project:{}:{}", pname, log_type)
    } else {
        return Err("Either service_id or project_name must be provided".to_string());
    };

    let stopped = manager.stop_stream(&source_id);
    Ok(stopped)
}
