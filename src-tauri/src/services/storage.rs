use crate::models::Project;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Project not found: {0}")]
    NotFound(String),
}

pub struct Storage {
    data_dir: PathBuf,
}

impl Storage {
    pub fn new() -> Result<Self, StorageError> {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("clickdevport");

        fs::create_dir_all(&data_dir)?;

        Ok(Self { data_dir })
    }

    fn projects_file(&self) -> PathBuf {
        self.data_dir.join("projects.json")
    }

    pub fn load_projects(&self) -> Result<Vec<Project>, StorageError> {
        let path = self.projects_file();
        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&path)?;
        let projects: Vec<Project> = serde_json::from_str(&content)?;
        Ok(projects)
    }

    pub fn save_projects(&self, projects: &[Project]) -> Result<(), StorageError> {
        let path = self.projects_file();
        let content = serde_json::to_string_pretty(projects)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn get_project(&self, id: &str) -> Result<Project, StorageError> {
        let projects = self.load_projects()?;
        projects
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| StorageError::NotFound(id.to_string()))
    }

    pub fn create_project(&self, project: Project) -> Result<Project, StorageError> {
        let mut projects = self.load_projects()?;
        projects.push(project.clone());
        self.save_projects(&projects)?;
        Ok(project)
    }

    pub fn update_project(&self, project: Project) -> Result<Project, StorageError> {
        let mut projects = self.load_projects()?;
        let index = projects
            .iter()
            .position(|p| p.id == project.id)
            .ok_or_else(|| StorageError::NotFound(project.id.clone()))?;

        projects[index] = project.clone();
        self.save_projects(&projects)?;
        Ok(project)
    }

    pub fn delete_project(&self, id: &str) -> Result<(), StorageError> {
        let mut projects = self.load_projects()?;
        let index = projects
            .iter()
            .position(|p| p.id == id)
            .ok_or_else(|| StorageError::NotFound(id.to_string()))?;

        projects.remove(index);
        self.save_projects(&projects)?;
        Ok(())
    }

    pub fn logs_dir(&self) -> PathBuf {
        let dir = self.data_dir.join("logs");
        let _ = fs::create_dir_all(&dir);
        dir
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new().expect("Failed to initialize storage")
    }
}
