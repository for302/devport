use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const TASK_NAME: &str = "ClickDevPort Auto-Start";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AutoStartConfig {
    pub enabled: bool,
    pub services: Vec<String>,
}

pub struct SchedulerManager {
    config_file: PathBuf,
}

impl SchedulerManager {
    pub fn new() -> Self {
        let config_file = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("clickdevport")
            .join("autostart_config.json");

        Self { config_file }
    }

    /// Get the path to the current executable
    fn get_exe_path() -> Result<String, String> {
        std::env::current_exe()
            .map_err(|e| format!("Failed to get executable path: {}", e))
            .map(|p| p.to_string_lossy().to_string())
    }

    /// Register the auto-start task in Windows Task Scheduler
    #[cfg(windows)]
    pub fn register_auto_start(&self) -> Result<(), String> {
        let exe_path = Self::get_exe_path()?;

        // First, try to delete existing task (ignore errors)
        let _ = self.unregister_auto_start();

        // Create new task with schtasks
        // /tn - task name
        // /tr - program to run
        // /sc onlogon - trigger at user logon
        // /rl highest - run with highest privileges
        let mut cmd = Command::new("schtasks");
        cmd.args([
            "/create",
            "/tn",
            TASK_NAME,
            "/tr",
            &format!("\"{}\"", exe_path),
            "/sc",
            "onlogon",
            "/rl",
            "highest",
            "/f", // Force create (overwrite if exists)
        ]);

        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let output = cmd.output()
            .map_err(|e| format!("Failed to execute schtasks: {}", e))?;

        if output.status.success() {
            // Update config to reflect enabled state
            let mut config = self.load_config();
            config.enabled = true;
            self.save_config(&config)?;
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            Err(format!(
                "Failed to create scheduled task. stderr: {}, stdout: {}",
                stderr, stdout
            ))
        }
    }

    #[cfg(not(windows))]
    pub fn register_auto_start(&self) -> Result<(), String> {
        Err("Auto-start registration is only supported on Windows".to_string())
    }

    /// Unregister the auto-start task from Windows Task Scheduler
    #[cfg(windows)]
    pub fn unregister_auto_start(&self) -> Result<(), String> {
        let mut cmd = Command::new("schtasks");
        cmd.args([
            "/delete",
            "/tn",
            TASK_NAME,
            "/f", // Force delete without confirmation
        ]);

        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let output = cmd.output()
            .map_err(|e| format!("Failed to execute schtasks: {}", e))?;

        // Update config to reflect disabled state
        let mut config = self.load_config();
        config.enabled = false;
        self.save_config(&config)?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // If task doesn't exist, that's fine
            if stderr.contains("does not exist") || stderr.contains("cannot find") {
                Ok(())
            } else {
                Err(format!("Failed to delete scheduled task: {}", stderr))
            }
        }
    }

    #[cfg(not(windows))]
    pub fn unregister_auto_start(&self) -> Result<(), String> {
        Err("Auto-start unregistration is only supported on Windows".to_string())
    }

    /// Check if the auto-start task exists in Windows Task Scheduler
    #[cfg(windows)]
    pub fn is_auto_start_enabled(&self) -> Result<bool, String> {
        let mut cmd = Command::new("schtasks");
        cmd.args(["/query", "/tn", TASK_NAME]);

        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let output = cmd.output()
            .map_err(|e| format!("Failed to execute schtasks: {}", e))?;

        // If the command succeeds, the task exists
        Ok(output.status.success())
    }

    #[cfg(not(windows))]
    pub fn is_auto_start_enabled(&self) -> Result<bool, String> {
        Ok(false)
    }

    /// Load auto-start configuration from file
    pub fn load_config(&self) -> AutoStartConfig {
        if !self.config_file.exists() {
            return AutoStartConfig::default();
        }

        match fs::read_to_string(&self.config_file) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => AutoStartConfig::default(),
        }
    }

    /// Save auto-start configuration to file
    pub fn save_config(&self, config: &AutoStartConfig) -> Result<(), String> {
        // Ensure directory exists
        if let Some(parent) = self.config_file.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
        fs::write(&self.config_file, json).map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Get the list of services configured for auto-start
    pub fn get_auto_start_services(&self) -> Vec<String> {
        self.load_config().services
    }

    /// Set the list of services to auto-start
    pub fn set_auto_start_services(&self, services: Vec<String>) -> Result<(), String> {
        let mut config = self.load_config();
        config.services = services;
        self.save_config(&config)
    }
}

impl Default for SchedulerManager {
    fn default() -> Self {
        Self::new()
    }
}
