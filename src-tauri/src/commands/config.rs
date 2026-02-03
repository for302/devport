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
    pub id: String,               // Unique ID: "{port}_{domain}"
    pub port: u16,
    pub domain: String,           // ServerName (e.g., localhost, mysite.local)
    pub url: String,              // Full URL (e.g., http://localhost:8080)
    pub document_root: String,    // DocumentRoot folder path
    pub is_ssl: bool,
    pub server_alias: Vec<String>, // ServerAlias entries
    pub config_file: String,      // Which config file this came from
    pub framework: String,        // Detected framework (e.g., "Laravel", "CodeIgniter", "PHP")
}

/// Request structure for creating/updating VirtualHost
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApacheVHostRequest {
    pub port: u16,
    pub domain: String,
    pub document_root: String,
    pub server_alias: Vec<String>,
    pub is_ssl: bool,
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
// Framework Detection
// ============================================================================

/// Detect framework from document root
fn detect_framework(document_root: &str) -> String {
    let path = PathBuf::from(document_root);

    // Check for Laravel (artisan file)
    if path.join("artisan").exists() {
        return "Laravel".to_string();
    }

    // Check for CodeIgniter 4 (spark file)
    if path.join("spark").exists() {
        return "CodeIgniter".to_string();
    }

    // Check for Symfony (bin/console)
    if path.join("bin").join("console").exists() {
        return "Symfony".to_string();
    }

    // Check for WordPress (wp-config.php or wp-content folder)
    if path.join("wp-config.php").exists() || path.join("wp-content").exists() {
        return "WordPress".to_string();
    }

    // Check for Next.js (next.config.js or next.config.mjs)
    if path.join("next.config.js").exists() || path.join("next.config.mjs").exists() {
        return "Next.js".to_string();
    }

    // Check for Nuxt (nuxt.config.ts or nuxt.config.js)
    if path.join("nuxt.config.ts").exists() || path.join("nuxt.config.js").exists() {
        return "Nuxt".to_string();
    }

    // Check for Vue (vue.config.js)
    if path.join("vue.config.js").exists() {
        return "Vue".to_string();
    }

    // Check for React (public/index.html + src/App.js pattern or vite.config with react)
    if path.join("public").join("index.html").exists() && path.join("src").exists() {
        return "React".to_string();
    }

    // Check for composer.json (general PHP project)
    if path.join("composer.json").exists() {
        return "PHP".to_string();
    }

    // Check for package.json (general Node.js project)
    if path.join("package.json").exists() {
        return "Node.js".to_string();
    }

    // Check for any PHP files
    if let Ok(entries) = fs::read_dir(&path) {
        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension() {
                if ext == "php" {
                    return "PHP".to_string();
                }
            }
        }
    }

    // Check for HTML files
    if path.join("index.html").exists() {
        return "HTML".to_string();
    }

    "Unknown".to_string()
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

            let id = format!("{}_{}", current_port, domain.replace(".", "_"));
            let doc_root_normalized = doc_root.replace("/", "\\");
            let framework = detect_framework(&doc_root_normalized);
            entries.push(ApachePortEntry {
                id,
                port: current_port,
                domain,
                url,
                document_root: doc_root_normalized,
                is_ssl,
                server_alias: current_aliases.clone(),
                config_file: config_file.to_string(),
                framework,
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
            let id = format!("{}_localhost", port);
            let doc_root_normalized = global_doc_root.replace("/", "\\");
            let framework = detect_framework(&doc_root_normalized);

            entries.push(ApachePortEntry {
                id,
                port,
                domain: "localhost".to_string(),
                url,
                document_root: doc_root_normalized,
                is_ssl,
                server_alias: vec![],
                config_file: config_file.to_string(),
                framework,
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
    #[cfg(windows)]
    use std::os::windows::process::CommandExt;

    let base_path = find_apache_base_path()
        .ok_or_else(|| "Apache installation not found".to_string())?;

    let httpd_path = base_path.join("bin").join("httpd.exe");

    if !httpd_path.exists() {
        return Err("Apache executable not found".to_string());
    }

    #[cfg(windows)]
    let output = Command::new(&httpd_path)
        .arg("-t")
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
        .map_err(|e| e.to_string())?;

    #[cfg(not(windows))]
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

// ============================================================================
// Apache VirtualHost CRUD Commands
// ============================================================================

/// Build a regex that matches a VirtualHost block by port and document root.
/// Handles both cases: with or without ServerName directive.
fn build_vhost_match_regex(port: u16, document_root: &str) -> Result<Regex, String> {
    // Normalize the document root for matching (handle both / and \ separators)
    let doc_root_escaped = regex::escape(document_root)
        .replace(r"\\", r"[\\/]")
        .replace(r"/", r"[\\/]");

    // Match the VirtualHost block by port + DocumentRoot content
    // Captures optional preceding comment line(s) as well
    let pattern = format!(
        "(?sm)(?:^[ \\t]*#[^\\n]*\\n)?<VirtualHost\\s+\\*:{}\\s*>\\s*\\n(?:.*?\\n)*?\\s*DocumentRoot\\s+\"?{}\"?\\s*\\n.*?</VirtualHost>\\s*\\n?",
        port,
        doc_root_escaped
    );
    Regex::new(&pattern).map_err(|e| format!("Regex error: {}", e))
}

/// Helper function to get vhosts config path
fn get_vhosts_config_path() -> Result<PathBuf, String> {
    let base_path = find_apache_base_path()
        .ok_or_else(|| "Apache installation not found".to_string())?;
    Ok(base_path.join("conf").join("extra").join("httpd-vhosts.conf"))
}

/// Helper function to get httpd.conf path
fn get_httpd_config_path() -> Result<PathBuf, String> {
    let base_path = find_apache_base_path()
        .ok_or_else(|| "Apache installation not found".to_string())?;
    Ok(base_path.join("conf").join("httpd.conf"))
}

/// Helper function to backup a file before modifying
fn backup_config_file(path: &PathBuf) -> Result<(), String> {
    let backup_path = path.with_extension(format!(
        "{}.bak",
        path.extension().unwrap_or_default().to_string_lossy()
    ));
    if path.exists() {
        fs::copy(path, &backup_path)
            .map_err(|e| format!("Failed to create backup: {}", e))?;
    }
    Ok(())
}

/// Generate VirtualHost block content
fn generate_vhost_block(request: &ApacheVHostRequest) -> String {
    let mut block = format!(
        "<VirtualHost *:{}>\n    ServerName {}\n",
        request.port, request.domain
    );

    if !request.server_alias.is_empty() {
        block.push_str(&format!("    ServerAlias {}\n", request.server_alias.join(" ")));
    }

    // Normalize path for Apache (use forward slashes)
    let doc_root = request.document_root.replace("\\", "/");
    block.push_str(&format!("    DocumentRoot \"{}\"\n", doc_root));
    block.push_str(&format!("    <Directory \"{}\">\n", doc_root));
    block.push_str("        AllowOverride All\n");
    block.push_str("        Require all granted\n");
    block.push_str("    </Directory>\n");

    if request.is_ssl {
        block.push_str("    SSLEngine on\n");
    }

    block.push_str("</VirtualHost>\n");
    block
}

/// Create a new VirtualHost entry
#[tauri::command]
pub async fn create_apache_vhost(request: ApacheVHostRequest) -> Result<ApachePortEntry, String> {
    let vhosts_path = get_vhosts_config_path()?;

    // Create backup
    backup_config_file(&vhosts_path)?;

    // Read existing content or create new
    let mut content = if vhosts_path.exists() {
        fs::read_to_string(&vhosts_path).map_err(|e| e.to_string())?
    } else {
        String::new()
    };

    // Check if vhost already exists
    let existing_ports = get_apache_ports().await?;
    if existing_ports.iter().any(|e| e.port == request.port && e.domain == request.domain) {
        return Err(format!("VirtualHost for port {} and domain {} already exists", request.port, request.domain));
    }

    // Ensure Listen port exists in httpd.conf
    ensure_listen_port(request.port).await?;

    // Generate and append new VirtualHost block
    let vhost_block = generate_vhost_block(&request);
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str("\n");
    content.push_str(&vhost_block);

    // Write updated content
    fs::write(&vhosts_path, &content).map_err(|e| format!("Failed to write vhosts config: {}", e))?;

    // Return the created entry
    let id = format!("{}_{}", request.port, request.domain.replace(".", "_"));
    let protocol = if request.is_ssl { "https" } else { "http" };
    let url = if (request.is_ssl && request.port == 443) || (!request.is_ssl && request.port == 80) {
        format!("{}://{}", protocol, request.domain)
    } else {
        format!("{}://{}:{}", protocol, request.domain, request.port)
    };
    let doc_root_normalized = request.document_root.replace("/", "\\");
    let framework = detect_framework(&doc_root_normalized);

    Ok(ApachePortEntry {
        id,
        port: request.port,
        domain: request.domain,
        url,
        document_root: doc_root_normalized,
        is_ssl: request.is_ssl,
        server_alias: request.server_alias,
        config_file: "httpd-vhosts.conf".to_string(),
        framework,
    })
}

/// Update an existing VirtualHost entry
#[tauri::command]
pub async fn update_apache_vhost(id: String, request: ApacheVHostRequest) -> Result<ApachePortEntry, String> {
    // Find the existing entry
    let existing_ports = get_apache_ports().await?;
    let existing = existing_ports.iter().find(|e| e.id == id)
        .ok_or_else(|| format!("VirtualHost with id {} not found", id))?;

    let config_file = &existing.config_file;
    let config_path = if config_file == "httpd-vhosts.conf" {
        get_vhosts_config_path()?
    } else if config_file == "httpd.conf" {
        get_httpd_config_path()?
    } else {
        return Err(format!("Editing VirtualHosts in {} is not supported", config_file));
    };

    // Backup
    backup_config_file(&config_path)?;

    // Read content
    let content = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;

    // Find and replace the VirtualHost block using port + document root matching
    let vhost_regex = build_vhost_match_regex(existing.port, &existing.document_root)?;

    let new_block = generate_vhost_block(&request);
    let new_content = vhost_regex.replace(&content, new_block.as_str()).to_string();

    if new_content == content {
        return Err("Could not find the VirtualHost block to update".to_string());
    }

    // Write updated content
    fs::write(&config_path, &new_content).map_err(|e| format!("Failed to write config: {}", e))?;

    // Handle port change - ensure new Listen port exists
    if existing.port != request.port {
        ensure_listen_port(request.port).await?;
    }

    // Return updated entry
    let new_id = format!("{}_{}", request.port, request.domain.replace(".", "_"));
    let protocol = if request.is_ssl { "https" } else { "http" };
    let url = if (request.is_ssl && request.port == 443) || (!request.is_ssl && request.port == 80) {
        format!("{}://{}", protocol, request.domain)
    } else {
        format!("{}://{}:{}", protocol, request.domain, request.port)
    };
    let doc_root_normalized = request.document_root.replace("/", "\\");
    let framework = detect_framework(&doc_root_normalized);

    Ok(ApachePortEntry {
        id: new_id,
        port: request.port,
        domain: request.domain,
        url,
        document_root: doc_root_normalized,
        is_ssl: request.is_ssl,
        server_alias: request.server_alias,
        config_file: config_file.clone(),
        framework,
    })
}

/// Delete a VirtualHost entry
#[tauri::command]
pub async fn delete_apache_vhost(id: String) -> Result<(), String> {
    // Find the existing entry
    let existing_ports = get_apache_ports().await?;
    let existing = existing_ports.iter().find(|e| e.id == id)
        .ok_or_else(|| format!("VirtualHost with id {} not found", id))?;

    let config_file = &existing.config_file;
    let config_path = if config_file == "httpd-vhosts.conf" {
        get_vhosts_config_path()?
    } else if config_file == "httpd.conf" {
        get_httpd_config_path()?
    } else {
        return Err(format!("Deleting VirtualHosts from {} is not supported", config_file));
    };

    // Backup
    backup_config_file(&config_path)?;

    // Read content
    let content = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;

    // Find and remove the VirtualHost block using port + document root matching
    let vhost_regex = build_vhost_match_regex(existing.port, &existing.document_root)?;

    let new_content = vhost_regex.replace(&content, "").to_string();

    if new_content == content {
        return Err(format!(
            "Could not find VirtualHost block for port {} with DocumentRoot '{}' in {}",
            existing.port, existing.document_root, config_file
        ));
    }

    // Write updated content
    fs::write(&config_path, &new_content).map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(())
}

/// Helper to ensure a Listen port exists in httpd.conf
async fn ensure_listen_port(port: u16) -> Result<(), String> {
    let httpd_path = get_httpd_config_path()?;
    let content = fs::read_to_string(&httpd_path).map_err(|e| e.to_string())?;

    // Check if Listen port already exists
    let listen_regex = Regex::new(&format!(r"(?m)^\s*Listen\s+(?:\d{{1,3}}\.\d{{1,3}}\.\d{{1,3}}\.\d{{1,3}}:)?{}\s*$", port))
        .map_err(|e| format!("Regex error: {}", e))?;

    if listen_regex.is_match(&content) {
        return Ok(()); // Port already configured
    }

    // Add Listen directive after existing Listen directives
    backup_config_file(&httpd_path)?;

    let last_listen_regex = Regex::new(r"(?m)^(\s*Listen\s+.+)$")
        .map_err(|e| format!("Regex error: {}", e))?;

    let new_content = if let Some(mat) = last_listen_regex.find_iter(&content).last() {
        let insert_pos = mat.end();
        format!("{}\nListen {}{}", &content[..insert_pos], port, &content[insert_pos..])
    } else {
        // No existing Listen directive, add at beginning
        format!("Listen {}\n{}", port, content)
    };

    fs::write(&httpd_path, &new_content).map_err(|e| format!("Failed to write httpd.conf: {}", e))?;

    Ok(())
}

/// Add a Listen port (without VirtualHost)
#[tauri::command]
pub async fn add_listen_port(port: u16) -> Result<(), String> {
    ensure_listen_port(port).await
}

/// Remove a Listen port from httpd.conf
#[tauri::command]
pub async fn remove_listen_port(port: u16) -> Result<(), String> {
    let httpd_path = get_httpd_config_path()?;
    backup_config_file(&httpd_path)?;

    let content = fs::read_to_string(&httpd_path).map_err(|e| e.to_string())?;

    // Remove Listen directive for this port
    let listen_regex = Regex::new(&format!(r"(?m)^\s*Listen\s+(?:\d{{1,3}}\.\d{{1,3}}\.\d{{1,3}}\.\d{{1,3}}:)?{}\s*\n?", port))
        .map_err(|e| format!("Regex error: {}", e))?;

    let new_content = listen_regex.replace_all(&content, "").to_string();

    fs::write(&httpd_path, &new_content).map_err(|e| format!("Failed to write httpd.conf: {}", e))?;

    Ok(())
}

/// Check if a DocumentRoot path exists
#[tauri::command]
pub async fn check_document_root(path: String) -> Result<bool, String> {
    let path = PathBuf::from(&path);
    Ok(path.exists() && path.is_dir())
}

/// Create DocumentRoot folder if it doesn't exist
#[tauri::command]
pub async fn create_document_root(path: String) -> Result<(), String> {
    let path = PathBuf::from(&path);

    if path.exists() {
        if path.is_dir() {
            return Ok(());
        } else {
            return Err("Path exists but is not a directory".to_string());
        }
    }

    fs::create_dir_all(&path).map_err(|e| format!("Failed to create directory: {}", e))?;

    // Create a basic index.html file
    let index_path = path.join("index.html");
    let index_content = r#"<!DOCTYPE html>
<html>
<head>
    <title>Welcome</title>
</head>
<body>
    <h1>It works!</h1>
    <p>This is the default page for this site.</p>
</body>
</html>
"#;

    fs::write(&index_path, index_content)
        .map_err(|e| format!("Failed to create index.html: {}", e))?;

    Ok(())
}

/// Get the Apache base path (exported for other modules)
#[tauri::command]
pub async fn get_apache_base_path() -> Result<String, String> {
    find_apache_base_path()
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| "Apache installation not found".to_string())
}
