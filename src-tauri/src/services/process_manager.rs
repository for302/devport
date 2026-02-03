use crate::models::process_info::ProcessInfo;
use crate::models::{Project, ProjectType};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::thread;
use tauri::{AppHandle, Emitter};
use thiserror::Error;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("Failed to start process: {0}")]
    StartError(String),
    #[error("Failed to stop process: {0}")]
    StopError(String),
    #[error("Process not found: {0}")]
    NotFound(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub struct ProcessManager {
    processes: HashMap<String, Child>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
        }
    }

    /// Build the command with port option based on project type
    fn build_command_with_port(start_command: &str, project_type: &ProjectType, port: u16) -> String {
        let port_str = port.to_string();

        // Check if port is already specified in the command
        if start_command.contains("--port") || start_command.contains("-p ") {
            return start_command.to_string();
        }

        match project_type {
            // Vite-based projects: npm run dev -- --port {port}
            ProjectType::Vite | ProjectType::React | ProjectType::Vue | ProjectType::Svelte => {
                if start_command.contains("npm run") || start_command.contains("pnpm run") || start_command.contains("yarn ") {
                    format!("{} -- --port {}", start_command, port_str)
                } else if start_command.contains("vite") {
                    format!("{} --port {}", start_command, port_str)
                } else {
                    start_command.to_string()
                }
            }
            // Next.js: npm run dev -- -p {port}
            ProjectType::NextJs => {
                if start_command.contains("npm run") || start_command.contains("pnpm run") || start_command.contains("yarn ") {
                    format!("{} -- -p {}", start_command, port_str)
                } else if start_command.contains("next") {
                    format!("{} -p {}", start_command, port_str)
                } else {
                    start_command.to_string()
                }
            }
            // Angular: ng serve --port {port}
            ProjectType::Angular => {
                if start_command.contains("ng serve") {
                    format!("{} --port {}", start_command, port_str)
                } else if start_command.contains("npm") {
                    format!("{} -- --port {}", start_command, port_str)
                } else {
                    start_command.to_string()
                }
            }
            // Django: python manage.py runserver 0.0.0.0:{port}
            ProjectType::Django => {
                if start_command.contains("runserver") && !start_command.contains(':') {
                    format!("{} 0.0.0.0:{}", start_command, port_str)
                } else {
                    start_command.to_string()
                }
            }
            // Flask: flask run --port {port}
            ProjectType::Flask => {
                if start_command.contains("flask run") {
                    format!("{} --port {}", start_command, port_str)
                } else {
                    start_command.to_string()
                }
            }
            // FastAPI/Uvicorn: uvicorn ... --port {port}
            ProjectType::FastApi => {
                if start_command.contains("uvicorn") {
                    format!("{} --port {}", start_command, port_str)
                } else {
                    start_command.to_string()
                }
            }
            // Tauri: Port is configured in tauri.conf.json, no CLI injection needed
            // The internal dev server (Vite) uses the PORT env var
            ProjectType::Tauri => {
                start_command.to_string()
            }
            // Electron: Uses PORT env variable (handled separately)
            ProjectType::Electron => {
                start_command.to_string()
            }
            // Python Desktop apps: No port needed
            ProjectType::PythonTkinter | ProjectType::PythonPyQt | ProjectType::PythonWx
            | ProjectType::PythonPygame | ProjectType::PythonKivy => {
                start_command.to_string()
            }
            // Node/Express: Uses PORT env variable (handled separately)
            ProjectType::Node | ProjectType::Express | ProjectType::Python | ProjectType::Unknown => {
                start_command.to_string()
            }
        }
    }

    pub fn start_project(
        &mut self,
        project: &Project,
        app_handle: AppHandle,
    ) -> Result<ProcessInfo, ProcessError> {
        let project_id = project.id.clone();

        // Build environment variables
        let mut env_vars = project.env_vars.clone();
        // Only inject PORT for projects that use it (port > 0)
        if project.port > 0 {
            env_vars.insert("PORT".to_string(), project.port.to_string());
        }
        // Flask-specific env var
        if matches!(project.project_type, ProjectType::Flask) {
            env_vars.insert("FLASK_RUN_PORT".to_string(), project.port.to_string());
        }

        // Set up venv environment variables if venv_path is present in env_vars
        if let Some(venv_rel) = project.env_vars.get("DEVPORT_VENV_PATH") {
            let project_path = std::path::Path::new(&project.path);
            let venv_abs = project_path.join(venv_rel);
            env_vars.insert("VIRTUAL_ENV".to_string(), venv_abs.to_string_lossy().to_string());

            // Prepend venv bin/Scripts to PATH
            #[cfg(windows)]
            let venv_bin = venv_abs.join("Scripts");
            #[cfg(not(windows))]
            let venv_bin = venv_abs.join("bin");

            if let Ok(current_path) = std::env::var("PATH") {
                env_vars.insert(
                    "PATH".to_string(),
                    format!("{};{}", venv_bin.to_string_lossy(), current_path),
                );
            }
        }

        // Build command with port option
        let command_with_port = Self::build_command_with_port(
            &project.start_command,
            &project.project_type,
            project.port,
        );

        // Start process - on Windows, run through cmd.exe to handle .cmd scripts
        #[cfg(target_os = "windows")]
        let mut child = Command::new("cmd")
            .args(["/C", &command_with_port])
            .current_dir(&project.path)
            .envs(&env_vars)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ProcessError::StartError(e.to_string()))?;

        #[cfg(not(target_os = "windows"))]
        let mut child = {
            let parts: Vec<&str> = command_with_port.split_whitespace().collect();
            if parts.is_empty() {
                return Err(ProcessError::StartError("Empty command".to_string()));
            }
            Command::new(parts[0])
                .args(&parts[1..])
                .current_dir(&project.path)
                .envs(&env_vars)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| ProcessError::StartError(e.to_string()))?
        };

        let pid = child.id();

        // Set up log streaming
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let project_id_clone = project_id.clone();
        let app_handle_clone = app_handle.clone();

        if let Some(stdout) = stdout {
            let project_id = project_id_clone.clone();
            let app = app_handle_clone.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    let _ = app.emit(
                        "process-log",
                        serde_json::json!({
                            "projectId": project_id,
                            "line": line,
                            "stream": "stdout"
                        }),
                    );
                }
            });
        }

        if let Some(stderr) = stderr {
            let project_id = project_id_clone.clone();
            let app = app_handle_clone.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    let _ = app.emit(
                        "process-log",
                        serde_json::json!({
                            "projectId": project_id,
                            "line": line,
                            "stream": "stderr"
                        }),
                    );
                }
            });
        }

        self.processes.insert(project_id.clone(), child);

        // Emit process started event
        let _ = app_handle.emit(
            "process-started",
            serde_json::json!({
                "projectId": project_id,
                "pid": pid
            }),
        );

        Ok(ProcessInfo::new(project_id, pid, project.port))
    }

    pub fn stop_project(
        &mut self,
        project_id: &str,
        app_handle: AppHandle,
    ) -> Result<(), ProcessError> {
        if let Some(mut child) = self.processes.remove(project_id) {
            let pid = child.id();

            // Kill the process tree on Windows
            #[cfg(windows)]
            let _ = Command::new("taskkill")
                .args(["/F", "/T", "/PID", &pid.to_string()])
                .creation_flags(CREATE_NO_WINDOW)
                .output();

            #[cfg(not(windows))]
            let _ = Command::new("kill")
                .args(["-9", &pid.to_string()])
                .output();

            // Also try to kill directly
            let _ = child.kill();
            let _ = child.wait();

            // Emit process stopped event
            let _ = app_handle.emit(
                "process-stopped",
                serde_json::json!({
                    "projectId": project_id
                }),
            );

            Ok(())
        } else {
            Err(ProcessError::NotFound(project_id.to_string()))
        }
    }

    pub fn is_running(&self, project_id: &str) -> bool {
        self.processes.contains_key(project_id)
    }

    pub fn get_running_processes(&self) -> Vec<String> {
        self.processes.keys().cloned().collect()
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}
