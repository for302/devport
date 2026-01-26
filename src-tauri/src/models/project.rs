use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    Tauri,     // Desktop app with Tauri (highest priority)
    Electron,  // Desktop app with Electron
    NextJs,
    Vite,
    React,
    Vue,
    Angular,
    Svelte,
    Python,
    Django,
    Flask,
    FastApi,
    Node,
    Express,
    Unknown,
}

impl Default for ProjectType {
    fn default() -> Self {
        ProjectType::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    pub port: u16,
    pub project_type: ProjectType,
    pub start_command: String,
    pub env_vars: std::collections::HashMap<String, String>,
    pub auto_start: bool,
    pub health_check_url: Option<String>,
    #[serde(default)]
    pub domain: Option<String>,  // Custom domain for hosts file (e.g., "my-app.test")
    pub created_at: String,
    pub updated_at: String,
}

impl Project {
    pub fn new(
        name: String,
        path: String,
        port: u16,
        project_type: ProjectType,
        start_command: String,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            path,
            port,
            project_type,
            start_command,
            env_vars: std::collections::HashMap::new(),
            auto_start: false,
            health_check_url: None,
            domain: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}
