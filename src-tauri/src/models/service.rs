use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    Running,
    Stopped,
    Error,
    Unhealthy,
    NotInstalled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigFile {
    pub name: String,
    pub path: String,
    pub description: String,
}

impl Default for ServiceStatus {
    fn default() -> Self {
        ServiceStatus::Stopped
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceType {
    Webserver,
    Database,
    Runtime,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthCheckType {
    Http,
    Tcp,
    Process,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckConfig {
    pub check_type: HealthCheckType,
    pub endpoint: Option<String>,
    pub interval: u64,
    pub timeout: u64,
    pub retries: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            check_type: HealthCheckType::Process,
            endpoint: None,
            interval: 5000,
            timeout: 2000,
            retries: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogConfig {
    pub stdout_path: String,
    pub stderr_path: String,
    pub max_size: String,
    pub max_files: u32,
    pub rotation_policy: String,
    pub retention_days: u32,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            stdout_path: String::new(),
            stderr_path: String::new(),
            max_size: "50MB".to_string(),
            max_files: 5,
            rotation_policy: "size".to_string(),
            retention_days: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub id: String,
    pub name: String,
    pub service_type: ServiceType,
    pub executable: String,
    pub args: Vec<String>,
    pub work_dir: String,
    pub env: HashMap<String, String>,
    pub port: u16,
    pub additional_ports: Vec<u16>,
    pub status: ServiceStatus,
    pub pid: Option<u32>,
    pub auto_start: bool,
    pub auto_restart: bool,
    pub restart_delay: u64,
    pub max_restarts: u32,
    pub restart_count: u32,
    pub health_check: HealthCheckConfig,
    pub log_config: LogConfig,
    pub depends_on: Vec<String>,
    pub last_started: Option<String>,
    pub last_stopped: Option<String>,
    pub error_message: Option<String>,
    pub installed: bool,
    pub config_files: Vec<ConfigFile>,
}

impl Service {
    pub fn new(id: String, name: String, service_type: ServiceType) -> Self {
        Self {
            id,
            name,
            service_type,
            executable: String::new(),
            args: Vec::new(),
            work_dir: String::new(),
            env: HashMap::new(),
            port: 0,
            additional_ports: Vec::new(),
            status: ServiceStatus::Stopped,
            pid: None,
            auto_start: false,
            auto_restart: true,
            restart_delay: 3000,
            max_restarts: 5,
            restart_count: 0,
            health_check: HealthCheckConfig::default(),
            log_config: LogConfig::default(),
            depends_on: Vec::new(),
            last_started: None,
            last_stopped: None,
            error_message: None,
            installed: false,
            config_files: Vec::new(),
        }
    }

    /// Check if the service executable exists on disk
    pub fn check_installed(&mut self) {
        self.installed = Path::new(&self.executable).exists();
        if !self.installed {
            self.status = ServiceStatus::NotInstalled;
        }
    }

    /// 가능한 Apache 설치 경로들을 검색
    fn find_apache_path() -> Option<(String, String, u16)> {
        // (base_path, executable, default_port)
        let possible_paths = [
            // XAMPP (가장 일반적)
            ("C:\\xampp\\apache", "C:\\xampp\\apache\\bin\\httpd.exe", 80u16),
            // XAMPP 다른 드라이브
            ("D:\\xampp\\apache", "D:\\xampp\\apache\\bin\\httpd.exe", 80),
            // DevPort 커스텀 경로
            ("C:\\DevPort\\runtime\\apache", "C:\\DevPort\\runtime\\apache\\bin\\httpd.exe", 8080),
            // Laragon
            ("C:\\laragon\\bin\\apache", "C:\\laragon\\bin\\apache\\bin\\httpd.exe", 80),
            // WampServer
            ("C:\\wamp64\\bin\\apache", "C:\\wamp64\\bin\\apache\\apache2.4.54.2\\bin\\httpd.exe", 80),
        ];

        for (base, exe, port) in possible_paths {
            if Path::new(exe).exists() {
                return Some((base.to_string(), exe.to_string(), port));
            }
        }
        None
    }

    /// 가능한 MySQL/MariaDB 설치 경로들을 검색
    /// 반환값: (base_path, executable, config_path)
    fn find_mysql_path() -> Option<(String, String, String)> {
        // (base_path, executable, primary_config)
        let possible_paths = [
            // XAMPP (가장 일반적) - my.ini가 bin 폴더에 있음
            ("C:\\xampp\\mysql", "C:\\xampp\\mysql\\bin\\mysqld.exe", "C:\\xampp\\mysql\\bin\\my.ini"),
            // XAMPP 다른 드라이브
            ("D:\\xampp\\mysql", "D:\\xampp\\mysql\\bin\\mysqld.exe", "D:\\xampp\\mysql\\bin\\my.ini"),
            // DevPort 커스텀 경로
            ("C:\\DevPort\\runtime\\mariadb", "C:\\DevPort\\runtime\\mariadb\\bin\\mysqld.exe", "C:\\DevPort\\runtime\\mariadb\\data\\my.ini"),
            // Laragon MariaDB
            ("C:\\laragon\\bin\\mariadb", "C:\\laragon\\bin\\mariadb\\bin\\mysqld.exe", "C:\\laragon\\bin\\mariadb\\my.ini"),
            // WampServer
            ("C:\\wamp64\\bin\\mariadb", "C:\\wamp64\\bin\\mariadb\\mariadb10.11.4\\bin\\mysqld.exe", "C:\\wamp64\\bin\\mariadb\\mariadb10.11.4\\my.ini"),
        ];

        for (base, exe, config) in possible_paths {
            if Path::new(exe).exists() {
                // config 파일이 있으면 해당 경로, 없으면 다른 위치에서 검색
                let config_path = if Path::new(config).exists() {
                    config.to_string()
                } else {
                    // 다른 가능한 위치에서 my.ini 검색
                    let alt_configs = [
                        format!("{}\\bin\\my.ini", base),
                        format!("{}\\my.ini", base),
                        format!("{}\\data\\my.ini", base),
                    ];
                    alt_configs.into_iter()
                        .find(|path| Path::new(path).exists())
                        .unwrap_or_default()
                };
                return Some((base.to_string(), exe.to_string(), config_path));
            }
        }
        None
    }

    pub fn apache() -> Self {
        let mut service = Self::new(
            "apache".to_string(),
            "Apache".to_string(),
            ServiceType::Webserver,
        );

        // 설치된 Apache 경로 자동 감지
        if let Some((base_path, exe_path, port)) = Self::find_apache_path() {
            service.executable = exe_path;
            service.work_dir = base_path.clone();
            service.port = port;
            service.health_check = HealthCheckConfig {
                check_type: HealthCheckType::Http,
                endpoint: Some(format!("http://localhost:{}/", port)),
                interval: 5000,
                timeout: 2000,
                retries: 2,
            };
            service.log_config = LogConfig {
                stdout_path: format!("{}\\logs\\access.log", base_path),
                stderr_path: format!("{}\\logs\\error.log", base_path),
                ..Default::default()
            };
            service.config_files = vec![
                ConfigFile {
                    name: "httpd.conf".to_string(),
                    path: format!("{}\\conf\\httpd.conf", base_path),
                    description: "Apache 메인 설정".to_string(),
                },
                ConfigFile {
                    name: "httpd-vhosts.conf".to_string(),
                    path: format!("{}\\conf\\extra\\httpd-vhosts.conf", base_path),
                    description: "가상 호스트 설정".to_string(),
                },
            ];
        } else {
            // 기본값 (설치 안됨 상태)
            service.executable = "C:\\xampp\\apache\\bin\\httpd.exe".to_string();
            service.work_dir = "C:\\xampp\\apache".to_string();
            service.port = 80;
            service.health_check = HealthCheckConfig {
                check_type: HealthCheckType::Http,
                endpoint: Some("http://localhost:80/".to_string()),
                interval: 5000,
                timeout: 2000,
                retries: 2,
            };
        }

        service.check_installed();
        service
    }

    pub fn mariadb() -> Self {
        let mut service = Self::new(
            "mariadb".to_string(),
            "MySQL/MariaDB".to_string(),
            ServiceType::Database,
        );

        // 설치된 MySQL/MariaDB 경로 자동 감지
        if let Some((base_path, exe_path, config_path)) = Self::find_mysql_path() {
            service.executable = exe_path;

            // 설정 파일 경로를 명시적으로 지정 (XAMPP에서 필수)
            // --defaults-file must be the FIRST argument for mysqld
            let mut args = Vec::new();
            if !config_path.is_empty() {
                args.push(format!("--defaults-file={}", config_path));
            }
            args.push("--console".to_string());
            service.args = args;

            service.work_dir = base_path.clone();
            service.port = 3306;
            service.health_check = HealthCheckConfig {
                check_type: HealthCheckType::Tcp,
                endpoint: Some("localhost:3306".to_string()),
                interval: 5000,
                timeout: 2000,
                retries: 2,
            };
            service.log_config = LogConfig {
                stdout_path: format!("{}\\data\\mysql.log", base_path),
                stderr_path: format!("{}\\data\\mysql_error.log", base_path),
                ..Default::default()
            };

            // config_files에 사용된 my.ini 경로 저장
            let my_ini_path = if !config_path.is_empty() {
                config_path
            } else if Path::new(&format!("{}\\bin\\my.ini", base_path)).exists() {
                format!("{}\\bin\\my.ini", base_path)
            } else if Path::new(&format!("{}\\my.ini", base_path)).exists() {
                format!("{}\\my.ini", base_path)
            } else {
                format!("{}\\data\\my.ini", base_path)
            };
            service.config_files = vec![
                ConfigFile {
                    name: "my.ini".to_string(),
                    path: my_ini_path,
                    description: "MySQL/MariaDB 메인 설정".to_string(),
                },
            ];
        } else {
            // 기본값 (설치 안됨 상태)
            service.executable = "C:\\xampp\\mysql\\bin\\mysqld.exe".to_string();
            service.args = vec!["--console".to_string()];
            service.work_dir = "C:\\xampp\\mysql".to_string();
            service.port = 3306;
            service.health_check = HealthCheckConfig {
                check_type: HealthCheckType::Tcp,
                endpoint: Some("localhost:3306".to_string()),
                interval: 5000,
                timeout: 2000,
                retries: 2,
            };
        }

        service.check_installed();
        service
    }

    pub fn is_running(&self) -> bool {
        matches!(self.status, ServiceStatus::Running)
    }

    pub fn is_healthy(&self) -> bool {
        matches!(self.status, ServiceStatus::Running)
    }

    pub fn can_restart(&self) -> bool {
        self.auto_restart && self.restart_count < self.max_restarts
    }

    pub fn reset_restart_count(&mut self) {
        self.restart_count = 0;
    }

    pub fn increment_restart_count(&mut self) {
        self.restart_count += 1;
    }
}
