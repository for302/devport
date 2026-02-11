use crate::models::{Project, ProjectType};
use crate::services::database_manager::DatabaseManager;
use crate::services::hosts_manager::HostsManager;
use crate::services::project_detector::ProjectDetector;
use crate::services::storage::Storage;
use crate::services::SharedProjectWatcher;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

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
    #[serde(default = "default_launch_mode")]
    pub launch_mode: String,
    #[serde(default)]
    pub create_database: bool,
    pub database_name: Option<String>,
}

fn default_launch_mode() -> String {
    "web".to_string()
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
    pub launch_mode: Option<String>,
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
pub async fn create_project(
    input: CreateProjectInput,
    project_watcher: State<'_, SharedProjectWatcher>,
    database_manager: State<'_, Arc<Mutex<DatabaseManager>>>,
) -> Result<Project, String> {
    let storage = Storage::new().map_err(|e| e.to_string())?;

    let project_path = if input.create_database { Some(input.path.clone()) } else { None };
    let project_type_clone = input.project_type.clone();

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
    project.launch_mode = input.launch_mode;

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

    // Create database if requested
    if input.create_database {
        if let Some(ref db_name) = input.database_name {
            if !db_name.is_empty() {
                let db_manager = database_manager.lock().await;
                let db_username = db_name.clone();
                let db_password = generate_db_password();

                match db_manager.create_database_with_user(db_name, &db_username, &db_password) {
                    Ok(creds) => {
                        // Inject DB environment variables into .env file
                        if let Some(ref proj_path) = project_path {
                            let _ = inject_db_env_vars(
                                proj_path,
                                &project_type_clone,
                                &creds,
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to create database: {}", e);
                        // Don't fail project creation if DB creation fails
                    }
                }
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

/// Generate a random password for database user
fn generate_db_password() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("dp_{:x}", seed & 0xFFFFFFFFFFFF)
}

/// Inject database environment variables into the project's .env file
fn inject_db_env_vars(
    project_path: &str,
    project_type: &ProjectType,
    creds: &crate::services::database_manager::DatabaseCredentials,
) -> Result<(), String> {
    let path = std::path::Path::new(project_path);
    let env_path = path.join(".env");

    let connection_url = format!(
        "mysql://{}:{}@{}:{}/{}",
        creds.username, creds.password, creds.host, creds.port, creds.database
    );

    // Build env vars based on framework type
    let mut new_vars: Vec<(String, String)> = Vec::new();

    match project_type {
        ProjectType::Django | ProjectType::Flask | ProjectType::FastApi
        | ProjectType::Python | ProjectType::NextJs | ProjectType::Node
        | ProjectType::Express | ProjectType::Vite | ProjectType::React => {
            new_vars.push(("DATABASE_URL".to_string(), connection_url));
        }
        _ => {
            // Generic: provide both DATABASE_URL and individual vars
            new_vars.push(("DATABASE_URL".to_string(), connection_url));
            new_vars.push(("DB_HOST".to_string(), creds.host.clone()));
            new_vars.push(("DB_PORT".to_string(), creds.port.to_string()));
            new_vars.push(("DB_DATABASE".to_string(), creds.database.clone()));
            new_vars.push(("DB_USERNAME".to_string(), creds.username.clone()));
            new_vars.push(("DB_PASSWORD".to_string(), creds.password.clone()));
        }
    }

    // Read existing .env or create new
    let mut content = if env_path.exists() {
        std::fs::read_to_string(&env_path).unwrap_or_default()
    } else {
        String::new()
    };

    // Append new vars
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str("\n# Database (auto-generated by DevPort)\n");
    for (key, value) in &new_vars {
        content.push_str(&format!("{}={}\n", key, value));
    }

    std::fs::write(&env_path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_project(input: UpdateProjectInput) -> Result<Project, String> {
    let storage = Storage::new().map_err(|e| e.to_string())?;

    let mut project = storage.get_project(&input.id).map_err(|e| e.to_string())?;

    if let Some(name) = input.name {
        project.name = name;
    }
    if let Some(port) = input.port {
        // Tauri: sync port to config files (tauri.conf.json + vite.config.ts)
        if matches!(project.project_type, ProjectType::Tauri) && port != project.port {
            let project_path = std::path::Path::new(&project.path);
            ProjectDetector::update_tauri_port(project_path, port)
                .map_err(|e| format!("Failed to update port config: {}", e))?;
        }
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
    if let Some(launch_mode) = input.launch_mode {
        project.launch_mode = launch_mode;
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
