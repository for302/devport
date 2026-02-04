use crate::models::{Project, ProjectType};
use crate::services::hosts_manager::HostsManager;
use crate::services::project_detector::ProjectDetector;
use crate::services::storage::Storage;
use crate::services::SharedProjectWatcher;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectInput {
    pub name: String,
    pub path: String,
    pub port: u16,
    pub project_type: ProjectType,
    pub start_command: String,
    #[serde(default)]
    pub auto_start: bool,
    pub health_check_url: Option<String>,
    pub domain: Option<String>,
    pub github_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectInput {
    pub id: String,
    pub name: Option<String>,
    pub port: Option<u16>,
    pub start_command: Option<String>,
    pub auto_start: Option<bool>,
    pub health_check_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedProjectInfo {
    pub project_type: ProjectType,
    pub name: String,
    pub start_command: String,
    pub default_port: u16,
    pub venv_path: Option<String>,
    pub github_url: Option<String>,
}

#[tauri::command]
pub fn get_projects() -> Result<Vec<Project>, String> {
    let storage = Storage::new().map_err(|e| e.to_string())?;
    let mut projects = storage.load_projects().map_err(|e| e.to_string())?;

    // Auto-detect GitHub URLs for projects that don't have one
    let mut updated = false;
    for project in &mut projects {
        if project.github_url.is_none() {
            let path = std::path::Path::new(&project.path);
            if let Some(github_url) = ProjectDetector::detect_github_url(path) {
                project.github_url = Some(github_url);
                updated = true;
            }
        }
    }

    // Save updated projects if any GitHub URLs were detected
    if updated {
        for project in &projects {
            if project.github_url.is_some() {
                let _ = storage.update_project(project.clone());
            }
        }
    }

    Ok(projects)
}

#[tauri::command]
pub fn create_project(
    input: CreateProjectInput,
    project_watcher: State<'_, SharedProjectWatcher>,
) -> Result<Project, String> {
    let storage = Storage::new().map_err(|e| e.to_string())?;

    let project = Project::new(
        input.name.clone(),
        input.path,
        input.port,
        input.project_type,
        input.start_command,
    );

    let mut project = project;
    project.auto_start = input.auto_start;
    project.health_check_url = input.health_check_url;
    project.domain = input.domain.clone();
    project.github_url = input.github_url;

    // Add hosts entry if domain is provided
    if let Some(ref domain) = input.domain {
        if !domain.is_empty() {
            // Validate domain before adding
            HostsManager::validate_domain(domain)?;

            let hosts_manager = HostsManager::new();
            let comment = format!("DevPort: {}", input.name);
            if let Err(e) = hosts_manager.add_entry(domain, "127.0.0.1", Some(&comment)) {
                eprintln!("Failed to add hosts entry: {}", e);
            }
        }
    }

    let created_project = storage.create_project(project).map_err(|e| e.to_string())?;

    // Start watching the new project
    if let Ok(mut watcher) = project_watcher.lock() {
        let _ = watcher.watch_project(&created_project);
    }

    Ok(created_project)
}

#[tauri::command]
pub fn update_project(input: UpdateProjectInput) -> Result<Project, String> {
    let storage = Storage::new().map_err(|e| e.to_string())?;

    let mut project = storage.get_project(&input.id).map_err(|e| e.to_string())?;

    if let Some(name) = input.name {
        project.name = name;
    }
    if let Some(port) = input.port {
        project.port = port;
    }
    if let Some(start_command) = input.start_command {
        project.start_command = start_command;
    }
    if let Some(auto_start) = input.auto_start {
        project.auto_start = auto_start;
    }
    if let Some(health_check_url) = input.health_check_url {
        project.health_check_url = Some(health_check_url);
    }

    project.updated_at = chrono::Utc::now().to_rfc3339();

    storage.update_project(project).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_project(
    id: String,
    project_watcher: State<'_, SharedProjectWatcher>,
) -> Result<(), String> {
    let storage = Storage::new().map_err(|e| e.to_string())?;

    // Get project before deleting to retrieve domain
    let project = storage.get_project(&id).map_err(|e| e.to_string())?;

    // Remove hosts entry if domain exists
    if let Some(ref domain) = project.domain {
        if !domain.is_empty() {
            let hosts_manager = HostsManager::new();
            if let Err(e) = hosts_manager.remove_entry(domain) {
                eprintln!("Failed to remove hosts entry for {}: {}", domain, e);
            }
        }
    }

    // Stop watching the project
    if let Ok(mut watcher) = project_watcher.lock() {
        watcher.unwatch_project(&id);
    }

    storage.delete_project(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn detect_project_type(path: String) -> Result<DetectedProjectInfo, String> {
    let detected = ProjectDetector::detect(&path).map_err(|e| e.to_string())?;

    Ok(DetectedProjectInfo {
        project_type: detected.project_type,
        name: detected.name,
        start_command: detected.start_command,
        default_port: detected.default_port,
        venv_path: detected.venv_path,
        github_url: detected.github_url,
    })
}
