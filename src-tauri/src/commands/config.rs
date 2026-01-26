use std::path::PathBuf;
use std::fs;
use regex::Regex;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigFile {
    pub name: String,
    pub path: String,
    pub content: String,
    pub file_type: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigSection {
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
}

/// Apache port/vhost entry with details
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApachePortEntry {
    pub port: u16,
    pub domain: String,           // ServerName (e.g., localhost, mysite.local)
    pub url: String,              // Full URL (e.g., http://localhost:8080)
    pub document_root: String,    // DocumentRoot folder path
    pub is_ssl: bool,
    pub server_alias: Vec<String>, // ServerAlias entries
    pub config_file: String,      // Which config file this came from
}

// ============================================================================
// Dynamic Path Finders
// ============================================================================

/// Find Apache installation path dynamically
fn find_apache_base_path() -> Option<PathBuf> {
    let possible_paths = [
        // XAMPP (most common)
        "C:\\xampp\\apache",
        "D:\\xampp\\apache",
        "E:\\xampp\\apache",
        // DevPort custom path
        "C:\\DevPort\\runtime\\apache",
        "D:\\DevPort\\runtime\\apache",
        // Laragon
        "C:\\laragon\\bin\\apache",
        "D:\\laragon\\bin\\apache",
    ];

    for base in possible_paths {
        let base_path = PathBuf::from(base);
        let httpd_exe = base_path.join("bin").join("httpd.exe");
        if httpd_exe.exists() {
            return Some(base_path);
        }
    }

    // Check WampServer with version subdirectories
    for wamp_base in ["C:\\wamp64\\bin\\apache", "C:\\wamp\\bin\\apache", "D:\\wamp64\\bin\\apache"] {
        let wamp_path = PathBuf::from(wamp_base);
        if wamp_path.exists() {
            if let Ok(entries) = fs::read_dir(&wamp_path) {
                for entry in entries.flatten() {
                    let subdir = entry.path();
                    if subdir.is_dir() {
                        let httpd_exe = subdir.join("bin").join("httpd.exe");
                        if httpd_exe.exists() {
                            return Some(subdir);
                        }
                    }
                }
            }
        }
    }

    // Check Laragon with version subdirectories
    for laragon_base in ["C:\\laragon\\bin\\apache", "D:\\laragon\\bin\\apache"] {
        let laragon_path = PathBuf::from(laragon_base);
        if laragon_path.exists() {
            if let Ok(entries) = fs::read_dir(&laragon_path) {
                for entry in entries.flatten() {
                    let subdir = entry.path();
                    if subdir.is_dir() {
                        let httpd_exe = subdir.join("bin").join("httpd.exe");
                        if httpd_exe.exists() {
                            return Some(subdir);
                        }
                    }
                }
            }
        }
    }

    None
}

/// Find MariaDB/MySQL installation path dynamically
fn find_mariadb_base_path() -> Option<PathBuf> {
    let possible_paths = [
        // XAMPP (most common) - uses mysql folder for MariaDB
        "C:\\xampp\\mysql",
        "D:\\xampp\\mysql",
        "E:\\xampp\\mysql",
        // DevPort custom path
        "C:\\DevPort\\runtime\\mariadb",
        "D:\\DevPort\\runtime\\mariadb",
        // Laragon MariaDB
        "C:\\laragon\\bin\\mariadb",
        "D:\\laragon\\bin\\mariadb",
        // Laragon MySQL
        "C:\\laragon\\bin\\mysql",
        "D:\\laragon\\bin\\mysql",
    ];

    for base in possible_paths {
        let base_path = PathBuf::from(base);
        let mysqld_exe = base_path.join("bin").join("mysqld.exe");
        if mysqld_exe.exists() {
            return Some(base_path);
        }
    }

    // Check WampServer with version subdirectories
    for wamp_base in ["C:\\wamp64\\bin\\mariadb", "C:\\wamp\\bin\\mariadb", "D:\\wamp64\\bin\\mariadb"] {
        let wamp_path = PathBuf::from(wamp_base);
        if wamp_path.exists() {
            if let Ok(entries) = fs::read_dir(&wamp_path) {
                for entry in entries.flatten() {
                    let subdir = entry.path();
                    if subdir.is_dir() {
                        let mysqld_exe = subdir.join("bin").join("mysqld.exe");
                        if mysqld_exe.exists() {
                            return Some(subdir);
                        }
                    }
                }
            }
        }
    }

    // Check Laragon with version subdirectories
    for laragon_base in ["C:\\laragon\\bin\\mariadb", "C:\\laragon\\bin\\mysql"] {
        let laragon_path = PathBuf::from(laragon_base);
        if laragon_path.exists() {
            if let Ok(entries) = fs::read_dir(&laragon_path) {
                for entry in entries.flatten() {
                    let subdir = entry.path();
                    if subdir.is_dir() {
                        let mysqld_exe = subdir.join("bin").join("mysqld.exe");
                        if mysqld_exe.exists() {
                            return Some(subdir);
                        }
                    }
                }
            }
        }
    }

    None
}

/// Find PHP installation path dynamically
fn find_php_base_path() -> Option<PathBuf> {
    let possible_paths = [
        // XAMPP (most common)
        "C:\\xampp\\php",
        "D:\\xampp\\php",
        "E:\\xampp\\php",
        // DevPort custom path
        "C:\\DevPort\\runtime\\php",
        "D:\\DevPort\\runtime\\php",
        // Laragon
        "C:\\laragon\\bin\\php",
        "D:\\laragon\\bin\\php",
    ];

    for base in possible_paths {
        let base_path = PathBuf::from(base);
        let php_exe = base_path.join("php.exe");
        if php_exe.exists() {
            return Some(base_path);
        }
    }

    // Check WampServer with version subdirectories
    for wamp_base in ["C:\\wamp64\\bin\\php", "C:\\wamp\\bin\\php", "D:\\wamp64\\bin\\php"] {
        let wamp_path = PathBuf::from(wamp_base);
        if wamp_path.exists() {
            if let Ok(entries) = fs::read_dir(&wamp_path) {
                for entry in entries.flatten() {
                    let subdir = entry.path();
                    if subdir.is_dir() {
                        let php_exe = subdir.join("php.exe");
                        if php_exe.exists() {
                            return Some(subdir);
                        }
                    }
                }
            }
        }
    }

    // Check Laragon with version subdirectories
    for laragon_base in ["C:\\laragon\\bin\\php"] {
        let laragon_path = PathBuf::from(laragon_base);
        if laragon_path.exists() {
            if let Ok(entries) = fs::read_dir(&laragon_path) {
                for entry in entries.flatten() {
                    let subdir = entry.path();
                    if subdir.is_dir() {
                        let php_exe = subdir.join("php.exe");
                        if php_exe.exists() {
                            return Some(subdir);
                        }
                    }
                }
            }
        }
    }

    None
}

// ============================================================================
// Apache Config Commands
// ============================================================================

/// Get Apache httpd.conf content
#[tauri::command]
pub async fn get_apache_config() -> Result<ConfigFile, String> {
    let base_path = find_apache_base_path()
        .ok_or_else(|| "Apache installation not found".to_string())?;

    let config_path = base_path.join("conf").join("httpd.conf");

    if !config_path.exists() {
        return Err(format!("Apache config file not found at: {}", config_path.display()));
    }

    let content = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;

    Ok(ConfigFile {
        name: "httpd.conf".to_string(),
        path: config_path.to_string_lossy().to_string(),
        content,
        file_type: "apache".to_string(),
    })
}

/// Save Apache httpd.conf content
#[tauri::command]
pub async fn save_apache_config(content: String) -> Result<(), String> {
    let base_path = find_apache_base_path()
        .ok_or_else(|| "Apache installation not found".to_string())?;

    let config_path = base_path.join("conf").join("httpd.conf");
    let backup_path = base_path.join("conf").join("httpd.conf.bak");

    // Create backup
    if config_path.exists() {
        fs::copy(&config_path, &backup_path).map_err(|e| format!("Failed to create backup: {}", e))?;
    }

    // Write new content
    fs::write(&config_path, content).map_err(|e| format!("Failed to save config: {}", e))?;

    Ok(())
}

/// Parse Apache config files to extract VirtualHost information
fn parse_apache_vhosts(content: &str, config_file: &str, default_doc_root: &str) -> Vec<ApachePortEntry> {
    let mut entries: Vec<ApachePortEntry> = Vec::new();

    // Regex patterns
    let vhost_start_re = Regex::new(r"(?i)<VirtualHost\s+[^:]*:(\d+)\s*>").unwrap();
    let vhost_end_re = Regex::new(r"(?i)</VirtualHost>").unwrap();
    let server_name_re = Regex::new(r"(?i)^\s*ServerName\s+(.+?)\s*$").unwrap();
    let server_alias_re = Regex::new(r"(?i)^\s*ServerAlias\s+(.+?)\s*$").unwrap();
    let doc_root_re = Regex::new(r#"(?i)^\s*DocumentRoot\s+"?([^"]+)"?\s*$"#).unwrap();
    let listen_re = Regex::new(r"(?i)^\s*Listen\s+(?:\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}:)?(\d+)").unwrap();
    let global_doc_root_re = Regex::new(r#"(?i)^\s*DocumentRoot\s+"?([^"]+)"?\s*$"#).unwrap();

    let mut in_vhost = false;
    let mut current_port: u16 = 0;
    let mut current_server_name = String::new();
    let mut current_doc_root = String::new();
    let mut current_aliases: Vec<String> = Vec::new();
    let mut global_doc_root = default_doc_root.to_string();
    let mut listen_ports: Vec<u16> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        // Track global DocumentRoot (outside VirtualHost)
        if !in_vhost {
            if let Some(caps) = global_doc_root_re.captures(trimmed) {
                global_doc_root = caps.get(1).unwrap().as_str().to_string();
            }

            // Track Listen directives for ports without VirtualHost
            if let Some(caps) = listen_re.captures(trimmed) {
                if let Ok(port) = caps.get(1).unwrap().as_str().parse::<u16>() {
                    if !listen_ports.contains(&port) {
                        listen_ports.push(port);
                    }
                }
            }
        }

        // VirtualHost start
        if let Some(caps) = vhost_start_re.captures(trimmed) {
            in_vhost = true;
            current_port = caps.get(1).unwrap().as_str().parse().unwrap_or(80);
            current_server_name.clear();
            current_doc_root.clear();
            current_aliases.clear();
            continue;
        }

        // VirtualHost end
        if vhost_end_re.is_match(trimmed) && in_vhost {
            let domain = if current_server_name.is_empty() {
                "localhost".to_string()
            } else {
                current_server_name.clone()
            };

            let doc_root = if current_doc_root.is_empty() {
                global_doc_root.clone()
            } else {
                current_doc_root.clone()
            };

            let is_ssl = current_port == 443;
            let protocol = if is_ssl { "https" } else { "http" };
            let url = if (is_ssl && current_port == 443) || (!is_ssl && current_port == 80) {
                format!("{}://{}", protocol, domain)
            } else {
                format!("{}://{}:{}", protocol, domain, current_port)
            };

            entries.push(ApachePortEntry {
                port: current_port,
                domain,
                url,
                document_root: doc_root.replace("/", "\\"),
                is_ssl,
                server_alias: current_aliases.clone(),
                config_file: config_file.to_string(),
            });

            in_vhost = false;
            continue;
        }

        // Inside VirtualHost - parse directives
        if in_vhost {
            if let Some(caps) = server_name_re.captures(trimmed) {
                current_server_name = caps.get(1).unwrap().as_str().to_string();
            }
            if let Some(caps) = server_alias_re.captures(trimmed) {
                let aliases: Vec<String> = caps.get(1).unwrap().as_str()
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
                current_aliases.extend(aliases);
            }
            if let Some(caps) = doc_root_re.captures(trimmed) {
                current_doc_root = caps.get(1).unwrap().as_str().to_string();
            }
        }
    }

    // Add entries for Listen ports that don't have VirtualHost
    for port in listen_ports {
        if !entries.iter().any(|e| e.port == port) {
            let is_ssl = port == 443;
            let protocol = if is_ssl { "https" } else { "http" };
            let url = if (is_ssl && port == 443) || (!is_ssl && port == 80) {
                format!("{}://localhost", protocol)
            } else {
                format!("{}://localhost:{}", protocol, port)
            };

            entries.push(ApachePortEntry {
                port,
                domain: "localhost".to_string(),
                url,
                document_root: global_doc_root.replace("/", "\\"),
                is_ssl,
                server_alias: vec![],
                config_file: config_file.to_string(),
            });
        }
    }

    entries
}

/// Get all configured Apache ports/vhosts from config files
#[tauri::command]
pub async fn get_apache_ports() -> Result<Vec<ApachePortEntry>, String> {
    let base_path = find_apache_base_path()
        .ok_or_else(|| "Apache installation not found".to_string())?;

    let mut all_entries: Vec<ApachePortEntry> = Vec::new();
    let default_doc_root = base_path.join("htdocs").to_string_lossy().to_string();

    // Parse main httpd.conf
    let config_path = base_path.join("conf").join("httpd.conf");
    if config_path.exists() {
        let content = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
        let entries = parse_apache_vhosts(&content, "httpd.conf", &default_doc_root);
        all_entries.extend(entries);
    }

    // Parse httpd-vhosts.conf (commonly used for VirtualHosts)
    let vhosts_path = base_path.join("conf").join("extra").join("httpd-vhosts.conf");
    if vhosts_path.exists() {
        if let Ok(content) = fs::read_to_string(&vhosts_path) {
            let entries = parse_apache_vhosts(&content, "httpd-vhosts.conf", &default_doc_root);
            // Add only if not duplicate port+domain
            for entry in entries {
                if !all_entries.iter().any(|e| e.port == entry.port && e.domain == entry.domain) {
                    all_entries.push(entry);
                }
            }
        }
    }

    // Parse httpd-ssl.conf for SSL vhosts
    let ssl_path = base_path.join("conf").join("extra").join("httpd-ssl.conf");
    if ssl_path.exists() {
        if let Ok(content) = fs::read_to_string(&ssl_path) {
            let entries = parse_apache_vhosts(&content, "httpd-ssl.conf", &default_doc_root);
            for entry in entries {
                if !all_entries.iter().any(|e| e.port == entry.port && e.domain == entry.domain) {
                    all_entries.push(entry);
                }
            }
        }
    }

    // Sort by port number
    all_entries.sort_by(|a, b| a.port.cmp(&b.port));

    Ok(all_entries)
}

/// Validate Apache config syntax
#[tauri::command]
pub async fn validate_apache_config() -> Result<String, String> {
    use std::process::Command;

    let base_path = find_apache_base_path()
        .ok_or_else(|| "Apache installation not found".to_string())?;

    let httpd_path = base_path.join("bin").join("httpd.exe");

    if !httpd_path.exists() {
        return Err("Apache executable not found".to_string());
    }

    let output = Command::new(&httpd_path)
        .arg("-t")
        .output()
        .map_err(|e| e.to_string())?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() || stderr.contains("Syntax OK") {
        Ok("Syntax OK".to_string())
    } else {
        Err(stderr)
    }
}

// ============================================================================
// MariaDB Config Commands
// ============================================================================

/// Get MariaDB my.ini content
#[tauri::command]
pub async fn get_mariadb_config() -> Result<ConfigFile, String> {
    let base_path = find_mariadb_base_path()
        .ok_or_else(|| "MariaDB/MySQL installation not found".to_string())?;

    // Try multiple possible locations for my.ini
    let possible_config_paths = [
        base_path.join("data").join("my.ini"),
        base_path.join("my.ini"),
        base_path.join("bin").join("my.ini"),
        base_path.join("my.cnf"),
    ];

    for config_path in possible_config_paths {
        if config_path.exists() {
            let content = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
            return Ok(ConfigFile {
                name: config_path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "my.ini".to_string()),
                path: config_path.to_string_lossy().to_string(),
                content,
                file_type: "ini".to_string(),
            });
        }
    }

    Err("MariaDB config file (my.ini) not found".to_string())
}

/// Save MariaDB my.ini content
#[tauri::command]
pub async fn save_mariadb_config(content: String, path: String) -> Result<(), String> {
    let config_path = PathBuf::from(&path);
    let backup_path = config_path.with_extension("ini.bak");

    // Create backup
    if config_path.exists() {
        fs::copy(&config_path, &backup_path).map_err(|e| format!("Failed to create backup: {}", e))?;
    }

    // Write new content
    fs::write(&config_path, content).map_err(|e| format!("Failed to save config: {}", e))?;

    Ok(())
}

// ============================================================================
// PHP Config Commands
// ============================================================================

/// Get PHP php.ini content
#[tauri::command]
pub async fn get_php_config() -> Result<ConfigFile, String> {
    let base_path = find_php_base_path()
        .ok_or_else(|| "PHP installation not found".to_string())?;

    // Try multiple possible locations for php.ini
    let possible_config_paths = [
        base_path.join("php.ini"),
        base_path.join("php.ini-development"),
        base_path.join("php.ini-production"),
    ];

    for config_path in possible_config_paths {
        if config_path.exists() {
            let content = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
            return Ok(ConfigFile {
                name: config_path.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "php.ini".to_string()),
                path: config_path.to_string_lossy().to_string(),
                content,
                file_type: "ini".to_string(),
            });
        }
    }

    Err("PHP config file (php.ini) not found".to_string())
}

/// Save PHP php.ini content
#[tauri::command]
pub async fn save_php_config(content: String) -> Result<(), String> {
    let base_path = find_php_base_path()
        .ok_or_else(|| "PHP installation not found".to_string())?;

    let config_path = base_path.join("php.ini");
    let backup_path = base_path.join("php.ini.bak");

    // Create backup
    if config_path.exists() {
        fs::copy(&config_path, &backup_path).map_err(|e| format!("Failed to create backup: {}", e))?;
    }

    // Write new content
    fs::write(&config_path, content).map_err(|e| format!("Failed to save config: {}", e))?;

    Ok(())
}

// ============================================================================
// Restore Backup
// ============================================================================

/// Restore config from backup
#[tauri::command]
pub async fn restore_config_backup(config_type: String) -> Result<(), String> {
    let (config_path, backup_path) = match config_type.as_str() {
        "apache" => {
            let base_path = find_apache_base_path()
                .ok_or_else(|| "Apache installation not found".to_string())?;
            (
                base_path.join("conf").join("httpd.conf"),
                base_path.join("conf").join("httpd.conf.bak"),
            )
        },
        "mariadb" => {
            let base_path = find_mariadb_base_path()
                .ok_or_else(|| "MariaDB installation not found".to_string())?;
            // Find existing config first
            let possible_paths = [
                base_path.join("data").join("my.ini"),
                base_path.join("my.ini"),
                base_path.join("bin").join("my.ini"),
            ];
            let config = possible_paths.iter()
                .find(|p| p.exists())
                .ok_or_else(|| "MariaDB config not found".to_string())?
                .clone();
            let backup = config.with_extension("ini.bak");
            (config, backup)
        },
        "php" => {
            let base_path = find_php_base_path()
                .ok_or_else(|| "PHP installation not found".to_string())?;
            (
                base_path.join("php.ini"),
                base_path.join("php.ini.bak"),
            )
        },
        _ => return Err("Unknown config type".to_string()),
    };

    if !backup_path.exists() {
        return Err("No backup file found".to_string());
    }

    fs::copy(&backup_path, &config_path).map_err(|e| format!("Failed to restore backup: {}", e))?;

    Ok(())
}
