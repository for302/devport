use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use chrono::Local;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Information about a backup file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    pub file_path: String,
    pub file_name: String,
    pub created_at: String,
    pub size_bytes: u64,
}

/// Result of backup operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupResult {
    pub success: bool,
    pub backup_path: Option<String>,
    pub files_backed_up: Vec<String>,
    pub error: Option<String>,
}

/// Result of restore operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResult {
    pub success: bool,
    pub files_restored: Vec<String>,
    pub error: Option<String>,
}

/// Get the config directory paths
fn get_config_paths() -> (PathBuf, PathBuf) {
    let appdata = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("clickdevport");

    let local_appdata = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("clickdevport");

    (appdata, local_appdata)
}

/// Get the default backup directory
fn get_backup_dir() -> PathBuf {
    dirs::document_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ClickDevPort Backups")
}

/// Get the path to config directory
#[tauri::command]
pub async fn get_config_path() -> Result<String, String> {
    let (appdata, _) = get_config_paths();
    Ok(appdata.to_string_lossy().to_string())
}

/// List all config files that can be backed up
#[tauri::command]
pub async fn list_config_files() -> Result<Vec<String>, String> {
    let (appdata, local_appdata) = get_config_paths();
    let mut files = Vec::new();

    // Check appdata files
    let appdata_files = [
        "projects.json",
        "installed_components.json",
        "settings.json",
    ];

    for file in appdata_files {
        let path = appdata.join(file);
        if path.exists() {
            files.push(path.to_string_lossy().to_string());
        }
    }

    // Check local appdata files
    let local_files = [
        "autostart_config.json",
        "session_state.json",
    ];

    for file in local_files {
        let path = local_appdata.join(file);
        if path.exists() {
            files.push(path.to_string_lossy().to_string());
        }
    }

    Ok(files)
}

/// Create a backup of all config files
#[tauri::command]
pub async fn create_backup(custom_path: Option<String>) -> Result<BackupResult, String> {
    let (appdata, local_appdata) = get_config_paths();

    // Determine backup destination
    let backup_dir = custom_path
        .map(PathBuf::from)
        .unwrap_or_else(get_backup_dir);

    // Create backup directory
    fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;

    // Generate backup filename with timestamp
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_filename = format!("clickdevport_backup_{}.zip", timestamp);
    let backup_path = backup_dir.join(&backup_filename);

    // Create zip file
    let file = fs::File::create(&backup_path)
        .map_err(|e| format!("Failed to create backup file: {}", e))?;
    let mut zip = zip::ZipWriter::new(file);

    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(6));

    let mut backed_up_files = Vec::new();

    // Backup appdata files
    let appdata_files = [
        ("projects.json", "projects.json"),
        ("installed_components.json", "installed_components.json"),
        ("settings.json", "settings.json"),
    ];

    for (filename, archive_name) in appdata_files {
        let file_path = appdata.join(filename);
        if file_path.exists() {
            if let Ok(content) = fs::read(&file_path) {
                zip.start_file(format!("appdata/{}", archive_name), options)
                    .map_err(|e| format!("Failed to add file to backup: {}", e))?;
                std::io::Write::write_all(&mut zip, &content)
                    .map_err(|e| format!("Failed to write to backup: {}", e))?;
                backed_up_files.push(filename.to_string());
            }
        }
    }

    // Backup local appdata files
    let local_files = [
        ("autostart_config.json", "autostart_config.json"),
    ];

    for (filename, archive_name) in local_files {
        let file_path = local_appdata.join(filename);
        if file_path.exists() {
            if let Ok(content) = fs::read(&file_path) {
                zip.start_file(format!("localappdata/{}", archive_name), options)
                    .map_err(|e| format!("Failed to add file to backup: {}", e))?;
                std::io::Write::write_all(&mut zip, &content)
                    .map_err(|e| format!("Failed to write to backup: {}", e))?;
                backed_up_files.push(filename.to_string());
            }
        }
    }

    zip.finish()
        .map_err(|e| format!("Failed to finalize backup: {}", e))?;

    Ok(BackupResult {
        success: true,
        backup_path: Some(backup_path.to_string_lossy().to_string()),
        files_backed_up: backed_up_files,
        error: None,
    })
}

/// Restore from a backup file
#[tauri::command]
pub async fn restore_backup(backup_path: String) -> Result<RestoreResult, String> {
    let (appdata, local_appdata) = get_config_paths();

    // Ensure directories exist
    fs::create_dir_all(&appdata)
        .map_err(|e| format!("Failed to create appdata directory: {}", e))?;
    fs::create_dir_all(&local_appdata)
        .map_err(|e| format!("Failed to create local appdata directory: {}", e))?;

    // Open backup file
    let file = fs::File::open(&backup_path)
        .map_err(|e| format!("Failed to open backup file: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("Failed to read backup archive: {}", e))?;

    let mut restored_files = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .map_err(|e| format!("Failed to read archive entry: {}", e))?;

        let name = file.name().to_string();

        let dest_path = if name.starts_with("appdata/") {
            let filename = name.strip_prefix("appdata/").unwrap();
            appdata.join(filename)
        } else if name.starts_with("localappdata/") {
            let filename = name.strip_prefix("localappdata/").unwrap();
            local_appdata.join(filename)
        } else {
            continue;
        };

        // Read file content
        let mut content = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut content)
            .map_err(|e| format!("Failed to read file from backup: {}", e))?;

        // Write to destination
        fs::write(&dest_path, &content)
            .map_err(|e| format!("Failed to write restored file: {}", e))?;

        restored_files.push(dest_path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default());
    }

    Ok(RestoreResult {
        success: true,
        files_restored: restored_files,
        error: None,
    })
}

/// List available backups in the default backup directory
#[tauri::command]
pub async fn list_backups() -> Result<Vec<BackupInfo>, String> {
    let backup_dir = get_backup_dir();

    if !backup_dir.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();

    let entries = fs::read_dir(&backup_dir)
        .map_err(|e| format!("Failed to read backup directory: {}", e))?;

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "zip") {
            if let Some(filename) = path.file_name() {
                let filename_str = filename.to_string_lossy().to_string();

                // Only include clickdevport backups
                if filename_str.starts_with("clickdevport_backup_") {
                    let metadata = fs::metadata(&path).ok();

                    let created_at = metadata
                        .as_ref()
                        .and_then(|m| m.created().ok())
                        .map(|t| {
                            let datetime: chrono::DateTime<chrono::Local> = t.into();
                            datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                        })
                        .unwrap_or_else(|| "Unknown".to_string());

                    let size_bytes = metadata.map(|m| m.len()).unwrap_or(0);

                    backups.push(BackupInfo {
                        file_path: path.to_string_lossy().to_string(),
                        file_name: filename_str,
                        created_at,
                        size_bytes,
                    });
                }
            }
        }
    }

    // Sort by filename (which includes timestamp) in reverse order
    backups.sort_by(|a, b| b.file_name.cmp(&a.file_name));

    Ok(backups)
}

/// Delete a backup file
#[tauri::command]
pub async fn delete_backup(backup_path: String) -> Result<(), String> {
    let path = PathBuf::from(&backup_path);

    if !path.exists() {
        return Err("Backup file not found".to_string());
    }

    // Safety check: only delete files in the backup directory
    let backup_dir = get_backup_dir();
    if !path.starts_with(&backup_dir) {
        return Err("Cannot delete files outside of backup directory".to_string());
    }

    fs::remove_file(&path)
        .map_err(|e| format!("Failed to delete backup: {}", e))
}

/// Open the backup directory in file explorer
#[tauri::command]
pub async fn open_backup_folder() -> Result<(), String> {
    let backup_dir = get_backup_dir();

    // Create directory if it doesn't exist
    fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;

    #[cfg(windows)]
    {
        use std::process::Command;
        Command::new("explorer")
            .arg(&backup_dir)
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(not(windows))]
    {
        use std::process::Command;
        Command::new("xdg-open")
            .arg(&backup_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    Ok(())
}

/// Open the config directory in file explorer
#[tauri::command]
pub async fn open_config_folder() -> Result<(), String> {
    let (appdata, _) = get_config_paths();

    #[cfg(windows)]
    {
        use std::process::Command;
        Command::new("explorer")
            .arg(&appdata)
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(not(windows))]
    {
        use std::process::Command;
        Command::new("xdg-open")
            .arg(&appdata)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    Ok(())
}
