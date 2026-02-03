use crate::models::PortInfo;
use std::process::Command;
use thiserror::Error;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Error, Debug)]
pub enum PortScannerError {
    #[error("Failed to execute netstat: {0}")]
    ExecutionError(String),
    #[error("Failed to parse output: {0}")]
    ParseError(String),
}

pub struct PortScanner;

impl PortScanner {
    pub fn scan_ports() -> Result<Vec<PortInfo>, PortScannerError> {
        #[cfg(windows)]
        let output = Command::new("netstat")
            .args(["-ano"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| PortScannerError::ExecutionError(e.to_string()))?;

        #[cfg(not(windows))]
        let output = Command::new("netstat")
            .args(["-ano"])
            .output()
            .map_err(|e| PortScannerError::ExecutionError(e.to_string()))?;

        if !output.status.success() {
            return Err(PortScannerError::ExecutionError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Self::parse_netstat_output(&stdout)
    }

    fn parse_netstat_output(output: &str) -> Result<Vec<PortInfo>, PortScannerError> {
        let mut ports = Vec::new();
        let mut seen_ports = std::collections::HashSet::new();

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("Active") || line.starts_with("Proto") {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 4 {
                continue;
            }

            let protocol = parts[0].to_uppercase();
            if protocol != "TCP" && protocol != "UDP" {
                continue;
            }

            let local_address = parts[1];
            let state = if protocol == "TCP" && parts.len() > 3 {
                parts[3].to_string()
            } else {
                "N/A".to_string()
            };

            let pid_str = parts.last().unwrap_or(&"0");
            let pid: u32 = pid_str.parse().unwrap_or(0);

            // Extract port from address (format: IP:PORT or [IPv6]:PORT)
            let port = if let Some(last_colon) = local_address.rfind(':') {
                local_address[last_colon + 1..]
                    .parse::<u16>()
                    .unwrap_or(0)
            } else {
                continue;
            };

            if port == 0 || seen_ports.contains(&port) {
                continue;
            }

            // Only include LISTENING ports or common dev ports
            let is_listening = state == "LISTENING";
            let is_dev_port = port >= 3000 && port <= 9999;

            if is_listening || is_dev_port {
                seen_ports.insert(port);
                let mut port_info = PortInfo::new(
                    port,
                    protocol.clone(),
                    state.clone(),
                    local_address.to_string(),
                );
                port_info.pid = Some(pid);
                port_info.process_name = Self::get_process_name(pid);
                ports.push(port_info);
            }
        }

        ports.sort_by_key(|p| p.port);
        Ok(ports)
    }

    fn get_process_name(pid: u32) -> Option<String> {
        if pid == 0 {
            return None;
        }

        #[cfg(windows)]
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .ok()?;

        #[cfg(not(windows))]
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.lines().next()?;
        let parts: Vec<&str> = line.split(',').collect();
        if parts.is_empty() {
            return None;
        }

        Some(parts[0].trim_matches('"').to_string())
    }

    pub fn is_port_available(port: u16) -> bool {
        #[cfg(windows)]
        let output = Command::new("netstat")
            .args(["-ano"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .ok();

        #[cfg(not(windows))]
        let output = Command::new("netstat")
            .args(["-ano"])
            .output()
            .ok();

        if let Some(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains(&format!(":{} ", port)) || line.contains(&format!(":{}\t", port)) {
                    if line.contains("LISTENING") || line.contains("ESTABLISHED") {
                        return false;
                    }
                }
            }
        }
        true
    }
}
