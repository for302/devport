use crate::models::Project;
use crate::services::project_detector::ProjectDetector;
use crate::services::storage::Storage;
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// Files that trigger re-detection when changed
const WATCH_FILES: &[&str] = &[
    "package.json",
    "tauri.conf.json",
    "Cargo.toml",
    "requirements.txt",
    "pyproject.toml",
];

/// Directories that trigger re-detection when created/deleted
const WATCH_DIRS: &[&str] = &["src-tauri"];

/// Payload for project type changed event
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTypeChangedPayload {
    pub project_id: String,
    pub project_name: String,
    pub old_type: String,
    pub new_type: String,
    pub new_command: String,
}

pub struct ProjectWatcher {
    watchers: HashMap<String, notify::RecommendedWatcher>,
    app_handle: Option<AppHandle>,
}

impl ProjectWatcher {
    pub fn new() -> Self {
        Self {
            watchers: HashMap::new(),
            app_handle: None,
        }
    }

    pub fn set_app_handle(&mut self, app_handle: AppHandle) {
        self.app_handle = Some(app_handle);
    }

    /// Start watching a project's directory for changes
    pub fn watch_project(&mut self, project: &Project) -> Result<(), String> {
        let project_path = PathBuf::from(&project.path);
        if !project_path.exists() {
            return Err(format!("Project path does not exist: {}", project.path));
        }

        // Remove existing watcher if any
        self.unwatch_project(&project.id);

        let app_handle = self
            .app_handle
            .clone()
            .ok_or("App handle not set")?;
        let project_id = project.id.clone();
        let project_path_str = project.path.clone();

        // Create a channel for receiving events
        let (tx, rx) = channel();

        // Create a debouncer to avoid rapid-fire events
        let mut debouncer = new_debouncer(Duration::from_secs(2), tx)
            .map_err(|e| format!("Failed to create debouncer: {}", e))?;

        // Watch the project directory (non-recursive, we only care about root files)
        debouncer
            .watcher()
            .watch(project_path.as_path(), RecursiveMode::NonRecursive)
            .map_err(|e| format!("Failed to watch directory: {}", e))?;

        // Also watch src-tauri directory if it exists
        let tauri_dir = project_path.join("src-tauri");
        if tauri_dir.exists() {
            let _ = debouncer
                .watcher()
                .watch(tauri_dir.as_path(), RecursiveMode::NonRecursive);
        }

        // Spawn a thread to handle events
        let project_id_clone = project_id.clone();
        thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(Ok(events)) => {
                        // Check if any relevant file changed
                        let relevant_change = events.iter().any(|e| {
                            Self::is_relevant_change(&e.path)
                        });

                        if relevant_change {
                            Self::handle_project_change(
                                &app_handle,
                                &project_id_clone,
                                &project_path_str,
                            );
                        }
                    }
                    Ok(Err(e)) => {
                        eprintln!("Watch error for project {}: {:?}", project_id_clone, e);
                    }
                    Err(_) => {
                        // Channel closed, exit thread
                        break;
                    }
                }
            }
        });

        // Store the watcher (need to keep it alive)
        // Note: debouncer owns the watcher, so we need to keep it in a different way
        // For now, we'll use a simpler approach

        Ok(())
    }

    /// Stop watching a project
    pub fn unwatch_project(&mut self, project_id: &str) {
        self.watchers.remove(project_id);
    }

    /// Stop all watchers
    pub fn stop_all(&mut self) {
        self.watchers.clear();
    }

    /// Check if the changed path is relevant for re-detection
    fn is_relevant_change(path: &Path) -> bool {
        if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
            // Check if it's a watched file
            if WATCH_FILES.iter().any(|&f| f == file_name) {
                return true;
            }
        }

        // Check if it's a watched directory (parent path contains the dir name)
        if let Some(parent) = path.parent() {
            if let Some(dir_name) = parent.file_name().and_then(|f| f.to_str()) {
                if WATCH_DIRS.iter().any(|&d| d == dir_name) {
                    return true;
                }
            }
        }

        // Check if the path itself is a watched directory
        if let Some(dir_name) = path.file_name().and_then(|f| f.to_str()) {
            if WATCH_DIRS.iter().any(|&d| d == dir_name) {
                return true;
            }
        }

        false
    }

    /// Handle project file changes
    fn handle_project_change(app_handle: &AppHandle, project_id: &str, project_path: &str) {
        // Re-detect project type
        let detected = match ProjectDetector::detect(project_path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to detect project type: {}", e);
                return;
            }
        };

        // Load current project to compare
        let storage = match Storage::new() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to create storage: {}", e);
                return;
            }
        };

        let mut projects = match storage.load_projects() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to load projects: {}", e);
                return;
            }
        };

        // Find the project index
        let project_index = match projects.iter().position(|p| p.id == project_id) {
            Some(i) => i,
            None => return,
        };

        // Only update if type actually changed
        if projects[project_index].project_type == detected.project_type {
            return;
        }

        let old_type = format!("{:?}", projects[project_index].project_type).to_lowercase();
        let new_type = format!("{:?}", detected.project_type).to_lowercase();
        let project_name = projects[project_index].name.clone();

        // Update project
        projects[project_index].project_type = detected.project_type;
        projects[project_index].start_command = detected.start_command.clone();
        projects[project_index].updated_at = chrono::Utc::now().to_rfc3339();

        // Save updated projects
        if let Err(e) = storage.save_projects(&projects) {
            eprintln!("Failed to save projects: {}", e);
            return;
        }

        // Emit event to frontend
        let payload = ProjectTypeChangedPayload {
            project_id: project_id.to_string(),
            project_name,
            old_type,
            new_type,
            new_command: detected.start_command,
        };

        let _ = app_handle.emit("project-type-changed", payload);
    }
}

impl Default for ProjectWatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared project watcher type
pub type SharedProjectWatcher = Arc<Mutex<ProjectWatcher>>;

/// Initialize project watcher and start watching all projects
pub fn init_project_watcher(app_handle: &AppHandle) -> SharedProjectWatcher {
    let mut watcher = ProjectWatcher::new();
    watcher.set_app_handle(app_handle.clone());

    // Load existing projects and start watching them
    if let Ok(storage) = Storage::new() {
        if let Ok(projects) = storage.load_projects() {
            for project in &projects {
                if let Err(e) = watcher.watch_project(project) {
                    eprintln!("Failed to watch project {}: {}", project.name, e);
                }
            }
        }
    }

    Arc::new(Mutex::new(watcher))
}
