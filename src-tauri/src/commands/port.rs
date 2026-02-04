use crate::models::PortInfo;
use crate::services::port_scanner::PortScanner;
use crate::services::process_manager::kill_process_tree;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
use crate::services::process_manager::CREATE_NO_WINDOW;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessDetails {
    pub pid: u32,
    pub name: String,
    pub path: Option<String>,
    pub command_line: Option<String>,
}

#[tauri::command]
pub async fn scan_ports() -> Result<Vec<PortInfo>, String> {
    tokio::task::spawn_blocking(|| {
        PortScanner::scan_ports().map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn check_port_available(port: u16) -> Result<bool, String> {
    let result = tokio::task::spawn_blocking(move || {
        PortScanner::is_port_available(port)
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(result)
}

/// Get detailed information about a process by PID
#[tauri::command]
pub async fn get_process_details(pid: u32) -> Result<ProcessDetails, String> {
    tokio::task::spawn_blocking(move || {
        // Use WMIC to get process details on Windows
        #[cfg(windows)]
        let output = Command::new("wmic")
            .args([
                "process",
                "where",
                &format!("ProcessId={}", pid),
                "get",
                "Name,ExecutablePath,CommandLine",
                "/format:csv",
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("Failed to get process details: {}", e))?;

        #[cfg(not(windows))]
        let output = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "comm="])
            .output()
            .map_err(|e| format!("Failed to get process details: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines: Vec<&str> = stdout.lines().filter(|l| !l.trim().is_empty()).collect();

        if lines.len() < 2 {
            return Err(format!("Process with PID {} not found", pid));
        }

        // Parse CSV output (Node,CommandLine,ExecutablePath,Name)
        let data_line = lines[1];
        let parts: Vec<&str> = data_line.split(',').collect();

        let (command_line, path, name) = if parts.len() >= 4 {
            (
                Some(parts[1].to_string()),
                if parts[2].is_empty() { None } else { Some(parts[2].to_string()) },
                parts[3].to_string(),
            )
        } else {
            (None, None, "Unknown".to_string())
        };

        Ok(ProcessDetails {
            pid,
            name,
            path,
            command_line,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Kill a process by PID
#[tauri::command]
pub async fn kill_process_by_pid(pid: u32) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let result = kill_process_tree(pid);
        if result.success {
            Ok(())
        } else {
            Err(format!(
                "Failed to kill process {}: {}",
                pid,
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    })
    .await
    .map_err(|e| e.to_string())?
}
