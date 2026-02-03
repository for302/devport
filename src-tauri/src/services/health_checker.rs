use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum HealthCheckError {
    #[error("Request failed: {0}")]
    RequestError(String),
    #[error("Timeout")]
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthStatus {
    pub project_id: String,
    pub is_healthy: bool,
    pub status_code: Option<u16>,
    pub response_time_ms: Option<u64>,
    pub error: Option<String>,
    pub checked_at: String,
}

pub struct HealthChecker;

impl HealthChecker {
    pub async fn check_health(
        project_id: &str,
        url: &str,
        timeout_secs: u64,
    ) -> HealthStatus {
        let start = std::time::Instant::now();
        let checked_at = chrono::Utc::now().to_rfc3339();

        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                return HealthStatus {
                    project_id: project_id.to_string(),
                    is_healthy: false,
                    status_code: None,
                    response_time_ms: None,
                    error: Some(e.to_string()),
                    checked_at,
                };
            }
        };

        match client.get(url).send().await {
            Ok(response) => {
                let elapsed = start.elapsed().as_millis() as u64;
                let status = response.status();
                let is_healthy = status.is_success();

                HealthStatus {
                    project_id: project_id.to_string(),
                    is_healthy,
                    status_code: Some(status.as_u16()),
                    response_time_ms: Some(elapsed),
                    error: if !is_healthy {
                        Some(format!("HTTP {}", status.as_u16()))
                    } else {
                        None
                    },
                    checked_at,
                }
            }
            Err(e) => {
                let elapsed = start.elapsed().as_millis() as u64;
                HealthStatus {
                    project_id: project_id.to_string(),
                    is_healthy: false,
                    status_code: None,
                    response_time_ms: Some(elapsed),
                    error: Some(e.to_string()),
                    checked_at,
                }
            }
        }
    }

    /// Check health by verifying a process is still alive (for desktop/local apps with no port)
    pub fn check_process_alive(project_id: &str, pid: u32) -> HealthStatus {
        let checked_at = chrono::Utc::now().to_rfc3339();

        let is_alive = Self::is_pid_alive(pid);

        HealthStatus {
            project_id: project_id.to_string(),
            is_healthy: is_alive,
            status_code: None,
            response_time_ms: Some(0),
            error: if !is_alive {
                Some("Process is no longer running".to_string())
            } else {
                None
            },
            checked_at,
        }
    }

    /// Check if a PID is still alive
    fn is_pid_alive(pid: u32) -> bool {
        #[cfg(windows)]
        {
            use std::process::Command;
            // Use tasklist to check if PID exists
            let output = Command::new("tasklist")
                .args(["/FI", &format!("PID eq {}", pid), "/NH"])
                .output();
            match output {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    // tasklist returns "INFO: No tasks are running..." if PID not found
                    !stdout.contains("No tasks") && stdout.contains(&pid.to_string())
                }
                Err(_) => false,
            }
        }

        #[cfg(not(windows))]
        {
            use std::process::Command;
            // Use kill -0 to check if process exists (sends no signal)
            Command::new("kill")
                .args(["-0", &pid.to_string()])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
    }

    pub fn default_health_url(port: u16) -> String {
        format!("http://localhost:{}", port)
    }
}
