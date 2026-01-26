use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// Base paths for DevPort runtime and tools
pub const DEVPORT_BASE_PATH: &str = "C:\\DevPort";
pub const RUNTIME_BASE_PATH: &str = "C:\\DevPort\\runtime";
pub const TOOLS_BASE_PATH: &str = "C:\\DevPort\\tools";

// Individual runtime paths
pub const APACHE_PATH: &str = "C:\\DevPort\\runtime\\apache";
pub const MARIADB_PATH: &str = "C:\\DevPort\\runtime\\mariadb";
pub const PHP_PATH: &str = "C:\\DevPort\\runtime\\php";
pub const NODEJS_PATH: &str = "C:\\DevPort\\runtime\\nodejs";
pub const GIT_PATH: &str = "C:\\DevPort\\runtime\\git";

// Tool paths
pub const PHPMYADMIN_PATH: &str = "C:\\DevPort\\tools\\phpmyadmin";
pub const COMPOSER_PATH: &str = "C:\\DevPort\\tools\\composer";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeType {
    Apache,
    MariaDB,
    PHP,
    NodeJS,
    Git,
    PhpMyAdmin,
    Composer,
}

impl RuntimeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RuntimeType::Apache => "apache",
            RuntimeType::MariaDB => "mariadb",
            RuntimeType::PHP => "php",
            RuntimeType::NodeJS => "nodejs",
            RuntimeType::Git => "git",
            RuntimeType::PhpMyAdmin => "phpmyadmin",
            RuntimeType::Composer => "composer",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            RuntimeType::Apache => "Apache HTTP Server",
            RuntimeType::MariaDB => "MariaDB",
            RuntimeType::PHP => "PHP",
            RuntimeType::NodeJS => "Node.js",
            RuntimeType::Git => "Git",
            RuntimeType::PhpMyAdmin => "phpMyAdmin",
            RuntimeType::Composer => "Composer",
        }
    }

    pub fn all_runtimes() -> Vec<RuntimeType> {
        vec![
            RuntimeType::Apache,
            RuntimeType::MariaDB,
            RuntimeType::PHP,
            RuntimeType::NodeJS,
            RuntimeType::Git,
        ]
    }

    pub fn all_tools() -> Vec<RuntimeType> {
        vec![RuntimeType::PhpMyAdmin, RuntimeType::Composer]
    }

    pub fn all() -> Vec<RuntimeType> {
        vec![
            RuntimeType::Apache,
            RuntimeType::MariaDB,
            RuntimeType::PHP,
            RuntimeType::NodeJS,
            RuntimeType::Git,
            RuntimeType::PhpMyAdmin,
            RuntimeType::Composer,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleConfig {
    pub runtime_type: RuntimeType,
    pub base_path: String,
    pub executable_path: String,
    pub version_command: String,
    pub version_args: Vec<String>,
    pub required_files: Vec<String>,
}

impl BundleConfig {
    pub fn apache() -> Self {
        BundleConfig {
            runtime_type: RuntimeType::Apache,
            base_path: APACHE_PATH.to_string(),
            executable_path: format!("{}\\bin\\httpd.exe", APACHE_PATH),
            version_command: format!("{}\\bin\\httpd.exe", APACHE_PATH),
            version_args: vec!["-v".to_string()],
            required_files: vec![
                "bin\\httpd.exe".to_string(),
                "conf\\httpd.conf".to_string(),
                "modules\\mod_php.so".to_string(),
            ],
        }
    }

    pub fn mariadb() -> Self {
        BundleConfig {
            runtime_type: RuntimeType::MariaDB,
            base_path: MARIADB_PATH.to_string(),
            executable_path: format!("{}\\bin\\mysqld.exe", MARIADB_PATH),
            version_command: format!("{}\\bin\\mysql.exe", MARIADB_PATH),
            version_args: vec!["--version".to_string()],
            required_files: vec![
                "bin\\mysqld.exe".to_string(),
                "bin\\mysql.exe".to_string(),
                "bin\\mysqladmin.exe".to_string(),
            ],
        }
    }

    pub fn php() -> Self {
        BundleConfig {
            runtime_type: RuntimeType::PHP,
            base_path: PHP_PATH.to_string(),
            executable_path: format!("{}\\php.exe", PHP_PATH),
            version_command: format!("{}\\php.exe", PHP_PATH),
            version_args: vec!["-v".to_string()],
            required_files: vec![
                "php.exe".to_string(),
                "php.ini".to_string(),
                "ext\\php_mysqli.dll".to_string(),
            ],
        }
    }

    pub fn nodejs() -> Self {
        BundleConfig {
            runtime_type: RuntimeType::NodeJS,
            base_path: NODEJS_PATH.to_string(),
            executable_path: format!("{}\\node.exe", NODEJS_PATH),
            version_command: format!("{}\\node.exe", NODEJS_PATH),
            version_args: vec!["--version".to_string()],
            required_files: vec![
                "node.exe".to_string(),
                "npm.cmd".to_string(),
                "npx.cmd".to_string(),
            ],
        }
    }

    pub fn git() -> Self {
        BundleConfig {
            runtime_type: RuntimeType::Git,
            base_path: GIT_PATH.to_string(),
            executable_path: format!("{}\\bin\\git.exe", GIT_PATH),
            version_command: format!("{}\\bin\\git.exe", GIT_PATH),
            version_args: vec!["--version".to_string()],
            required_files: vec![
                "bin\\git.exe".to_string(),
                "bin\\bash.exe".to_string(),
            ],
        }
    }

    pub fn phpmyadmin() -> Self {
        BundleConfig {
            runtime_type: RuntimeType::PhpMyAdmin,
            base_path: PHPMYADMIN_PATH.to_string(),
            executable_path: String::new(), // phpMyAdmin is PHP-based, no executable
            version_command: String::new(),
            version_args: vec![],
            required_files: vec![
                "index.php".to_string(),
                "config.inc.php".to_string(),
            ],
        }
    }

    pub fn composer() -> Self {
        BundleConfig {
            runtime_type: RuntimeType::Composer,
            base_path: COMPOSER_PATH.to_string(),
            executable_path: format!("{}\\composer.phar", COMPOSER_PATH),
            version_command: format!("{}\\php.exe", PHP_PATH), // Uses PHP to run
            version_args: vec![
                format!("{}\\composer.phar", COMPOSER_PATH),
                "--version".to_string(),
            ],
            required_files: vec!["composer.phar".to_string()],
        }
    }

    pub fn for_runtime(runtime_type: RuntimeType) -> Self {
        match runtime_type {
            RuntimeType::Apache => Self::apache(),
            RuntimeType::MariaDB => Self::mariadb(),
            RuntimeType::PHP => Self::php(),
            RuntimeType::NodeJS => Self::nodejs(),
            RuntimeType::Git => Self::git(),
            RuntimeType::PhpMyAdmin => Self::phpmyadmin(),
            RuntimeType::Composer => Self::composer(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeInfo {
    pub runtime_type: RuntimeType,
    pub name: String,
    pub version: Option<String>,
    pub is_installed: bool,
    pub base_path: String,
    pub executable_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleStatus {
    pub runtime_type: RuntimeType,
    pub name: String,
    pub is_installed: bool,
    pub is_valid: bool,
    pub missing_files: Vec<String>,
    pub base_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundlePaths {
    pub devport_base: String,
    pub runtime_base: String,
    pub tools_base: String,
    pub apache: String,
    pub mariadb: String,
    pub php: String,
    pub nodejs: String,
    pub git: String,
    pub phpmyadmin: String,
    pub composer: String,
}

impl Default for BundlePaths {
    fn default() -> Self {
        BundlePaths {
            devport_base: DEVPORT_BASE_PATH.to_string(),
            runtime_base: RUNTIME_BASE_PATH.to_string(),
            tools_base: TOOLS_BASE_PATH.to_string(),
            apache: APACHE_PATH.to_string(),
            mariadb: MARIADB_PATH.to_string(),
            php: PHP_PATH.to_string(),
            nodejs: NODEJS_PATH.to_string(),
            git: GIT_PATH.to_string(),
            phpmyadmin: PHPMYADMIN_PATH.to_string(),
            composer: COMPOSER_PATH.to_string(),
        }
    }
}

pub struct Bundler;

impl Bundler {
    /// Check if a specific file exists relative to the base path
    fn file_exists(base_path: &str, relative_path: &str) -> bool {
        let full_path = PathBuf::from(base_path).join(relative_path);
        full_path.exists() && full_path.is_file()
    }

    /// Verify bundle integrity for a specific runtime
    pub fn verify_runtime_integrity(runtime_type: RuntimeType) -> BundleStatus {
        let config = BundleConfig::for_runtime(runtime_type);
        let base_path = Path::new(&config.base_path);

        let mut missing_files = Vec::new();
        let base_exists = base_path.exists() && base_path.is_dir();

        if base_exists {
            for file in &config.required_files {
                if !Self::file_exists(&config.base_path, file) {
                    missing_files.push(file.clone());
                }
            }
        } else {
            missing_files = config.required_files.clone();
        }

        let is_valid = base_exists && missing_files.is_empty();

        BundleStatus {
            runtime_type,
            name: runtime_type.display_name().to_string(),
            is_installed: base_exists,
            is_valid,
            missing_files,
            base_path: config.base_path,
        }
    }

    /// Verify bundle integrity for all runtimes
    pub fn verify_all_integrity() -> Vec<BundleStatus> {
        RuntimeType::all()
            .into_iter()
            .map(Self::verify_runtime_integrity)
            .collect()
    }

    /// Get version string for a runtime by executing its version command
    pub async fn get_runtime_version(runtime_type: RuntimeType) -> Option<String> {
        let config = BundleConfig::for_runtime(runtime_type);

        // phpMyAdmin doesn't have a version command, read from VERSION file or index.php
        if runtime_type == RuntimeType::PhpMyAdmin {
            return Self::get_phpmyadmin_version();
        }

        if config.version_command.is_empty() {
            return None;
        }

        // Check if executable exists first
        if !Path::new(&config.version_command).exists() {
            return None;
        }

        let output = tokio::process::Command::new(&config.version_command)
            .args(&config.version_args)
            .output()
            .await
            .ok()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let combined = format!("{}{}", stdout, stderr);

            // Extract version from output
            Self::parse_version(&combined, runtime_type)
        } else {
            None
        }
    }

    /// Parse version string from command output based on runtime type
    fn parse_version(output: &str, runtime_type: RuntimeType) -> Option<String> {
        let first_line = output.lines().next()?;

        match runtime_type {
            RuntimeType::Apache => {
                // "Server version: Apache/2.4.58 (Win64)"
                first_line
                    .split('/')
                    .nth(1)
                    .and_then(|s| s.split_whitespace().next())
                    .map(|s| s.to_string())
            }
            RuntimeType::MariaDB => {
                // "mysql  Ver 15.1 Distrib 11.2.2-MariaDB, for Win64 (AMD64)"
                first_line
                    .split("Distrib ")
                    .nth(1)
                    .and_then(|s| s.split('-').next())
                    .or_else(|| {
                        // Alternative: "Ver 15.1"
                        first_line.split("Ver ").nth(1).and_then(|s| s.split_whitespace().next())
                    })
                    .map(|s| s.to_string())
            }
            RuntimeType::PHP => {
                // "PHP 8.3.0 (cli) ..."
                first_line
                    .split_whitespace()
                    .nth(1)
                    .map(|s| s.to_string())
            }
            RuntimeType::NodeJS => {
                // "v20.10.0"
                first_line.trim_start_matches('v').to_string().into()
            }
            RuntimeType::Git => {
                // "git version 2.43.0.windows.1"
                first_line
                    .split_whitespace()
                    .nth(2)
                    .map(|s| s.split(".windows").next().unwrap_or(s).to_string())
            }
            RuntimeType::Composer => {
                // "Composer version 2.6.5 2023-10-06 10:11:52"
                first_line
                    .split_whitespace()
                    .nth(2)
                    .map(|s| s.to_string())
            }
            RuntimeType::PhpMyAdmin => {
                // Handled separately
                Some(first_line.to_string())
            }
        }
    }

    /// Get phpMyAdmin version from VERSION file or index.php
    fn get_phpmyadmin_version() -> Option<String> {
        let version_file = PathBuf::from(PHPMYADMIN_PATH).join("VERSION");
        if version_file.exists() {
            return std::fs::read_to_string(version_file)
                .ok()
                .map(|s| s.trim().to_string());
        }

        // Try reading from libraries/classes/Version.php or similar
        let version_php = PathBuf::from(PHPMYADMIN_PATH).join("libraries\\classes\\Version.php");
        if version_php.exists() {
            if let Ok(content) = std::fs::read_to_string(&version_php) {
                // Look for VERSION constant
                for line in content.lines() {
                    if line.contains("VERSION") && line.contains("'") {
                        if let Some(start) = line.find('\'') {
                            if let Some(end) = line[start + 1..].find('\'') {
                                return Some(line[start + 1..start + 1 + end].to_string());
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Get runtime info for a specific runtime
    pub async fn get_runtime_info(runtime_type: RuntimeType) -> RuntimeInfo {
        let config = BundleConfig::for_runtime(runtime_type);
        let status = Self::verify_runtime_integrity(runtime_type);
        let version = if status.is_valid {
            Self::get_runtime_version(runtime_type).await
        } else {
            None
        };

        RuntimeInfo {
            runtime_type,
            name: runtime_type.display_name().to_string(),
            version,
            is_installed: status.is_valid,
            base_path: config.base_path.clone(),
            executable_path: config.executable_path,
        }
    }

    /// Get runtime info for all runtimes
    pub async fn get_all_runtime_info() -> Vec<RuntimeInfo> {
        let mut infos = Vec::new();
        for runtime_type in RuntimeType::all() {
            infos.push(Self::get_runtime_info(runtime_type).await);
        }
        infos
    }

    /// Get bundle paths configuration
    pub fn get_bundle_paths() -> BundlePaths {
        BundlePaths::default()
    }

    /// Check if DevPort base directory exists
    pub fn is_devport_installed() -> bool {
        Path::new(DEVPORT_BASE_PATH).exists()
    }

    /// Get executable path for a runtime
    pub fn get_executable_path(runtime_type: RuntimeType) -> String {
        BundleConfig::for_runtime(runtime_type).executable_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundle_config_creation() {
        let apache = BundleConfig::apache();
        assert_eq!(apache.runtime_type, RuntimeType::Apache);
        assert!(apache.base_path.contains("apache"));
        assert!(apache.executable_path.contains("httpd.exe"));
    }

    #[test]
    fn test_runtime_type_display_names() {
        assert_eq!(RuntimeType::Apache.display_name(), "Apache HTTP Server");
        assert_eq!(RuntimeType::MariaDB.display_name(), "MariaDB");
        assert_eq!(RuntimeType::PHP.display_name(), "PHP");
    }

    #[test]
    fn test_bundle_paths_default() {
        let paths = BundlePaths::default();
        assert_eq!(paths.devport_base, "C:\\DevPort");
        assert_eq!(paths.apache, "C:\\DevPort\\runtime\\apache");
    }

    #[test]
    fn test_parse_version_apache() {
        let output = "Server version: Apache/2.4.58 (Win64)\nServer built:   Oct 19 2023";
        let version = Bundler::parse_version(output, RuntimeType::Apache);
        assert_eq!(version, Some("2.4.58".to_string()));
    }

    #[test]
    fn test_parse_version_php() {
        let output = "PHP 8.3.0 (cli) (built: Nov 21 2023 14:14:28) (NTS Visual C++ 2019 x64)";
        let version = Bundler::parse_version(output, RuntimeType::PHP);
        assert_eq!(version, Some("8.3.0".to_string()));
    }

    #[test]
    fn test_parse_version_nodejs() {
        let output = "v20.10.0\n";
        let version = Bundler::parse_version(output, RuntimeType::NodeJS);
        assert_eq!(version, Some("20.10.0".to_string()));
    }
}
