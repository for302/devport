use crate::models::{Service, ServiceStatus, HealthCheckType};
use crate::services::log_manager::LogManager;
use crate::services::port_scanner::PortScanner;
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use tokio::time::{sleep, Duration};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;
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

    pub async fn start_service(&mut self, id: &str) -> Result<(), String> {
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
            self.start_service_internal(&dep_id).await?;
        }

        self.start_service_internal(id).await
    }

    async fn start_service_internal(&mut self, id: &str) -> Result<(), String> {
        let service = self.services.get(id).ok_or("Service not found")?;

        if service.is_running() {
            return Ok(());
        }

        let service = self.services.get(id).unwrap();
        let executable = service.executable.clone();
        let args = service.args.clone();
        let work_dir = service.work_dir.clone();
        let env = service.env.clone();

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
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        #[cfg(windows)]
        {
            cmd.creation_flags(CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP);
        }

        match cmd.spawn() {
            Ok(child) => {
                let pid = child.id();
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
            // Process started by DevPort - kill via Child handle
            #[cfg(windows)]
            {
                if let Some(pid) = pid {
                    let _ = Command::new("taskkill")
                        .args(["/F", "/T", "/PID", &pid.to_string()])
                        .creation_flags(CREATE_NO_WINDOW)
                        .output();
                }
            }

            #[cfg(not(windows))]
            {
                let _ = child.kill();
            }

            let _ = child.wait();
        } else if let Some(pid) = pid {
            // Externally started process - kill by PID directly
            #[cfg(windows)]
            {
                let output = Command::new("taskkill")
                    .args(["/F", "/T", "/PID", &pid.to_string()])
                    .creation_flags(CREATE_NO_WINDOW)
                    .output()
                    .map_err(|e| format!("Failed to kill process: {}", e))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(format!("Failed to stop external process (PID {}): {}", pid, stderr));
                }
            }

            #[cfg(not(windows))]
            {
                let _ = Command::new("kill")
                    .args(["-9", &pid.to_string()])
                    .output();
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

    pub async fn restart_service(&mut self, id: &str) -> Result<(), String> {
        self.stop_service(id).await?;
        sleep(Duration::from_millis(1000)).await;
        self.start_service(id).await
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

        self.start_service(id).await.is_ok()
    }

    pub fn get_all_statuses(&self) -> HashMap<String, ServiceStatus> {
        self.services
            .iter()
            .map(|(id, service)| (id.clone(), service.status.clone()))
            .collect()
    }
}

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}
