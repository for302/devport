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

/// Windows constant for creating a process without a console window
#[cfg(windows)]
pub const CREATE_NO_WINDOW: u32 = 0x08000000;

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

/// Result of a process kill operation
#[derive(Debug)]
pub struct KillResult {
    pub success: bool,
    pub error: Option<String>,
}

/// Kill a process tree by PID.
/// On Windows, uses `taskkill /F /T /PID` to kill the process and all child processes.
/// On Unix, uses `kill -9`.
///
/// # Arguments
/// * `pid` - The process ID to kill
///
/// # Returns
/// * `KillResult` indicating success or failure with optional error message
pub fn kill_process_tree(pid: u32) -> KillResult {
    #[cfg(windows)]
    {
        let output = Command::new("taskkill")
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        match output {
            Ok(out) => {
                if out.status.success() {
                    KillResult { success: true, error: None }
                } else {
                    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                    KillResult {
                        success: false,
                        error: Some(stderr.trim().to_string())
                    }
                }
            }
            Err(e) => KillResult {
                success: false,
                error: Some(e.to_string()),
            },
        }
    }

    #[cfg(not(windows))]
    {
        let output = Command::new("kill")
            .args(["-9", &pid.to_string()])
            .output();

        match output {
            Ok(out) => {
                if out.status.success() {
                    KillResult { success: true, error: None }
                } else {
                    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                    KillResult {
                        success: false,
                        error: Some(stderr.trim().to_string())
                    }
                }
            }
            Err(e) => KillResult {
                success: false,
                error: Some(e.to_string()),
            },
        }
    }
}

/// Kill a process tree by PID, ignoring the result.
/// Convenience wrapper around `kill_process_tree` for fire-and-forget scenarios.
pub fn kill_process_tree_silent(pid: u32) {
    let _ = kill_process_tree(pid);
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
        // Only inject PORT for web projects (port > 0)
        // Tauri/Electron manage their own dev server ports via config files
        // (e.g., tauri.conf.json devUrl). Injecting PORT causes a port mismatch
        // where Vite starts on the injected port but the native app waits for the
        // configured port, preventing the native window from ever launching.
        if project.port > 0
            && !matches!(
                project.project_type,
                ProjectType::Tauri | ProjectType::Electron
            )
        {
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
        // Use CREATE_NO_WINDOW to prevent console window from appearing
        #[cfg(target_os = "windows")]
        let mut child = Command::new("cmd")
            .args(["/C", &command_with_port])
            .current_dir(&project.path)
            .envs(&env_vars)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW)
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
            let project_type = project.project_type.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                let mut launched_notified = false;
                for line in reader.lines().map_while(Result::ok) {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let lower = trimmed.to_lowercase();

                    // Detect framework-specific readiness from stdout
                    if !launched_notified {
                        let is_ready = match project_type {
                            ProjectType::Vite | ProjectType::React | ProjectType::Vue | ProjectType::Svelte => {
                                lower.contains("ready in") || lower.contains("local:")
                            }
                            ProjectType::NextJs => {
                                lower.contains("ready in") || lower.contains("local:")
                            }
                            ProjectType::Flask => {
                                lower.contains("running on")
                            }
                            ProjectType::Django => {
                                lower.contains("starting development server")
                            }
                            ProjectType::FastApi => {
                                lower.contains("application startup complete")
                            }
                            ProjectType::Node | ProjectType::Express => {
                                lower.contains("listening on") || lower.contains("server running")
                            }
                            ProjectType::Electron => {
                                lower.contains("electron") && lower.contains("ready")
                            }
                            ProjectType::Angular => {
                                lower.contains("compiled successfully") || lower.contains("local:")
                            }
                            _ => false,
                        };
                        if is_ready {
                            launched_notified = true;
                            let _ = app.emit(
                                "build-status",
                                serde_json::json!({
                                    "projectId": project_id,
                                    "status": "launched",
                                    "message": trimmed
                                }),
                            );
                        }
                    }

                    // Emit progress for meaningful stdout lines (URL, version, status)
                    if !launched_notified {
                        let is_progress =
                            lower.contains("http://localhost") || lower.contains("http://127.0.0.1")
                            || lower.contains("vite v") || lower.contains("next.js")
                            || lower.contains("webpack") || lower.contains("compiled")
                            || lower.contains("ready in") || lower.contains("starting")
                            || lower.contains("listening") || lower.contains("running on")
                            || lower.contains("server running");
                        if is_progress {
                            let _ = app.emit(
                                "build-status",
                                serde_json::json!({
                                    "projectId": project_id,
                                    "status": "progress",
                                    "message": trimmed
                                }),
                            );
                        }
                    }

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
            let is_tauri = matches!(project.project_type, ProjectType::Tauri);
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                let mut build_notified = false;
                let mut compile_count: u32 = 0;
                for line in reader.lines().map_while(Result::ok) {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    // Detect Tauri/Cargo build status from stderr
                    if is_tauri {
                        if trimmed.starts_with("Compiling") {
                            compile_count += 1;
                            if !build_notified {
                                build_notified = true;
                                let _ = app.emit(
                                    "build-status",
                                    serde_json::json!({
                                        "projectId": project_id,
                                        "status": "compiling",
                                        "message": trimmed
                                    }),
                                );
                            } else if compile_count % 20 == 0 {
                                // Every 20 crates, emit progress so user sees movement
                                let _ = app.emit(
                                    "build-status",
                                    serde_json::json!({
                                        "projectId": project_id,
                                        "status": "progress",
                                        "message": format!("Compiled {} crates... ({})", compile_count, trimmed)
                                    }),
                                );
                            }
                        } else if trimmed.starts_with("Downloading") || trimmed.starts_with("Downloaded") {
                            let _ = app.emit(
                                "build-status",
                                serde_json::json!({
                                    "projectId": project_id,
                                    "status": "progress",
                                    "message": trimmed
                                }),
                            );
                        } else if trimmed.starts_with("Finished") {
                            let msg = if compile_count > 0 {
                                format!("{} ({} crates compiled)", trimmed, compile_count)
                            } else {
                                trimmed.to_string()
                            };
                            let _ = app.emit(
                                "build-status",
                                serde_json::json!({
                                    "projectId": project_id,
                                    "status": "compiled",
                                    "message": msg
                                }),
                            );
                        } else if trimmed.starts_with("Running") {
                            let _ = app.emit(
                                "build-status",
                                serde_json::json!({
                                    "projectId": project_id,
                                    "status": "launched",
                                    "message": trimmed
                                }),
                            );
                        } else if line.contains("error[E") {
                            let _ = app.emit(
                                "build-status",
                                serde_json::json!({
                                    "projectId": project_id,
                                    "status": "error",
                                    "message": trimmed
                                }),
                            );
                        }
                    } else {
                        // Non-Tauri: detect build/error from stderr
                        let lower = trimmed.to_lowercase();
                        if !build_notified && (lower.contains("building") || lower.contains("bundling") || lower.contains("compiling")) {
                            build_notified = true;
                            let _ = app.emit(
                                "build-status",
                                serde_json::json!({
                                    "projectId": project_id,
                                    "status": "compiling",
                                    "message": trimmed
                                }),
                            );
                        } else if lower.contains("error:") || lower.contains("error ") && lower.contains("failed") {
                            let _ = app.emit(
                                "build-status",
                                serde_json::json!({
                                    "projectId": project_id,
                                    "status": "error",
                                    "message": trimmed
                                }),
                            );
                        } else if lower.contains("downloading") || lower.contains("installing")
                            || lower.contains("resolving") || lower.contains("transforming")
                            || lower.contains("optimizing") || lower.contains("generating")
                        {
                            let _ = app.emit(
                                "build-status",
                                serde_json::json!({
                                    "projectId": project_id,
                                    "status": "progress",
                                    "message": trimmed
                                }),
                            );
                        }
                    }

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

        // Emit starting status (before process-started so UI can show "Starting...")
        let _ = app_handle.emit(
            "build-status",
            serde_json::json!({
                "projectId": project_id,
                "status": "starting"
            }),
        );

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
