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

#[derive(Debug, Clone)]
pub struct DetectedProject {
    pub project_type: ProjectType,
    pub name: String,
    pub start_command: String,
    pub default_port: u16,
}

pub struct ProjectDetector;

impl ProjectDetector {
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
    fn read_tauri_port(project_path: &Path) -> Option<u16> {
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

        // Default unknown
        Ok(DetectedProject {
            project_type: ProjectType::Unknown,
            name: path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            start_command: String::new(),
            default_port: 3000,
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
            });
        }

        // Detect Vue (without Vite)
        if dependencies.get("vue").is_some() {
            return Ok(DetectedProject {
                project_type: ProjectType::Vue,
                name,
                start_command: "npm run serve".to_string(),
                default_port: 8080,
            });
        }

        // Detect React (CRA or standalone)
        if dependencies.get("react").is_some() {
            return Ok(DetectedProject {
                project_type: ProjectType::React,
                name,
                start_command: "npm start".to_string(),
                default_port: 3000,
            });
        }

        // Detect Express
        if dependencies.get("express").is_some() {
            return Ok(DetectedProject {
                project_type: ProjectType::Express,
                name,
                start_command: "npm start".to_string(),
                default_port: 3000,
            });
        }

        // Default Node.js
        Ok(DetectedProject {
            project_type: ProjectType::Node,
            name,
            start_command: "npm start".to_string(),
            default_port: 3000,
        })
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

        // Check for Django
        if requirements.contains("django") || requirements.contains("Django") {
            let manage_py = path.join("manage.py");
            if manage_py.exists() {
                return Ok(DetectedProject {
                    project_type: ProjectType::Django,
                    name,
                    start_command: "python manage.py runserver".to_string(),
                    default_port: 8000,
                });
            }
        }

        // Check for FastAPI
        if requirements.contains("fastapi") {
            return Ok(DetectedProject {
                project_type: ProjectType::FastApi,
                name,
                start_command: "uvicorn main:app --reload".to_string(),
                default_port: 8000,
            });
        }

        // Check for Flask
        if requirements.contains("flask") || requirements.contains("Flask") {
            return Ok(DetectedProject {
                project_type: ProjectType::Flask,
                name,
                start_command: "flask run".to_string(),
                default_port: 5000,
            });
        }

        // Default Python
        Ok(DetectedProject {
            project_type: ProjectType::Python,
            name,
            start_command: "python main.py".to_string(),
            default_port: 8000,
        })
    }
}
