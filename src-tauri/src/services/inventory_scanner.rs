use crate::models::inventory::{InstallSource, InventoryCategory, InventoryItem, InventoryResult};
use crate::services::port_scanner::PortScanner;
use regex::Regex;
use std::path::Path;
use std::process::Command;
use std::time::Instant;
use chrono::Local;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Definition of a tool to scan for
struct ToolDefinition {
    id: &'static str,
    name: &'static str,
    category: InventoryCategory,
    commands: &'static [&'static str],
    /// Known installation paths to check (for tools not in PATH like XAMPP)
    known_paths: &'static [&'static str],
    version_arg: &'static str,
    version_regex: &'static str,
    port: Option<u16>,
}

const TOOL_DEFINITIONS: &[ToolDefinition] = &[
    // Runtimes
    ToolDefinition {
        id: "node",
        name: "Node.js",
        category: InventoryCategory::Runtime,
        commands: &["node"],
        known_paths: &[
            "C:\\Program Files\\nodejs\\node.exe",
            "C:\\Program Files (x86)\\nodejs\\node.exe",
        ],
        version_arg: "--version",
        version_regex: r"v?(\d+\.\d+\.\d+)",
        port: None,
    },
    ToolDefinition {
        id: "php",
        name: "PHP",
        category: InventoryCategory::Runtime,
        commands: &["php"],
        known_paths: &[
            "C:\\xampp\\php\\php.exe",
            "D:\\xampp\\php\\php.exe",
            "C:\\laragon\\bin\\php\\php-8.2.0-nts-Win32-vs16-x64\\php.exe",
            "C:\\laragon\\bin\\php\\php-8.1.0-nts-Win32-vs16-x64\\php.exe",
            "C:\\wamp64\\bin\\php\\php8.2.0\\php.exe",
            "C:\\wamp64\\bin\\php\\php8.1.0\\php.exe",
            "C:\\DevPort\\runtime\\php\\php.exe",
        ],
        version_arg: "--version",
        version_regex: r"PHP (\d+\.\d+\.\d+)",
        port: None,
    },
    ToolDefinition {
        id: "python",
        name: "Python",
        category: InventoryCategory::Runtime,
        commands: &["python", "python3"],
        known_paths: &[
            "C:\\Python312\\python.exe",
            "C:\\Python311\\python.exe",
            "C:\\Python310\\python.exe",
        ],
        version_arg: "--version",
        version_regex: r"Python (\d+\.\d+\.\d+)",
        port: None,
    },
    // Web Servers
    ToolDefinition {
        id: "apache",
        name: "Apache",
        category: InventoryCategory::WebServer,
        commands: &["httpd"],
        known_paths: &[
            "C:\\xampp\\apache\\bin\\httpd.exe",
            "D:\\xampp\\apache\\bin\\httpd.exe",
            "C:\\laragon\\bin\\apache\\httpd-2.4.54-win64-VS17\\bin\\httpd.exe",
            "C:\\wamp64\\bin\\apache\\apache2.4.54.2\\bin\\httpd.exe",
            "C:\\DevPort\\runtime\\apache\\bin\\httpd.exe",
            "C:\\Apache24\\bin\\httpd.exe",
        ],
        version_arg: "-v",
        version_regex: r"Apache/(\d+\.\d+\.\d+)",
        port: Some(80),
    },
    // Databases - NOTE: mysql/mariadb detection is handled separately in scan_mysql_or_mariadb()
    //            because XAMPP's mysql.exe is actually MariaDB engine.
    // Build Tools
    ToolDefinition {
        id: "vite",
        name: "Vite",
        category: InventoryCategory::BuildTool,
        commands: &["vite"],
        known_paths: &[],
        version_arg: "--version",
        version_regex: r"(\d+\.\d+\.\d+)",
        port: None,
    },
    ToolDefinition {
        id: "webpack",
        name: "Webpack",
        category: InventoryCategory::BuildTool,
        commands: &["webpack"],
        known_paths: &[],
        version_arg: "--version",
        version_regex: r"(\d+\.\d+\.\d+)",
        port: None,
    },
    ToolDefinition {
        id: "esbuild",
        name: "esbuild",
        category: InventoryCategory::BuildTool,
        commands: &["esbuild"],
        known_paths: &[],
        version_arg: "--version",
        version_regex: r"(\d+\.\d+\.\d+)",
        port: None,
    },
    ToolDefinition {
        id: "typescript",
        name: "TypeScript",
        category: InventoryCategory::BuildTool,
        commands: &["tsc"],
        known_paths: &[],
        version_arg: "--version",
        version_regex: r"Version (\d+\.\d+\.\d+)",
        port: None,
    },
    ToolDefinition {
        id: "gradle",
        name: "Gradle",
        category: InventoryCategory::BuildTool,
        commands: &["gradle"],
        known_paths: &[],
        version_arg: "--version",
        version_regex: r"Gradle (\d+\.\d+\.?\d*)",
        port: None,
    },
    ToolDefinition {
        id: "maven",
        name: "Maven",
        category: InventoryCategory::BuildTool,
        commands: &["mvn"],
        known_paths: &[],
        version_arg: "--version",
        version_regex: r"Apache Maven (\d+\.\d+\.\d+)",
        port: None,
    },
    // Package Managers
    ToolDefinition {
        id: "npm",
        name: "npm",
        category: InventoryCategory::PackageManager,
        commands: &["npm"],
        known_paths: &[
            "C:\\Program Files\\nodejs\\npm.cmd",
        ],
        version_arg: "--version",
        version_regex: r"(\d+\.\d+\.\d+)",
        port: None,
    },
    ToolDefinition {
        id: "pnpm",
        name: "pnpm",
        category: InventoryCategory::PackageManager,
        commands: &["pnpm"],
        known_paths: &[],
        version_arg: "--version",
        version_regex: r"(\d+\.\d+\.\d+)",
        port: None,
    },
    ToolDefinition {
        id: "yarn",
        name: "Yarn",
        category: InventoryCategory::PackageManager,
        commands: &["yarn"],
        known_paths: &[],
        version_arg: "--version",
        version_regex: r"(\d+\.\d+\.\d+)",
        port: None,
    },
    ToolDefinition {
        id: "pip",
        name: "pip",
        category: InventoryCategory::PackageManager,
        commands: &["pip", "pip3"],
        known_paths: &[],
        version_arg: "--version",
        version_regex: r"pip (\d+\.\d+\.?\d*)",
        port: None,
    },
    ToolDefinition {
        id: "composer",
        name: "Composer",
        category: InventoryCategory::PackageManager,
        commands: &["composer"],
        known_paths: &[
            "C:\\ProgramData\\ComposerSetup\\bin\\composer.bat",
            "C:\\xampp\\php\\composer.phar",
        ],
        version_arg: "--version",
        version_regex: r"Composer[^\d]*(\d+\.\d+\.\d+)",
        port: None,
    },
    ToolDefinition {
        id: "cargo",
        name: "Cargo",
        category: InventoryCategory::PackageManager,
        commands: &["cargo"],
        known_paths: &[],
        version_arg: "--version",
        version_regex: r"cargo (\d+\.\d+\.\d+)",
        port: None,
    },
    // Dev Tools
    ToolDefinition {
        id: "git",
        name: "Git",
        category: InventoryCategory::DevTool,
        commands: &["git"],
        known_paths: &[
            "C:\\Program Files\\Git\\bin\\git.exe",
            "C:\\Program Files (x86)\\Git\\bin\\git.exe",
        ],
        version_arg: "--version",
        version_regex: r"git version (\d+\.\d+\.\d+)",
        port: None,
    },
    ToolDefinition {
        id: "docker",
        name: "Docker",
        category: InventoryCategory::DevTool,
        commands: &["docker"],
        known_paths: &[
            "C:\\Program Files\\Docker\\Docker\\resources\\bin\\docker.exe",
        ],
        version_arg: "--version",
        version_regex: r"Docker version (\d+\.\d+\.\d+)",
        port: None,
    },
    ToolDefinition {
        id: "vscode",
        name: "VS Code",
        category: InventoryCategory::DevTool,
        commands: &["code"],
        known_paths: &[
            "C:\\Program Files\\Microsoft VS Code\\bin\\code.cmd",
            "C:\\Users\\*\\AppData\\Local\\Programs\\Microsoft VS Code\\bin\\code.cmd",
        ],
        version_arg: "--version",
        version_regex: r"(\d+\.\d+\.\d+)",
        port: None,
    },
    ToolDefinition {
        id: "gh",
        name: "GitHub CLI",
        category: InventoryCategory::DevTool,
        commands: &["gh"],
        known_paths: &[
            "C:\\Program Files\\GitHub CLI\\gh.exe",
        ],
        version_arg: "--version",
        version_regex: r"gh version (\d+\.\d+\.\d+)",
        port: None,
    },
    ToolDefinition {
        id: "curl",
        name: "curl",
        category: InventoryCategory::DevTool,
        commands: &["curl"],
        known_paths: &[
            "C:\\Windows\\System32\\curl.exe",
        ],
        version_arg: "--version",
        version_regex: r"curl (\d+\.\d+\.\d+)",
        port: None,
    },
];

/// Definition of a web application to scan for (non-executable, directory-based)
struct WebAppDefinition {
    id: &'static str,
    name: &'static str,
    category: InventoryCategory,
    /// Known directory paths where this web app might be installed
    known_dirs: &'static [&'static str],
    /// File to read for version detection (relative to the install directory)
    version_file: &'static str,
    /// Regex to extract version from the version file content
    version_regex: &'static str,
    port: Option<u16>,
}

const WEBAPP_DEFINITIONS: &[WebAppDefinition] = &[
    WebAppDefinition {
        id: "phpmyadmin",
        name: "phpMyAdmin",
        category: InventoryCategory::DevTool,
        known_dirs: &[
            "C:\\xampp\\phpMyAdmin",
            "D:\\xampp\\phpMyAdmin",
            "C:\\wamp64\\apps\\phpmyadmin",
            "C:\\laragon\\etc\\apps\\phpMyAdmin",
            "C:\\DevPort\\tools\\phpmyadmin",
        ],
        version_file: "VERSION",
        version_regex: r"(\d+\.\d+\.\d+)",
        port: None,
    },
];

pub struct InventoryScanner;

impl InventoryScanner {
    /// Find executable using the 'where' command on Windows
    fn find_executable_in_path(cmd: &str) -> Option<String> {
        #[cfg(windows)]
        let output = Command::new("where")
            .arg(cmd)
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .ok()?;

        #[cfg(not(windows))]
        let output = Command::new("which")
            .arg(cmd)
            .output()
            .ok()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let path = stdout.lines().next()?.trim().to_string();
            if Path::new(&path).exists() {
                return Some(path);
            }
        }
        None
    }

    /// Find executable by checking known paths
    fn find_executable_in_known_paths(known_paths: &[&str]) -> Option<String> {
        for path in known_paths {
            // Handle wildcard paths (e.g., C:\Users\*\AppData\...)
            if path.contains('*') {
                if let Some(found) = Self::expand_wildcard_path(path) {
                    return Some(found);
                }
            } else if Path::new(path).exists() {
                return Some(path.to_string());
            }
        }
        None
    }

    /// Expand wildcard paths (simple implementation for common patterns)
    fn expand_wildcard_path(pattern: &str) -> Option<String> {
        // Handle C:\Users\*\... pattern
        if pattern.starts_with("C:\\Users\\*\\") {
            let suffix = &pattern[12..]; // Remove "C:\Users\*\"
            if let Ok(entries) = std::fs::read_dir("C:\\Users") {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        let candidate = entry.path().join(suffix);
                        if candidate.exists() {
                            return Some(candidate.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
        None
    }

    /// Get version using a command with specified arguments
    fn get_version(executable_path: &str, args: &str, regex_pattern: &str) -> Option<String> {
        #[cfg(windows)]
        let output = Command::new(executable_path)
            .arg(args)
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .ok()?;

        #[cfg(not(windows))]
        let output = Command::new(executable_path)
            .arg(args)
            .output()
            .ok()?;

        let combined_output = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        if let Ok(re) = Regex::new(regex_pattern) {
            if let Some(caps) = re.captures(&combined_output) {
                return caps.get(1).map(|m| m.as_str().to_string());
            }
        }
        None
    }

    /// Determine install source based on executable path
    fn detect_install_source(path: &str) -> InstallSource {
        let path_lower = path.to_lowercase();

        if path_lower.contains("devport") {
            InstallSource::DevPort
        } else if path_lower.contains("xampp") {
            InstallSource::Xampp
        } else if path_lower.contains("laragon") {
            InstallSource::Laragon
        } else if path_lower.contains("wamp") {
            InstallSource::Wamp
        } else if path_lower.contains("scoop") {
            InstallSource::Scoop
        } else if path_lower.contains("chocolatey") || path_lower.contains("choco") {
            InstallSource::Chocolatey
        } else if path_lower.contains("program files")
            || path_lower.contains("windows")
            || path_lower.contains("system32")
        {
            InstallSource::System
        } else if path_lower.contains("appdata") {
            InstallSource::Manual
        } else {
            InstallSource::Unknown
        }
    }

    /// Scan a web application (directory-based, no executable)
    fn scan_webapp(def: &WebAppDefinition) -> InventoryItem {
        let mut item = InventoryItem::new(
            def.id.to_string(),
            def.name.to_string(),
            def.category.clone(),
        );
        item.port = def.port;

        for dir in def.known_dirs {
            let dir_path = Path::new(dir);
            if dir_path.is_dir() {
                item.is_installed = true;
                item.executable_path = Some(dir.to_string());
                item.install_source = Self::detect_install_source(dir);

                // Try to read version from version file
                let version_file_path = dir_path.join(def.version_file);
                if let Ok(content) = std::fs::read_to_string(&version_file_path) {
                    if let Ok(re) = Regex::new(def.version_regex) {
                        if let Some(caps) = re.captures(&content) {
                            item.version = caps.get(1).map(|m| m.as_str().to_string());
                        }
                    }
                }
                return item;
            }
        }

        item
    }

    /// MySQL/MariaDB known paths for scanning
    const MYSQL_KNOWN_PATHS: &'static [&'static str] = &[
        "C:\\xampp\\mysql\\bin\\mysql.exe",
        "D:\\xampp\\mysql\\bin\\mysql.exe",
        "C:\\laragon\\bin\\mariadb\\mariadb-10.9.3-winx64\\bin\\mysql.exe",
        "C:\\laragon\\bin\\mysql\\mysql-8.0.30-winx64\\bin\\mysql.exe",
        "C:\\wamp64\\bin\\mariadb\\mariadb10.11.4\\bin\\mysql.exe",
        "C:\\DevPort\\runtime\\mariadb\\bin\\mysql.exe",
        "C:\\Program Files\\MySQL\\MySQL Server 8.0\\bin\\mysql.exe",
    ];

    /// Detect MySQL or MariaDB by running the executable and checking the version output.
    /// Returns (mariadb_item, mysql_item) - one will be detected, the other not.
    fn scan_mysql_or_mariadb() -> (InventoryItem, InventoryItem) {
        let mut mariadb_item = InventoryItem::new(
            "mariadb".to_string(),
            "MariaDB".to_string(),
            InventoryCategory::Database,
        );
        mariadb_item.port = Some(3306);

        let mut mysql_item = InventoryItem::new(
            "mysql".to_string(),
            "MySQL".to_string(),
            InventoryCategory::Database,
        );
        mysql_item.port = Some(3306);

        // Find the mysql executable
        let exe_path = Self::find_executable_in_path("mysql")
            .or_else(|| Self::find_executable_in_path("mysqld"))
            .or_else(|| Self::find_executable_in_known_paths(Self::MYSQL_KNOWN_PATHS));

        let Some(path) = exe_path else {
            return (mariadb_item, mysql_item);
        };

        // Run --version and check whether output contains "MariaDB"
        let version_output = {
            #[cfg(windows)]
            let output = Command::new(&path)
                .arg("--version")
                .creation_flags(CREATE_NO_WINDOW)
                .output();

            #[cfg(not(windows))]
            let output = Command::new(&path)
                .arg("--version")
                .output();

            match output {
                Ok(o) => format!(
                    "{}\n{}",
                    String::from_utf8_lossy(&o.stdout),
                    String::from_utf8_lossy(&o.stderr)
                ),
                Err(_) => return (mariadb_item, mysql_item),
            }
        };

        let is_mariadb = version_output.contains("MariaDB") || version_output.contains("mariadb");
        let source = Self::detect_install_source(&path);

        // Extract version
        let version_regex = if is_mariadb {
            r"MariaDB[^\d]*(\d+\.\d+\.\d+)"
        } else {
            r"(?:mysql|Ver)[^\d]*(\d+\.\d+\.\d+)"
        };
        let version = if let Ok(re) = Regex::new(version_regex) {
            re.captures(&version_output)
                .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
        } else {
            None
        };

        if is_mariadb {
            mariadb_item.is_installed = true;
            mariadb_item.executable_path = Some(path);
            mariadb_item.install_source = source;
            mariadb_item.version = version;
        } else {
            mysql_item.is_installed = true;
            mysql_item.executable_path = Some(path);
            mysql_item.install_source = source;
            mysql_item.version = version;
        }

        (mariadb_item, mysql_item)
    }

    /// Scan a single tool
    fn scan_tool(def: &ToolDefinition) -> InventoryItem {
        let mut item = InventoryItem::new(
            def.id.to_string(),
            def.name.to_string(),
            def.category.clone(),
        );
        item.port = def.port;

        // First, try to find in PATH
        for cmd in def.commands {
            if let Some(path) = Self::find_executable_in_path(cmd) {
                item.is_installed = true;
                item.executable_path = Some(path.clone());
                item.install_source = Self::detect_install_source(&path);
                item.version = Self::get_version(&path, def.version_arg, def.version_regex);
                return item;
            }
        }

        // If not found in PATH, check known installation paths
        if let Some(path) = Self::find_executable_in_known_paths(def.known_paths) {
            item.is_installed = true;
            item.executable_path = Some(path.clone());
            item.install_source = Self::detect_install_source(&path);
            item.version = Self::get_version(&path, def.version_arg, def.version_regex);
        }

        item
    }

    /// Check if a port is actively listening (i.e. service is running)
    fn is_port_listening(port: u16) -> bool {
        !PortScanner::is_port_available(port)
    }

    /// Set is_running for all installed items that have a port defined
    fn detect_running_by_port(result: &mut InventoryResult) {
        let all_categories: [&mut Vec<InventoryItem>; 7] = [
            &mut result.runtimes,
            &mut result.web_servers,
            &mut result.databases,
            &mut result.build_tools,
            &mut result.frameworks,
            &mut result.package_managers,
            &mut result.dev_tools,
        ];

        for items in all_categories {
            for item in items.iter_mut() {
                if item.is_installed {
                    if let Some(port) = item.port {
                        item.is_running = Self::is_port_listening(port);
                    }
                }
            }
        }
    }

    /// Scan all tools and return the result
    pub async fn scan_all() -> InventoryResult {
        let start = Instant::now();
        let mut result = InventoryResult::new();

        // Scan all CLI tools
        for def in TOOL_DEFINITIONS {
            let item = Self::scan_tool(def);
            match def.category {
                InventoryCategory::Runtime => result.runtimes.push(item),
                InventoryCategory::WebServer => result.web_servers.push(item),
                InventoryCategory::Database => result.databases.push(item),
                InventoryCategory::BuildTool => result.build_tools.push(item),
                InventoryCategory::Framework => result.frameworks.push(item),
                InventoryCategory::PackageManager => result.package_managers.push(item),
                InventoryCategory::DevTool => result.dev_tools.push(item),
            }
        }

        // Scan MySQL/MariaDB separately (engine detection)
        let (mariadb_item, mysql_item) = Self::scan_mysql_or_mariadb();
        result.databases.push(mariadb_item);
        result.databases.push(mysql_item);

        // Scan web applications (directory-based)
        for def in WEBAPP_DEFINITIONS {
            let item = Self::scan_webapp(def);
            match def.category {
                InventoryCategory::Runtime => result.runtimes.push(item),
                InventoryCategory::WebServer => result.web_servers.push(item),
                InventoryCategory::Database => result.databases.push(item),
                InventoryCategory::BuildTool => result.build_tools.push(item),
                InventoryCategory::Framework => result.frameworks.push(item),
                InventoryCategory::PackageManager => result.package_managers.push(item),
                InventoryCategory::DevTool => result.dev_tools.push(item),
            }
        }

        // Detect running state by checking if defined ports are in use
        Self::detect_running_by_port(&mut result);

        result.scanned_at = Local::now().to_rfc3339();
        result.scan_duration_ms = start.elapsed().as_millis() as u64;

        result
    }

    /// Refresh a single inventory item by ID
    pub async fn refresh_item(id: &str) -> Option<InventoryItem> {
        let mut item = if id == "mariadb" || id == "mysql" {
            // Handle mysql/mariadb specially
            let (mariadb_item, mysql_item) = Self::scan_mysql_or_mariadb();
            Some(if id == "mariadb" { mariadb_item } else { mysql_item })
        } else if let Some(def) = TOOL_DEFINITIONS.iter().find(|d| d.id == id) {
            Some(Self::scan_tool(def))
        } else if let Some(def) = WEBAPP_DEFINITIONS.iter().find(|d| d.id == id) {
            Some(Self::scan_webapp(def))
        } else {
            None
        };

        // Detect running state by port
        if let Some(ref mut item) = item {
            if item.is_installed {
                if let Some(port) = item.port {
                    item.is_running = Self::is_port_listening(port);
                }
            }
        }

        item
    }
}
