use crate::models::{Service, ServiceStatus, HealthCheckType};
use crate::services::log_manager::LogManager;
use crate::services::port_scanner::PortScanner;
use crate::services::process_manager::{kill_process_tree, kill_process_tree_silent, CREATE_NO_WINDOW};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread;
use tauri::{AppHandle, Emitter};
use tokio::time::{sleep, Duration};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;

pub struct ServiceManager {
    pub services: HashMap<String, Service>,
    pub processes: HashMap<String, Child>,
    pub log_manager: LogManager,
}

impl ServiceManager {
    pub fn new() -> Self {
        let mut services = HashMap::new();

        let apache = Service::apache();
        let mariadb = Service::mariadb();

        services.insert(apache.id.clone(), apache);
        services.insert(mariadb.id.clone(), mariadb);

        let mut manager = Self {
            services,
            processes: HashMap::new(),
            log_manager: LogManager::new(),
        };

        // Detect externally running services (e.g. started by XAMPP)
        manager.detect_external_processes();

        manager
    }

    /// Detect services that are already running externally (not started by DevPort).
    /// Uses port scanning to find PIDs occupying service ports.
    fn detect_external_processes(&mut self) {
        let port_infos = match PortScanner::scan_ports() {
            Ok(infos) => infos,
            Err(_) => return,
        };

        for service in self.services.values_mut() {
            if !service.installed || service.is_running() {
                continue;
            }

            // Check if the service's port is occupied
            if let Some(port_info) = port_infos.iter().find(|p| {
                p.port == service.port
                    && p.state.contains("LISTEN")
            }) {
                service.status = ServiceStatus::Running;
                service.pid = port_info.pid;

                let log_path = self.log_manager.get_log_path(&service.id, "stdout");
                let _ = self.log_manager.write_log(
                    &log_path,
                    &format!(
                        "Detected externally running {} on port {} (PID: {})",
                        service.name,
                        service.port,
                        port_info.pid.map(|p| p.to_string()).unwrap_or_else(|| "unknown".to_string())
                    ),
                );
            }
        }
    }

    pub fn get_services(&self) -> Vec<&Service> {
        self.services.values().collect()
    }

    pub fn get_service(&self, id: &str) -> Option<&Service> {
        self.services.get(id)
    }

    pub fn get_service_mut(&mut self, id: &str) -> Option<&mut Service> {
        self.services.get_mut(id)
    }

    pub async fn start_service(&mut self, id: &str, app_handle: Option<AppHandle>) -> Result<(), String> {
        let service = self.services.get(id).ok_or("Service not found")?;

        if service.is_running() {
            return Ok(());
        }

        let deps_to_start: Vec<String> = service
            .depends_on
            .iter()
            .filter(|dep_id| {
                self.services
                    .get(*dep_id)
                    .map(|dep| !dep.is_running())
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        for dep_id in deps_to_start {
            self.start_service_internal(&dep_id, app_handle.clone()).await?;
        }

        self.start_service_internal(id, app_handle).await
    }

    async fn start_service_internal(&mut self, id: &str, app_handle: Option<AppHandle>) -> Result<(), String> {
        let service = self.services.get(id).ok_or("Service not found")?;

        if service.is_running() {
            return Ok(());
        }

        // Pre-start security checks
        match id {
            "mariadb" => {
                if let Err(e) = self.ensure_mariadb_localhost_binding() {
                    eprintln!("Warning: MariaDB security check failed: {}", e);
                }
            }
            "apache" => {
                let port = service.port;
                if let Err(e) = self.validate_apache_listen_port(port) {
                    let service = self.services.get_mut(id).unwrap();
                    service.status = ServiceStatus::Error;
                    service.error_message = Some(e.clone());
                    return Err(e);
                }
                // Enforce phpMyAdmin "Require local" security
                if let Err(e) = self.enforce_phpmyadmin_require_local() {
                    eprintln!("Warning: phpMyAdmin security enforcement failed: {}", e);
                }
            }
            _ => {}
        }

        let service = self.services.get(id).unwrap();
        let executable = service.executable.clone();
        let args = service.args.clone();
        let work_dir = service.work_dir.clone();
        let env = service.env.clone();
        let service_id = id.to_string();
        let service_name = service.name.clone();

        if !std::path::Path::new(&executable).exists() {
            let service = self.services.get_mut(id).unwrap();
            service.status = ServiceStatus::Error;
            service.error_message = Some(format!("Executable not found: {}", executable));
            return Err(format!("Executable not found: {}", executable));
        }

        let mut cmd = Command::new(&executable);
        cmd.args(&args)
            .current_dir(&work_dir)
            .envs(&env)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        #[cfg(windows)]
        {
            cmd.creation_flags(CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP);
        }

        match cmd.spawn() {
            Ok(mut child) => {
                let pid = child.id();

                // Capture stdout in a separate thread
                if let Some(stdout) = child.stdout.take() {
                    let sid = service_id.clone();
                    let sname = service_name.clone();
                    let log_path = self.log_manager.get_log_path(&sid, "stdout");
                    let app = app_handle.clone();

                    thread::spawn(move || {
                        let reader = BufReader::new(stdout);
                        for line in reader.lines().map_while(Result::ok) {
                            // Write to log file
                            let _ = LogManager::append_line_to_file(&log_path, &line);

                            // Emit event to frontend
                            if let Some(ref app) = app {
                                let level = detect_log_level(&line);
                                let _ = app.emit("service-log", serde_json::json!({
                                    "serviceId": sid,
                                    "serviceName": sname,
                                    "line": line,
                                    "stream": "stdout",
                                    "level": level,
                                    "timestamp": chrono::Utc::now().to_rfc3339()
                                }));
                            }
                        }
                    });
                }

                // Capture stderr in a separate thread
                if let Some(stderr) = child.stderr.take() {
                    let sid = service_id.clone();
                    let sname = service_name.clone();
                    let log_path = self.log_manager.get_log_path(&sid, "stderr");
                    let app = app_handle.clone();

                    thread::spawn(move || {
                        let reader = BufReader::new(stderr);
                        for line in reader.lines().map_while(Result::ok) {
                            // Write to log file
                            let _ = LogManager::append_line_to_file(&log_path, &line);

                            // Emit event to frontend
                            if let Some(ref app) = app {
                                let level = detect_log_level(&line);
                                // For stderr, keep detected level (info for [Note], error for [ERROR], etc.)
                                // Only default to "warning" if no specific level detected
                                let final_level = if level == "debug" { "warning" } else { level };
                                let _ = app.emit("service-log", serde_json::json!({
                                    "serviceId": sid,
                                    "serviceName": sname,
                                    "line": line,
                                    "stream": "stderr",
                                    "level": final_level,
                                    "timestamp": chrono::Utc::now().to_rfc3339()
                                }));
                            }
                        }
                    });
                }

                self.processes.insert(id.to_string(), child);

                let service = self.services.get_mut(id).unwrap();
                service.status = ServiceStatus::Running;
                service.pid = Some(pid);
                service.last_started = Some(chrono::Utc::now().to_rfc3339());
                service.error_message = None;
                service.reset_restart_count();

                let log_path = self.log_manager.get_log_path(id, "stdout");
                let _ = self.log_manager.write_log(
                    &log_path,
                    &format!("Service {} started with PID {}", service.name, pid),
                );

                // Emit success message to frontend
                if let Some(ref app) = app_handle {
                    let _ = app.emit("service-log", serde_json::json!({
                        "serviceId": service_id,
                        "serviceName": service_name,
                        "line": format!("Service started successfully (PID: {})", pid),
                        "stream": "system",
                        "level": "success",
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }));
                }

                Ok(())
            }
            Err(e) => {
                let service = self.services.get_mut(id).unwrap();
                service.status = ServiceStatus::Error;
                service.error_message = Some(e.to_string());
                Err(e.to_string())
            }
        }
    }

    pub async fn stop_service(&mut self, id: &str) -> Result<(), String> {
        let service = self.services.get(id).ok_or("Service not found")?;

        if !service.is_running() {
            return Ok(());
        }

        let pid = service.pid;

        if let Some(mut child) = self.processes.remove(id) {
            // Process started by DevPort - kill via centralized function
            if let Some(pid) = pid {
                kill_process_tree_silent(pid);
            }
            let _ = child.wait();
        } else if let Some(pid) = pid {
            // Externally started process - kill by PID directly
            let result = kill_process_tree(pid);
            if !result.success {
                return Err(format!(
                    "Failed to stop external process (PID {}): {}",
                    pid,
                    result.error.unwrap_or_else(|| "Unknown error".to_string())
                ));
            }
        }

        if let Some(service) = self.services.get_mut(id) {
            service.status = ServiceStatus::Stopped;
            service.pid = None;
            service.last_stopped = Some(chrono::Utc::now().to_rfc3339());

            let log_path = self.log_manager.get_log_path(id, "stdout");
            let _ = self.log_manager.write_log(
                &log_path,
                &format!("Service {} stopped", service.name),
            );
        }

        Ok(())
    }

    pub async fn restart_service(&mut self, id: &str, app_handle: Option<AppHandle>) -> Result<(), String> {
        self.stop_service(id).await?;
        sleep(Duration::from_millis(1000)).await;
        self.start_service(id, app_handle).await
    }

    pub async fn check_health(&mut self, id: &str) -> ServiceStatus {
        let service = match self.services.get(id) {
            Some(s) => s.clone(),
            None => return ServiceStatus::Error,
        };

        if service.pid.is_none() {
            return ServiceStatus::Stopped;
        }

        if let Some(child) = self.processes.get_mut(id) {
            match child.try_wait() {
                Ok(Some(_)) => {
                    if let Some(service) = self.services.get_mut(id) {
                        service.status = ServiceStatus::Error;
                        service.pid = None;
                        service.error_message = Some("Process exited unexpectedly".to_string());
                    }
                    return ServiceStatus::Error;
                }
                Ok(None) => {}
                Err(_) => {}
            }
        }

        let is_healthy = match service.health_check.check_type {
            HealthCheckType::Http => self.check_http_health(&service).await,
            HealthCheckType::Tcp => self.check_tcp_health(&service).await,
            HealthCheckType::Process => true,
        };

        let new_status = if is_healthy {
            ServiceStatus::Running
        } else {
            ServiceStatus::Unhealthy
        };

        if let Some(service) = self.services.get_mut(id) {
            service.status = new_status.clone();
        }

        new_status
    }

    async fn check_http_health(&self, service: &Service) -> bool {
        if let Some(endpoint) = &service.health_check.endpoint {
            let client = reqwest::Client::builder()
                .timeout(Duration::from_millis(service.health_check.timeout))
                .build();

            if let Ok(client) = client {
                if let Ok(response) = client.get(endpoint).send().await {
                    return response.status().is_success() || response.status().as_u16() == 302;
                }
            }
        }
        false
    }

    async fn check_tcp_health(&self, service: &Service) -> bool {
        if let Some(endpoint) = &service.health_check.endpoint {
            let timeout = Duration::from_millis(service.health_check.timeout);
            match tokio::time::timeout(timeout, tokio::net::TcpStream::connect(endpoint)).await {
                Ok(Ok(_)) => return true,
                _ => return false,
            }
        }
        false
    }

    pub async fn auto_restart_if_needed(&mut self, id: &str) -> bool {
        let service = match self.services.get(id) {
            Some(s) => s.clone(),
            None => return false,
        };

        if !service.auto_restart {
            return false;
        }

        if service.status != ServiceStatus::Error && service.status != ServiceStatus::Unhealthy {
            return false;
        }

        if !service.can_restart() {
            if let Some(s) = self.services.get_mut(id) {
                s.error_message = Some("Max restart attempts reached".to_string());
            }
            return false;
        }

        if let Some(s) = self.services.get_mut(id) {
            s.increment_restart_count();
        }

        let delay = service.restart_delay;
        sleep(Duration::from_millis(delay)).await;

        self.start_service(id, None).await.is_ok()
    }

    pub fn get_all_statuses(&self) -> HashMap<String, ServiceStatus> {
        self.services
            .iter()
            .map(|(id, service)| (id.clone(), service.status.clone()))
            .collect()
    }

    /// Ensure MariaDB my.ini has bind-address = 127.0.0.1
    fn ensure_mariadb_localhost_binding(&self) -> Result<(), String> {
        let possible_paths = [
            "C:\\DevPort\\runtime\\mariadb\\data\\my.ini",
            "C:\\DevPort\\runtime\\mariadb\\my.ini",
        ];

        let config_path = possible_paths.iter()
            .map(|p| std::path::PathBuf::from(p))
            .find(|p| p.exists());

        let config_path = match config_path {
            Some(p) => p,
            None => return Ok(()), // No config found, skip
        };

        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read my.ini: {}", e))?;

        // Check if bind-address is already correctly set
        let has_correct_bind = content.lines().any(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with('#')
                && trimmed.starts_with("bind-address")
                && trimmed.contains("127.0.0.1")
        });

        if has_correct_bind {
            return Ok(());
        }

        // Create backup
        let backup_path = config_path.with_extension("ini.bak");
        let _ = fs::copy(&config_path, &backup_path);

        // Check if bind-address exists but is wrong (e.g., 0.0.0.0)
        let has_bind = content.lines().any(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with('#') && trimmed.starts_with("bind-address")
        });

        let new_content = if has_bind {
            // Replace existing bind-address
            content.lines().map(|line| {
                let trimmed = line.trim();
                if !trimmed.starts_with('#') && trimmed.starts_with("bind-address") {
                    "bind-address = 127.0.0.1"
                } else {
                    line
                }
            }).collect::<Vec<_>>().join("\n")
        } else {
            // Add bind-address under [mysqld] section
            let mut result = String::new();
            let mut in_mysqld = false;
            let mut added = false;

            for line in content.lines() {
                result.push_str(line);
                result.push('\n');

                if line.trim() == "[mysqld]" {
                    in_mysqld = true;
                } else if in_mysqld && !added {
                    result.push_str("bind-address = 127.0.0.1\n");
                    added = true;
                }
            }

            if !added {
                // [mysqld] section not found, append
                result.push_str("\n[mysqld]\nbind-address = 127.0.0.1\n");
            }

            result
        };

        fs::write(&config_path, new_content)
            .map_err(|e| format!("Failed to write my.ini: {}", e))?;

        Ok(())
    }

    /// Validate Apache listen port for DevPort installation
    fn validate_apache_listen_port(&self, expected_port: u16) -> Result<(), String> {
        let httpd_conf = Path::new("C:\\DevPort\\runtime\\apache\\conf\\httpd.conf");
        if !httpd_conf.exists() {
            return Ok(()); // Not a DevPort Apache installation
        }

        let content = fs::read_to_string(httpd_conf)
            .map_err(|e| format!("Failed to read httpd.conf: {}", e))?;

        let has_listen = content.lines().any(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with('#')
                && trimmed.starts_with("Listen")
                && trimmed.contains(&expected_port.to_string())
        });

        if !has_listen {
            return Err(format!(
                "Apache httpd.conf does not have 'Listen {}'. Please check your configuration.",
                expected_port
            ));
        }

        Ok(())
    }

    /// Enforce "Require local" for phpMyAdmin VHost directories on Apache start
    fn enforce_phpmyadmin_require_local(&self) -> Result<(), String> {
        let httpd_conf = Path::new("C:\\DevPort\\runtime\\apache\\conf\\httpd.conf");
        if !httpd_conf.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(httpd_conf)
            .map_err(|e| format!("Failed to read httpd.conf: {}", e))?;

        // Check for phpMyAdmin/pma VHost blocks with "Require all granted" instead of "Require local"
        let mut modified = false;

        // Find <Directory> blocks within VirtualHost that contain phpmyadmin/pma paths
        // and replace "Require all granted" with "Require local"
        let lines: Vec<&str> = content.lines().collect();
        let mut result_lines: Vec<String> = Vec::new();
        let mut in_phpmyadmin_dir = false;

        for line in &lines {
            let trimmed = line.trim().to_lowercase();

            if trimmed.starts_with("<directory") && (trimmed.contains("phpmyadmin") || trimmed.contains("pma")) {
                in_phpmyadmin_dir = true;
            }

            if in_phpmyadmin_dir && trimmed == "require all granted" {
                result_lines.push(line.replace("Require all granted", "Require local"));
                modified = true;
            } else {
                result_lines.push(line.to_string());
            }

            if trimmed.starts_with("</directory>") {
                in_phpmyadmin_dir = false;
            }
        }

        if modified {
            let new_content = result_lines.join("\n");
            // Create backup
            let backup_path = httpd_conf.with_extension("conf.bak");
            let _ = fs::copy(httpd_conf, &backup_path);
            fs::write(httpd_conf, &new_content)
                .map_err(|e| format!("Failed to write httpd.conf: {}", e))?;
        }

        Ok(())
    }
}

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect log level from a log line based on keywords
fn detect_log_level(line: &str) -> &'static str {
    let lower = line.to_lowercase();
    if lower.contains("error") || lower.contains("fatal") || lower.contains("fail") || lower.contains("crit") {
        "error"
    } else if lower.contains("warn") {
        "warning"
    } else if lower.contains("note") || lower.contains("notice") || lower.contains("info") || lower.contains("ready for connections") {
        "info"
    } else {
        "debug"
    }
}
