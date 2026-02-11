use crate::models::{ConfigFile, Service};
use crate::services::ServiceManager;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigFileInfo {
    pub name: String,
    pub path: String,
    pub description: String,
}

impl From<&ConfigFile> for ConfigFileInfo {
    fn from(cf: &ConfigFile) -> Self {
        Self {
            name: cf.name.clone(),
            path: cf.path.clone(),
            description: cf.description.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceInfo {
    pub id: String,
    pub name: String,
    pub service_type: String,
    pub port: u16,
    pub status: String,
    pub pid: Option<u32>,
    pub auto_start: bool,
    pub auto_restart: bool,
    pub last_started: Option<String>,
    pub last_stopped: Option<String>,
    pub error_message: Option<String>,
    pub installed: bool,
    pub config_files: Vec<ConfigFileInfo>,
}

impl From<&Service> for ServiceInfo {
    fn from(service: &Service) -> Self {
        Self {
            id: service.id.clone(),
            name: service.name.clone(),
            service_type: format!("{:?}", service.service_type).to_lowercase(),
            port: service.port,
            status: format!("{:?}", service.status).to_lowercase(),
            pid: service.pid,
            auto_start: service.auto_start,
            auto_restart: service.auto_restart,
            last_started: service.last_started.clone(),
            last_stopped: service.last_stopped.clone(),
            error_message: service.error_message.clone(),
            installed: service.installed,
            config_files: service.config_files.iter().map(ConfigFileInfo::from).collect(),
        }
    }
}

#[tauri::command]
pub async fn get_services(
    service_manager: State<'_, Arc<Mutex<ServiceManager>>>,
) -> Result<Vec<ServiceInfo>, String> {
    let manager = service_manager.lock().await;
    let services: Vec<ServiceInfo> = manager.get_services().iter().map(|s| ServiceInfo::from(*s)).collect();
    Ok(services)
}

#[tauri::command]
pub async fn get_service(
    service_manager: State<'_, Arc<Mutex<ServiceManager>>>,
    id: String,
) -> Result<ServiceInfo, String> {
    let manager = service_manager.lock().await;
    manager
        .get_service(&id)
        .map(|s| ServiceInfo::from(s))
        .ok_or_else(|| "Service not found".to_string())
}

#[tauri::command]
pub async fn start_service(
    app: AppHandle,
    service_manager: State<'_, Arc<Mutex<ServiceManager>>>,
    id: String,
) -> Result<ServiceInfo, String> {
    let mut manager = service_manager.lock().await;
    manager.start_service(&id, Some(app)).await?;
    manager
        .get_service(&id)
        .map(|s| ServiceInfo::from(s))
        .ok_or_else(|| "Service not found".to_string())
}

#[tauri::command]
pub async fn stop_service(
    service_manager: State<'_, Arc<Mutex<ServiceManager>>>,
    id: String,
) -> Result<ServiceInfo, String> {
    let mut manager = service_manager.lock().await;
    manager.stop_service(&id).await?;
    manager
        .get_service(&id)
        .map(|s| ServiceInfo::from(s))
        .ok_or_else(|| "Service not found".to_string())
}

#[tauri::command]
pub async fn restart_service(
    app: AppHandle,
    service_manager: State<'_, Arc<Mutex<ServiceManager>>>,
    id: String,
) -> Result<ServiceInfo, String> {
    let mut manager = service_manager.lock().await;
    manager.restart_service(&id, Some(app)).await?;
    manager
        .get_service(&id)
        .map(|s| ServiceInfo::from(s))
        .ok_or_else(|| "Service not found".to_string())
}

#[tauri::command]
pub async fn check_service_health(
    service_manager: State<'_, Arc<Mutex<ServiceManager>>>,
    id: String,
) -> Result<String, String> {
    let mut manager = service_manager.lock().await;
    let status = manager.check_health(&id).await;
    Ok(format!("{:?}", status).to_lowercase())
}

#[tauri::command]
pub async fn get_all_service_statuses(
    service_manager: State<'_, Arc<Mutex<ServiceManager>>>,
) -> Result<std::collections::HashMap<String, String>, String> {
    let manager = service_manager.lock().await;
    let statuses = manager
        .get_all_statuses()
        .into_iter()
        .map(|(id, status)| (id, format!("{:?}", status).to_lowercase()))
        .collect();
    Ok(statuses)
}

#[tauri::command]
pub async fn set_service_auto_start(
    service_manager: State<'_, Arc<Mutex<ServiceManager>>>,
    id: String,
    auto_start: bool,
) -> Result<ServiceInfo, String> {
    let mut manager = service_manager.lock().await;
    if let Some(service) = manager.get_service_mut(&id) {
        service.auto_start = auto_start;
        Ok(ServiceInfo::from(&service.clone()))
    } else {
        Err("Service not found".to_string())
    }
}

#[tauri::command]
pub async fn set_service_auto_restart(
    service_manager: State<'_, Arc<Mutex<ServiceManager>>>,
    id: String,
    auto_restart: bool,
) -> Result<ServiceInfo, String> {
    let mut manager = service_manager.lock().await;
    if let Some(service) = manager.get_service_mut(&id) {
        service.auto_restart = auto_restart;
        Ok(ServiceInfo::from(&service.clone()))
    } else {
        Err("Service not found".to_string())
    }
}

#[tauri::command]
pub async fn get_service_config(
    service_manager: State<'_, Arc<Mutex<ServiceManager>>>,
    id: String,
    config_name: String,
) -> Result<String, String> {
    let manager = service_manager.lock().await;
    let service = manager
        .get_service(&id)
        .ok_or_else(|| "Service not found".to_string())?;

    let config_file = service
        .config_files
        .iter()
        .find(|cf| cf.name == config_name)
        .ok_or_else(|| format!("Config file '{}' not found", config_name))?;

    fs::read_to_string(&config_file.path)
        .map_err(|e| format!("Failed to read config file: {}", e))
}

#[tauri::command]
pub async fn save_service_config(
    service_manager: State<'_, Arc<Mutex<ServiceManager>>>,
    id: String,
    config_name: String,
    content: String,
) -> Result<(), String> {
    let manager = service_manager.lock().await;
    let service = manager
        .get_service(&id)
        .ok_or_else(|| "Service not found".to_string())?;

    let config_file = service
        .config_files
        .iter()
        .find(|cf| cf.name == config_name)
        .ok_or_else(|| format!("Config file '{}' not found", config_name))?;

    fs::write(&config_file.path, content)
        .map_err(|e| format!("Failed to write config file: {}", e))
}

#[tauri::command]
pub async fn get_service_config_list(
    service_manager: State<'_, Arc<Mutex<ServiceManager>>>,
    id: String,
) -> Result<Vec<ConfigFileInfo>, String> {
    let manager = service_manager.lock().await;
    let service = manager
        .get_service(&id)
        .ok_or_else(|| "Service not found".to_string())?;

    Ok(service.config_files.iter().map(ConfigFileInfo::from).collect())
}
