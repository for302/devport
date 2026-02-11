use crate::models::ProjectType;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DetectorError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Path not found: {0}")]
    PathNotFound(String),
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedProject {
    pub project_type: ProjectType,
    pub name: String,
    pub start_command: String,
    pub default_port: u16,
    pub venv_path: Option<String>,
    pub github_url: Option<String>,
}

pub struct ProjectDetector;

impl ProjectDetector {
    /// Update Tauri project port in tauri.conf.json (devUrl) and vite.config.ts (port)
    pub fn update_tauri_port(project_path: &Path, new_port: u16) -> Result<(), String> {
        // 1. Update tauri.conf.json: build.devUrl
        let tauri_conf = project_path.join("src-tauri").join("tauri.conf.json");
        if tauri_conf.exists() {
            let content = fs::read_to_string(&tauri_conf)
                .map_err(|e| format!("Failed to read tauri.conf.json: {}", e))?;
            let mut json: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse tauri.conf.json: {}", e))?;

            if let Some(dev_url) = json.pointer_mut("/build/devUrl") {
                if let Some(url_str) = dev_url.as_str() {
                    // Replace port in URL like "http://localhost:1420"
                    let re = regex::Regex::new(r"(https?://[^:]+:)\d+")
                        .map_err(|e| format!("Regex error: {}", e))?;
                    let new_url = re.replace(url_str, format!("${{1}}{}", new_port).as_str());
                    *dev_url = serde_json::Value::String(new_url.to_string());
                }
            }

            let pretty = serde_json::to_string_pretty(&json)
                .map_err(|e| format!("Failed to serialize tauri.conf.json: {}", e))?;
            fs::write(&tauri_conf, pretty)
                .map_err(|e| format!("Failed to write tauri.conf.json: {}", e))?;
        }

        // 2. Update vite.config.ts: port: NNNN (all occurrences)
        let vite_configs = ["vite.config.ts", "vite.config.js", "vite.config.mts", "vite.config.mjs"];
        for config_name in &vite_configs {
            let config_path = project_path.join(config_name);
            if config_path.exists() {
                let content = fs::read_to_string(&config_path)
                    .map_err(|e| format!("Failed to read {}: {}", config_name, e))?;

                let re = regex::Regex::new(r"port:\s*\d+")
                    .map_err(|e| format!("Regex error: {}", e))?;
                let new_content = re.replace_all(&content, format!("port: {}", new_port).as_str());

                if new_content != content {
                    fs::write(&config_path, new_content.as_ref())
                        .map_err(|e| format!("Failed to write {}: {}", config_name, e))?;
                }
                break; // Only update the first matching config file
            }
        }

        Ok(())
    }

    /// Check for pnpm-lock.yaml
    fn has_pnpm_lock(project_path: &Path) -> bool {
        project_path.join("pnpm-lock.yaml").exists()
    }

    /// Check for yarn.lock
    fn has_yarn_lock(project_path: &Path) -> bool {
        project_path.join("yarn.lock").exists()
    }

    /// Extract port number from a string after a specific pattern
    fn extract_port_after_pattern(content: &str, pattern: &str) -> Option<u16> {
        if let Some(pos) = content.find(pattern) {
            let after = &content[pos + pattern.len()..];
            // Skip whitespace
            let trimmed = after.trim_start();
            // Extract digits
            let port_str: String = trimmed.chars().take_while(|c| c.is_ascii_digit()).collect();
            if !port_str.is_empty() {
                return port_str.parse::<u16>().ok();
            }
        }
        None
    }

    /// Extract port from URL like "http://localhost:1420"
    fn extract_port_from_url(url: &str) -> Option<u16> {
        // Find the last colon followed by digits
        if let Some(colon_pos) = url.rfind(':') {
            let after_colon = &url[colon_pos + 1..];
            // Extract digits, stop at / or end
            let port_str: String = after_colon.chars().take_while(|c| c.is_ascii_digit()).collect();
            if !port_str.is_empty() {
                return port_str.parse::<u16>().ok();
            }
        }
        None
    }

    /// Read port from vite.config.ts or vite.config.js
    fn read_vite_port(project_path: &Path) -> Option<u16> {
        let config_files = ["vite.config.ts", "vite.config.js", "vite.config.mts", "vite.config.mjs"];

        for config_file in &config_files {
            let config_path = project_path.join(config_file);
            if config_path.exists() {
                if let Ok(content) = fs::read_to_string(&config_path) {
                    // Try to find "port:" or "port :" pattern (not "export", "import", etc.)
                    // Look for patterns like "port:" or "port :" with word boundary
                    for pattern in &["port:", "port :"] {
                        if let Some(port) = Self::extract_port_after_pattern(&content, pattern) {
                            return Some(port);
                        }
                    }
                }
            }
        }
        None
    }

    /// Read port from tauri.conf.json (devUrl)
    pub fn read_tauri_port(project_path: &Path) -> Option<u16> {
        let tauri_conf = project_path.join("src-tauri").join("tauri.conf.json");
        if tauri_conf.exists() {
            if let Ok(content) = fs::read_to_string(&tauri_conf) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(dev_url) = json.pointer("/build/devUrl").and_then(|v| v.as_str()) {
                        return Self::extract_port_from_url(dev_url);
                    }
                }
            }
        }
        None
    }

    /// Read port from package.json scripts (for Next.js -p flag)
    fn read_next_port(project_path: &Path) -> Option<u16> {
        let package_json = project_path.join("package.json");
        if package_json.exists() {
            if let Ok(content) = fs::read_to_string(&package_json) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(dev_script) = json.pointer("/scripts/dev").and_then(|v| v.as_str()) {
                        // Check for -p <port> or --port <port>
                        if let Some(port) = Self::extract_port_after_pattern(dev_script, "-p ") {
                            return Some(port);
                        }
                        if let Some(port) = Self::extract_port_after_pattern(dev_script, "--port ") {
                            return Some(port);
                        }
                    }
                }
            }
        }
        None
    }

    /// Detect the actual configured port for a project
    fn detect_configured_port(project_path: &Path, project_type: &ProjectType, default_port: u16) -> u16 {
        // First check if this is a Tauri project
        if project_path.join("src-tauri").exists() {
            if let Some(port) = Self::read_tauri_port(project_path) {
                return port;
            }
        }

        // Then check based on project type
        match project_type {
            ProjectType::Vite | ProjectType::React | ProjectType::Vue | ProjectType::Svelte => {
                Self::read_vite_port(project_path).unwrap_or(default_port)
            }
            ProjectType::NextJs => {
                Self::read_next_port(project_path).unwrap_or(default_port)
            }
            _ => default_port,
        }
    }

    pub fn detect(path: &str) -> Result<DetectedProject, DetectorError> {
        let path = Path::new(path);
        if !path.exists() {
            return Err(DetectorError::PathNotFound(path.display().to_string()));
        }

        // Check for package.json (Node.js projects)
        let package_json = path.join("package.json");
        if package_json.exists() {
            return Self::detect_from_package_json(&package_json, path);
        }

        // Check for Python projects
        let requirements = path.join("requirements.txt");
        let pyproject = path.join("pyproject.toml");
        if requirements.exists() || pyproject.exists() {
            return Self::detect_python_project(path);
        }

        // Check for Python .py files (even without requirements.txt)
        let has_py_files = path.join("main.py").exists() || path.join("app.py").exists();
        if has_py_files {
            return Self::detect_python_project(path);
        }

        // Default unknown
        Ok(DetectedProject {
            project_type: ProjectType::Unknown,
            name: path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            start_command: String::new(),
            default_port: 3000,
            venv_path: None,
            github_url: Self::detect_github_url(path),
        })
    }

    fn detect_from_package_json(
        package_json: &Path,
        project_path: &Path,
    ) -> Result<DetectedProject, DetectorError> {
        let content = fs::read_to_string(package_json)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;

        let name = json["name"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string();

        let dependencies = json.get("dependencies").cloned().unwrap_or_default();
        let dev_dependencies = json.get("devDependencies").cloned().unwrap_or_default();
        let scripts = json.get("scripts").cloned().unwrap_or_default();

        // Detect Tauri (HIGHEST PRIORITY)
        // Check for src-tauri folder OR @tauri-apps/cli devDependency
        let has_tauri_folder = project_path.join("src-tauri").exists();
        let has_tauri_cli = dev_dependencies.get("@tauri-apps/cli").is_some();

        if has_tauri_folder || has_tauri_cli {
            // Determine the package manager from scripts
            let start_cmd = if scripts.get("tauri").is_some() {
                if Self::has_pnpm_lock(project_path) {
                    "pnpm tauri dev"
                } else if Self::has_yarn_lock(project_path) {
                    "yarn tauri dev"
                } else {
                    "npm run tauri dev"
                }
            } else {
                "npm run tauri dev"
            };

            // Read port from tauri.conf.json or fall back to vite config
            let default_port = Self::read_tauri_port(project_path)
                .or_else(|| Self::read_vite_port(project_path))
                .unwrap_or(1420);

            return Ok(DetectedProject {
                project_type: ProjectType::Tauri,
                name,
                start_command: start_cmd.to_string(),
                default_port,
                venv_path: None,
                github_url: Self::detect_github_url(project_path),
            });
        }

        // Detect Electron (HIGH PRIORITY)
        if dependencies.get("electron").is_some() || dev_dependencies.get("electron").is_some() {
            // Find the appropriate start command for Electron
            let start_cmd = if scripts.get("electron:dev").is_some() {
                "npm run electron:dev"
            } else if scripts.get("electron-dev").is_some() {
                "npm run electron-dev"
            } else if scripts.get("dev").is_some() {
                // Check if dev script includes electron
                let dev_script = scripts.get("dev").and_then(|v| v.as_str()).unwrap_or("");
                if dev_script.contains("electron") {
                    "npm run dev"
                } else {
                    "npm start"
                }
            } else {
                "npm start"
            };

            // Electron apps typically use a configurable port
            let default_port = Self::read_vite_port(project_path).unwrap_or(3000);

            return Ok(DetectedProject {
                project_type: ProjectType::Electron,
                name,
                start_command: start_cmd.to_string(),
                default_port,
                venv_path: None,
                github_url: Self::detect_github_url(project_path),
            });
        }

        // Detect Next.js
        if dependencies.get("next").is_some() || dev_dependencies.get("next").is_some() {
            let start_cmd = if scripts.get("dev").is_some() {
                "npm run dev"
            } else {
                "npx next dev"
            };
            // Read actual port from package.json scripts or tauri.conf.json
            let default_port = Self::detect_configured_port(project_path, &ProjectType::NextJs, 3000);
            return Ok(DetectedProject {
                project_type: ProjectType::NextJs,
                name,
                start_command: start_cmd.to_string(),
                default_port,
                venv_path: None,
                github_url: Self::detect_github_url(project_path),
            });
        }

        // Detect Vite
        if dependencies.get("vite").is_some() || dev_dependencies.get("vite").is_some() {
            let project_type = if dependencies.get("vue").is_some() {
                ProjectType::Vue
            } else if dependencies.get("svelte").is_some() {
                ProjectType::Svelte
            } else if dependencies.get("react").is_some() {
                ProjectType::React
            } else {
                ProjectType::Vite
            };

            // Read actual port from vite.config.ts/js or tauri.conf.json
            let default_port = Self::detect_configured_port(project_path, &project_type, 5173);

            return Ok(DetectedProject {
                project_type,
                name,
                start_command: "npm run dev".to_string(),
                default_port,
                venv_path: None,
                github_url: Self::detect_github_url(project_path),
            });
        }

        // Detect Angular
        if dependencies.get("@angular/core").is_some()
            || dev_dependencies.get("@angular/cli").is_some()
        {
            return Ok(DetectedProject {
                project_type: ProjectType::Angular,
                name,
                start_command: "npm start".to_string(),
                default_port: 4200,
                venv_path: None,
                github_url: Self::detect_github_url(project_path),
            });
        }

        // Detect Vue (without Vite)
        if dependencies.get("vue").is_some() {
            return Ok(DetectedProject {
                project_type: ProjectType::Vue,
                name,
                start_command: "npm run serve".to_string(),
                default_port: 8080,
                venv_path: None,
                github_url: Self::detect_github_url(project_path),
            });
        }

        // Detect React (CRA or standalone)
        if dependencies.get("react").is_some() {
            return Ok(DetectedProject {
                project_type: ProjectType::React,
                name,
                start_command: "npm start".to_string(),
                default_port: 3000,
                venv_path: None,
                github_url: Self::detect_github_url(project_path),
            });
        }

        // Detect Express
        if dependencies.get("express").is_some() {
            return Ok(DetectedProject {
                project_type: ProjectType::Express,
                name,
                start_command: "npm start".to_string(),
                default_port: 3000,
                venv_path: None,
                github_url: Self::detect_github_url(project_path),
            });
        }

        // Default Node.js
        Ok(DetectedProject {
            project_type: ProjectType::Node,
            name,
            start_command: "npm start".to_string(),
            default_port: 3000,
            venv_path: None,
            github_url: Self::detect_github_url(project_path),
        })
    }

    /// Detect venv directory in project path
    fn detect_venv(path: &Path) -> Option<String> {
        for venv_dir in &["venv", ".venv", "env"] {
            let venv_path = path.join(venv_dir);
            // Check for Windows venv structure
            if venv_path.join("Scripts").join("python.exe").exists() {
                return Some(venv_dir.to_string());
            }
            // Check for Unix venv structure
            if venv_path.join("bin").join("python").exists() {
                return Some(venv_dir.to_string());
            }
        }
        None
    }

    /// Detect GitHub URL from .git/config
    /// Parses remote "origin" URL and converts SSH format to HTTPS
    pub fn detect_github_url(project_path: &Path) -> Option<String> {
        let git_config = project_path.join(".git").join("config");
        if !git_config.exists() {
            return None;
        }

        let content = fs::read_to_string(&git_config).ok()?;

        // Find [remote "origin"] section and extract URL
        let mut in_origin_section = false;
        for line in content.lines() {
            let trimmed = line.trim();

            // Check for section headers
            if trimmed.starts_with('[') {
                in_origin_section = trimmed == "[remote \"origin\"]";
                continue;
            }

            // If we're in the origin section, look for url
            if in_origin_section && trimmed.starts_with("url") {
                if let Some(url_part) = trimmed.split('=').nth(1) {
                    let url = url_part.trim();
                    return Self::convert_to_github_https_url(url);
                }
            }
        }

        None
    }

    /// Convert various Git URL formats to GitHub HTTPS URL
    /// Returns None if not a GitHub URL
    fn convert_to_github_https_url(url: &str) -> Option<String> {
        let url = url.trim();

        // Handle SSH format: git@github.com:user/repo.git
        if url.starts_with("git@github.com:") {
            let path = url.strip_prefix("git@github.com:")?;
            let path = path.strip_suffix(".git").unwrap_or(path);
            return Some(format!("https://github.com/{}", path));
        }

        // Handle HTTPS format: https://github.com/user/repo.git
        if url.starts_with("https://github.com/") {
            let path = url.strip_prefix("https://github.com/")?;
            let path = path.strip_suffix(".git").unwrap_or(path);
            return Some(format!("https://github.com/{}", path));
        }

        // Handle HTTP format: http://github.com/user/repo.git
        if url.starts_with("http://github.com/") {
            let path = url.strip_prefix("http://github.com/")?;
            let path = path.strip_suffix(".git").unwrap_or(path);
            return Some(format!("https://github.com/{}", path));
        }

        // Not a GitHub URL
        None
    }

    /// Detect Flask start command by analyzing app.py/main.py patterns
    /// Returns "python app.py" if `if __name__ == "__main__"` + `.run(` pattern exists
    /// Otherwise returns "flask run"
    fn detect_flask_start_command(project_path: &Path, venv: &Option<String>) -> String {
        // Check app.py first
        let app_py = project_path.join("app.py");
        if app_py.exists() {
            if let Ok(content) = fs::read_to_string(&app_py) {
                // Check for if __name__ == "__main__" pattern and .run() call
                if content.contains("if __name__")
                    && content.contains("__main__")
                    && content.contains(".run(")
                {
                    return Self::python_command(venv, "app.py");
                }
            }
        }

        // Check main.py as fallback
        let main_py = project_path.join("main.py");
        if main_py.exists() {
            if let Ok(content) = fs::read_to_string(&main_py) {
                if content.contains("if __name__")
                    && content.contains("__main__")
                    && content.contains(".run(")
                {
                    return Self::python_command(venv, "main.py");
                }
            }
        }

        // Default to flask run
        "flask run".to_string()
    }

    /// Build python command using venv if available
    fn python_command(venv: &Option<String>, script: &str) -> String {
        match venv {
            Some(venv_dir) => {
                if cfg!(windows) {
                    format!("{}\\Scripts\\python.exe {}", venv_dir, script)
                } else {
                    format!("{}/bin/python {}", venv_dir, script)
                }
            }
            None => format!("python {}", script),
        }
    }

    /// Scan .py files for GUI framework imports
    fn scan_py_imports(path: &Path) -> Option<ProjectType> {
        let py_files = ["main.py", "app.py", "gui.py", "window.py", "game.py"];
        let mut all_content = String::new();

        for py_file in &py_files {
            let file_path = path.join(py_file);
            if file_path.exists() {
                if let Ok(content) = fs::read_to_string(&file_path) {
                    all_content.push_str(&content);
                    all_content.push('\n');
                }
            }
        }

        if all_content.is_empty() {
            return None;
        }

        // Check for GUI framework imports (order matters - more specific first)
        if all_content.contains("import pygame") || all_content.contains("from pygame") {
            return Some(ProjectType::PythonPygame);
        }
        if all_content.contains("import kivy") || all_content.contains("from kivy") {
            return Some(ProjectType::PythonKivy);
        }
        if all_content.contains("PyQt5") || all_content.contains("PyQt6") || all_content.contains("PySide6") || all_content.contains("PySide2") {
            return Some(ProjectType::PythonPyQt);
        }
        if all_content.contains("import wx") || all_content.contains("from wx") {
            return Some(ProjectType::PythonWx);
        }
        if all_content.contains("import tkinter") || all_content.contains("from tkinter") {
            return Some(ProjectType::PythonTkinter);
        }

        None
    }

    fn detect_python_project(path: &Path) -> Result<DetectedProject, DetectorError> {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Python Project".to_string());

        let requirements_path = path.join("requirements.txt");
        let requirements = if requirements_path.exists() {
            fs::read_to_string(&requirements_path).unwrap_or_default()
        } else {
            String::new()
        };

        let pyproject_path = path.join("pyproject.toml");
        let pyproject = if pyproject_path.exists() {
            fs::read_to_string(&pyproject_path).unwrap_or_default()
        } else {
            String::new()
        };

        let deps_content = format!("{}\n{}", requirements, pyproject);

        // Detect venv
        let venv = Self::detect_venv(path);

        // Check for Django
        if deps_content.contains("django") || deps_content.contains("Django") {
            let manage_py = path.join("manage.py");
            if manage_py.exists() {
                let cmd = match &venv {
                    Some(v) => {
                        if cfg!(windows) {
                            format!("{}\\Scripts\\python.exe manage.py runserver", v)
                        } else {
                            format!("{}/bin/python manage.py runserver", v)
                        }
                    }
                    None => "python manage.py runserver".to_string(),
                };
                return Ok(DetectedProject {
                    project_type: ProjectType::Django,
                    name,
                    start_command: cmd,
                    default_port: 8000,
                    venv_path: venv,
                    github_url: Self::detect_github_url(path),
                });
            }
        }

        // Check for FastAPI
        if deps_content.contains("fastapi") {
            return Ok(DetectedProject {
                project_type: ProjectType::FastApi,
                name,
                start_command: "uvicorn main:app --reload".to_string(),
                default_port: 8000,
                venv_path: venv,
                github_url: Self::detect_github_url(path),
            });
        }

        // Check for Flask
        if deps_content.contains("flask") || deps_content.contains("Flask") {
            // Detect start command based on app.py pattern
            let start_command = Self::detect_flask_start_command(path, &venv);

            return Ok(DetectedProject {
                project_type: ProjectType::Flask,
                name,
                start_command,
                default_port: 5000,
                venv_path: venv,
                github_url: Self::detect_github_url(path),
            });
        }

        // Check for GUI frameworks in requirements/pyproject
        if deps_content.contains("pygame") {
            let entry = if path.join("main.py").exists() { "main.py" } else { "game.py" };
            return Ok(DetectedProject {
                project_type: ProjectType::PythonPygame,
                name,
                start_command: Self::python_command(&venv, entry),
                default_port: 0,
                venv_path: venv,
                github_url: Self::detect_github_url(path),
            });
        }
        if deps_content.contains("kivy") {
            return Ok(DetectedProject {
                project_type: ProjectType::PythonKivy,
                name,
                start_command: Self::python_command(&venv, "main.py"),
                default_port: 0,
                venv_path: venv,
                github_url: Self::detect_github_url(path),
            });
        }
        if deps_content.contains("PyQt5") || deps_content.contains("PyQt6")
            || deps_content.contains("PySide6") || deps_content.contains("PySide2")
        {
            return Ok(DetectedProject {
                project_type: ProjectType::PythonPyQt,
                name,
                start_command: Self::python_command(&venv, "main.py"),
                default_port: 0,
                venv_path: venv,
                github_url: Self::detect_github_url(path),
            });
        }
        if deps_content.contains("wxPython") || deps_content.contains("wxpython") {
            return Ok(DetectedProject {
                project_type: ProjectType::PythonWx,
                name,
                start_command: Self::python_command(&venv, "main.py"),
                default_port: 0,
                venv_path: venv,
                github_url: Self::detect_github_url(path),
            });
        }

        // Scan .py files for import-based detection (e.g., tkinter is built-in)
        if let Some(gui_type) = Self::scan_py_imports(path) {
            let entry = if path.join("main.py").exists() {
                "main.py"
            } else if path.join("app.py").exists() {
                "app.py"
            } else if path.join("gui.py").exists() {
                "gui.py"
            } else {
                "main.py"
            };
            return Ok(DetectedProject {
                project_type: gui_type,
                name,
                start_command: Self::python_command(&venv, entry),
                default_port: 0,
                venv_path: venv,
                github_url: Self::detect_github_url(path),
            });
        }

        // Default Python
        let entry = if path.join("main.py").exists() {
            "main.py"
        } else if path.join("app.py").exists() {
            "app.py"
        } else {
            "main.py"
        };
        Ok(DetectedProject {
            project_type: ProjectType::Python,
            name,
            start_command: Self::python_command(&venv, entry),
            default_port: 8000,
            venv_path: venv,
            github_url: Self::detect_github_url(path),
        })
    }
}
