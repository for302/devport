use std::path::PathBuf;
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldOptions {
    pub name: String,
    pub path: String,
    pub framework: String,
    pub package_manager: Option<String>,
    pub typescript: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldResult {
    pub success: bool,
    pub message: String,
    pub project_path: String,
}

/// Get the DevPort runtime PATH
fn get_devport_path() -> String {
    let paths = [
        "C:\\DevPort\\runtime\\nodejs",
        "C:\\DevPort\\runtime\\php",
        "C:\\DevPort\\runtime\\git\\bin",
        "C:\\DevPort\\tools\\composer",
    ];

    let system_path = std::env::var("PATH").unwrap_or_default();
    format!("{};{}", paths.join(";"), system_path)
}

/// Create a new project using framework-specific scaffolding
#[tauri::command]
pub async fn scaffold_project(options: ScaffoldOptions) -> Result<ScaffoldResult, String> {
    let project_path = PathBuf::from(&options.path);
    let parent_path = project_path.parent().ok_or("Invalid project path")?;

    // Ensure parent directory exists
    std::fs::create_dir_all(parent_path).map_err(|e| e.to_string())?;

    let pm = options.package_manager.as_deref().unwrap_or("npm");
    let devport_path = get_devport_path();

    let result = match options.framework.as_str() {
        "nextjs" => scaffold_nextjs(&options.name, parent_path, pm, &devport_path).await,
        "vite-react" => scaffold_vite(&options.name, parent_path, pm, "react", options.typescript.unwrap_or(true), &devport_path).await,
        "vite-vue" => scaffold_vite(&options.name, parent_path, pm, "vue", options.typescript.unwrap_or(true), &devport_path).await,
        "vite-svelte" => scaffold_vite(&options.name, parent_path, pm, "svelte", options.typescript.unwrap_or(true), &devport_path).await,
        "laravel" => scaffold_laravel(&options.name, parent_path, &devport_path).await,
        "vanilla-ts" => scaffold_vite(&options.name, parent_path, pm, "vanilla", true, &devport_path).await,
        "vanilla-php" => scaffold_vanilla_php(&options.name, &project_path).await,
        _ => Err(format!("Unknown framework: {}", options.framework)),
    };

    match result {
        Ok(msg) => Ok(ScaffoldResult {
            success: true,
            message: msg,
            project_path: project_path.to_string_lossy().to_string(),
        }),
        Err(e) => Ok(ScaffoldResult {
            success: false,
            message: e,
            project_path: project_path.to_string_lossy().to_string(),
        }),
    }
}

async fn scaffold_nextjs(name: &str, parent: &std::path::Path, pm: &str, path_env: &str) -> Result<String, String> {
    let create_cmd = match pm {
        "pnpm" => "pnpm",
        "yarn" => "yarn",
        _ => "npx",
    };

    let args: Vec<&str> = match pm {
        "pnpm" => vec!["create", "next-app", name, "--typescript", "--eslint", "--tailwind", "--src-dir", "--app", "--import-alias", "@/*"],
        "yarn" => vec!["create", "next-app", name, "--typescript", "--eslint", "--tailwind", "--src-dir", "--app", "--import-alias", "@/*"],
        _ => vec!["create-next-app@latest", name, "--typescript", "--eslint", "--tailwind", "--src-dir", "--app", "--import-alias", "@/*", "--use-npm"],
    };

    let output = Command::new(create_cmd)
        .args(&args)
        .current_dir(parent)
        .env("PATH", path_env)
        .output()
        .map_err(|e| format!("Failed to run create-next-app: {}", e))?;

    if output.status.success() {
        Ok("Next.js project created successfully".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to create Next.js project: {}", stderr))
    }
}

async fn scaffold_vite(name: &str, parent: &std::path::Path, pm: &str, template: &str, typescript: bool, path_env: &str) -> Result<String, String> {
    let template_name = if typescript {
        format!("{}-ts", template)
    } else {
        template.to_string()
    };

    let (cmd, args): (&str, Vec<&str>) = match pm {
        "pnpm" => ("pnpm", vec!["create", "vite", name, "--template", &template_name]),
        "yarn" => ("yarn", vec!["create", "vite", name, "--template", &template_name]),
        _ => ("npm", vec!["create", "vite@latest", name, "--", "--template", &template_name]),
    };

    let output = Command::new(cmd)
        .args(&args)
        .current_dir(parent)
        .env("PATH", path_env)
        .output()
        .map_err(|e| format!("Failed to run create vite: {}", e))?;

    if output.status.success() {
        Ok(format!("Vite {} project created successfully", template))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to create Vite project: {}", stderr))
    }
}

async fn scaffold_laravel(name: &str, parent: &std::path::Path, path_env: &str) -> Result<String, String> {
    // Use composer create-project
    let output = Command::new("composer")
        .args(["create-project", "laravel/laravel", name])
        .current_dir(parent)
        .env("PATH", path_env)
        .output()
        .map_err(|e| format!("Failed to run composer: {}", e))?;

    if output.status.success() {
        Ok("Laravel project created successfully".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to create Laravel project: {}", stderr))
    }
}

async fn scaffold_vanilla_php(name: &str, project_path: &PathBuf) -> Result<String, String> {
    // Create directory structure
    std::fs::create_dir_all(project_path).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(project_path.join("public")).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(project_path.join("src")).map_err(|e| e.to_string())?;

    // Create index.php
    let index_content = format!(r#"<?php
/**
 * {} - Main Entry Point
 */

require_once __DIR__ . '/../src/bootstrap.php';

echo "Hello from {}!";
"#, name, name);

    std::fs::write(project_path.join("public/index.php"), index_content)
        .map_err(|e| e.to_string())?;

    // Create bootstrap.php
    let bootstrap_content = r#"<?php
/**
 * Bootstrap file - Initialize application
 */

// Error reporting
error_reporting(E_ALL);
ini_set('display_errors', '1');

// Autoloader (if using composer)
if (file_exists(__DIR__ . '/../vendor/autoload.php')) {
    require_once __DIR__ . '/../vendor/autoload.php';
}
"#;

    std::fs::write(project_path.join("src/bootstrap.php"), bootstrap_content)
        .map_err(|e| e.to_string())?;

    // Create .htaccess for Apache
    let htaccess_content = r#"RewriteEngine On
RewriteCond %{REQUEST_FILENAME} !-f
RewriteCond %{REQUEST_FILENAME} !-d
RewriteRule ^(.*)$ index.php [QSA,L]
"#;

    std::fs::write(project_path.join("public/.htaccess"), htaccess_content)
        .map_err(|e| e.to_string())?;

    Ok("Vanilla PHP project created successfully".to_string())
}

/// Install dependencies for a project
#[tauri::command]
pub async fn install_dependencies(project_path: String, package_manager: String) -> Result<String, String> {
    let path = PathBuf::from(&project_path);
    let devport_path = get_devport_path();

    // Check for package.json (Node.js project)
    if path.join("package.json").exists() {
        let cmd = match package_manager.as_str() {
            "pnpm" => "pnpm",
            "yarn" => "yarn",
            _ => "npm",
        };

        let output = Command::new(cmd)
            .arg("install")
            .current_dir(&path)
            .env("PATH", &devport_path)
            .output()
            .map_err(|e| format!("Failed to run {}: {}", cmd, e))?;

        if output.status.success() {
            return Ok("Dependencies installed successfully".to_string());
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to install dependencies: {}", stderr));
        }
    }

    // Check for composer.json (PHP project)
    if path.join("composer.json").exists() {
        let output = Command::new("composer")
            .arg("install")
            .current_dir(&path)
            .env("PATH", &devport_path)
            .output()
            .map_err(|e| format!("Failed to run composer: {}", e))?;

        if output.status.success() {
            return Ok("Composer dependencies installed successfully".to_string());
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to install composer dependencies: {}", stderr));
        }
    }

    Err("No package.json or composer.json found".to_string())
}

/// Check if Python is available on the system
#[tauri::command]
pub async fn check_python_available() -> Result<(bool, String), String> {
    // Try python first, then python3
    for cmd in ["python", "python3"] {
        let output = Command::new(cmd)
            .arg("--version")
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let version = String::from_utf8_lossy(&out.stdout).to_string();
                // Check if it's a real Python (not Windows Store stub)
                if !version.contains("was not found") {
                    return Ok((true, version.trim().to_string()));
                }
            }
        }
    }

    Ok((false, "Python not found".to_string()))
}

/// Get available framework templates
#[tauri::command]
pub async fn get_framework_templates() -> Result<Vec<FrameworkTemplate>, String> {
    Ok(vec![
        FrameworkTemplate {
            id: "nextjs".to_string(),
            name: "Next.js".to_string(),
            category: "Node.js".to_string(),
            description: "React framework with SSR and file-based routing".to_string(),
            icon: "nextjs".to_string(),
        },
        FrameworkTemplate {
            id: "vite-react".to_string(),
            name: "React (Vite)".to_string(),
            category: "Node.js".to_string(),
            description: "React with Vite for fast development".to_string(),
            icon: "react".to_string(),
        },
        FrameworkTemplate {
            id: "vite-vue".to_string(),
            name: "Vue (Vite)".to_string(),
            category: "Node.js".to_string(),
            description: "Vue.js with Vite for fast development".to_string(),
            icon: "vue".to_string(),
        },
        FrameworkTemplate {
            id: "vite-svelte".to_string(),
            name: "Svelte (Vite)".to_string(),
            category: "Node.js".to_string(),
            description: "Svelte with Vite for fast development".to_string(),
            icon: "svelte".to_string(),
        },
        FrameworkTemplate {
            id: "laravel".to_string(),
            name: "Laravel".to_string(),
            category: "PHP".to_string(),
            description: "PHP framework for web artisans".to_string(),
            icon: "laravel".to_string(),
        },
        FrameworkTemplate {
            id: "vanilla-ts".to_string(),
            name: "Vanilla TypeScript".to_string(),
            category: "Node.js".to_string(),
            description: "Plain TypeScript with Vite".to_string(),
            icon: "typescript".to_string(),
        },
        FrameworkTemplate {
            id: "vanilla-php".to_string(),
            name: "Vanilla PHP".to_string(),
            category: "PHP".to_string(),
            description: "Plain PHP project structure".to_string(),
            icon: "php".to_string(),
        },
    ])
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameworkTemplate {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub icon: String,
}
