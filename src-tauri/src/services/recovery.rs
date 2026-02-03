use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionState {
    pub services: HashMap<String, ServiceState>,
    pub projects: HashMap<String, ProjectState>,
    pub saved_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceState {
    pub id: String,
    pub was_running: bool,
    pub pid: Option<u32>,
    pub auto_start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectState {
    pub id: String,
    pub was_running: bool,
    pub pid: Option<u32>,
    pub port: u16,
}

pub struct RecoveryManager {
    state_file: PathBuf,
}

impl RecoveryManager {
    pub fn new() -> Self {
        let state_file = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("clickdevport")
            .join("session_state.json");

        Self { state_file }
    }

    /// Save current session state for recovery
    pub fn save_session_state(&self, state: &SessionState) -> Result<(), String> {
        // Ensure directory exists
        if let Some(parent) = self.state_file.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let json = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
        fs::write(&self.state_file, json).map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Load previous session state
    pub fn load_session_state(&self) -> Option<SessionState> {
        if !self.state_file.exists() {
            return None;
        }

        let content = fs::read_to_string(&self.state_file).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Clear saved session state
    pub fn clear_session_state(&self) -> Result<(), String> {
        if self.state_file.exists() {
            fs::remove_file(&self.state_file).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Check if a PID is still running
    #[cfg(windows)]
    pub fn is_pid_running(&self, pid: u32) -> bool {
        use std::process::Command;
        use std::os::windows::process::CommandExt;

        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/NH"])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                !stdout.contains("No tasks") && stdout.contains(&pid.to_string())
            }
            Err(_) => false,
        }
    }

    #[cfg(not(windows))]
    pub fn is_pid_running(&self, pid: u32) -> bool {
        use std::path::Path;
        Path::new(&format!("/proc/{}", pid)).exists()
    }

    /// Kill stale process by PID
    #[cfg(windows)]
    pub fn kill_stale_process(&self, pid: u32) -> Result<(), String> {
        use std::process::Command;
        use std::os::windows::process::CommandExt;

        let output = Command::new("taskkill")
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to kill process: {}", stderr))
        }
    }

    #[cfg(not(windows))]
    pub fn kill_stale_process(&self, pid: u32) -> Result<(), String> {
        use std::process::Command;

        let _ = Command::new("kill")
            .args(["-9", &pid.to_string()])
            .output();

        Ok(())
    }

    /// Check and cleanup stale PIDs from previous session
    pub fn cleanup_stale_processes(&self) -> Vec<String> {
        let mut cleaned = Vec::new();

        if let Some(state) = self.load_session_state() {
            // Check service PIDs
            for (id, service_state) in &state.services {
                if let Some(pid) = service_state.pid {
                    if self.is_pid_running(pid) {
                        // Process is still running, might need cleanup
                        if let Ok(()) = self.kill_stale_process(pid) {
                            cleaned.push(format!("Service {}: killed stale PID {}", id, pid));
                        }
                    }
                }
            }

            // Check project PIDs
            for (id, project_state) in &state.projects {
                if let Some(pid) = project_state.pid {
                    if self.is_pid_running(pid) {
                        if let Ok(()) = self.kill_stale_process(pid) {
                            cleaned.push(format!("Project {}: killed stale PID {}", id, pid));
                        }
                    }
                }
            }
        }

        cleaned
    }

    /// Get services that should be auto-started
    pub fn get_auto_start_services(&self) -> Vec<String> {
        if let Some(state) = self.load_session_state() {
            return state
                .services
                .iter()
                .filter(|(_, s)| s.auto_start)
                .map(|(id, _)| id.clone())
                .collect();
        }
        Vec::new()
    }

    /// Get services that were running before crash
    pub fn get_services_to_restore(&self) -> Vec<String> {
        if let Some(state) = self.load_session_state() {
            return state
                .services
                .iter()
                .filter(|(_, s)| s.was_running)
                .map(|(id, _)| id.clone())
                .collect();
        }
        Vec::new()
    }

    /// Check if port is in use
    #[cfg(windows)]
    pub fn is_port_in_use(&self, port: u16) -> bool {
        use std::process::Command;
        use std::os::windows::process::CommandExt;

        let output = Command::new("netstat")
            .args(["-ano"])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout.contains(&format!(":{}", port))
            }
            Err(_) => false,
        }
    }

    #[cfg(not(windows))]
    pub fn is_port_in_use(&self, port: u16) -> bool {
        use std::net::TcpListener;
        TcpListener::bind(("127.0.0.1", port)).is_err()
    }

    /// Get PID using a specific port
    #[cfg(windows)]
    pub fn get_pid_on_port(&self, port: u16) -> Option<u32> {
        use std::process::Command;
        use std::os::windows::process::CommandExt;

        let output = Command::new("netstat")
            .args(["-ano"])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            if line.contains(&format!(":{}", port)) && line.contains("LISTENING") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(pid_str) = parts.last() {
                    return pid_str.parse().ok();
                }
            }
        }

        None
    }

    #[cfg(not(windows))]
    pub fn get_pid_on_port(&self, port: u16) -> Option<u32> {
        use std::process::Command;

        let output = Command::new("lsof")
            .args(["-i", &format!(":{}", port), "-t"])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.trim().parse().ok()
    }
}

impl Default for RecoveryManager {
    fn default() -> Self {
        Self::new()
    }
}
