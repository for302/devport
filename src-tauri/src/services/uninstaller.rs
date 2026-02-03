use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Uninstall mode determines what gets removed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum UninstallMode {
    /// Basic: Remove app and runtime bundles only (keeps projects, backups, logs)
    Basic,
    /// FullData: Basic + projects, backups, logs, mariadb data
    FullData,
    /// SystemRevert: FullData + Task Scheduler, hosts, firewall rules, shortcuts
    SystemRevert,
}

/// Result of an uninstall operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UninstallResult {
    pub success: bool,
    pub mode: UninstallMode,
    pub removed_items: Vec<RemovedItem>,
    pub failed_items: Vec<FailedItem>,
    pub services_stopped: bool,
    pub projects_stopped: bool,
    pub requires_reboot: bool,
    pub error_message: Option<String>,
}

/// An item that was successfully removed
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemovedItem {
    pub item_type: UninstallItemType,
    pub path: Option<String>,
    pub name: String,
}

/// An item that failed to be removed
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedItem {
    pub item_type: UninstallItemType,
    pub path: Option<String>,
    pub name: String,
    pub reason: String,
}

/// Type of item being uninstalled
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum UninstallItemType {
    Executable,
    Directory,
    File,
    TaskScheduler,
    HostsEntry,
    FirewallRule,
    Shortcut,
    RegistryKey,
}

/// Preview of what will be deleted
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UninstallPreview {
    pub mode: UninstallMode,
    pub items: Vec<UninstallPreviewItem>,
    pub total_size_bytes: u64,
    pub requires_admin: bool,
    pub warnings: Vec<String>,
}

/// A single item in the uninstall preview
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UninstallPreviewItem {
    pub item_type: UninstallItemType,
    pub path: Option<String>,
    pub name: String,
    pub size_bytes: Option<u64>,
    pub exists: bool,
}

/// Manager for uninstall operations
pub struct UninstallManager {
    devport_path: PathBuf,
    appdata_path: PathBuf,
}

impl UninstallManager {
    pub fn new() -> Self {
        let devport_path = PathBuf::from("C:\\DevPort");
        let appdata_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("DevPort");

        Self {
            devport_path,
            appdata_path,
        }
    }

    /// Get a preview of what will be deleted for the given mode
    pub fn get_uninstall_preview(&self, mode: &UninstallMode) -> UninstallPreview {
        let mut items = Vec::new();
        let mut total_size: u64 = 0;
        let mut warnings = Vec::new();
        let mut requires_admin = false;

        // Basic mode items
        items.extend(self.get_basic_items(&mut total_size));

        // Full data mode items
        if *mode == UninstallMode::FullData || *mode == UninstallMode::SystemRevert {
            items.extend(self.get_full_data_items(&mut total_size));
        }

        // System revert mode items
        if *mode == UninstallMode::SystemRevert {
            items.extend(self.get_system_revert_items());
            requires_admin = true;
            warnings.push("This will modify system files (hosts, firewall, Task Scheduler). Administrator privileges required.".to_string());
        }

        // Add warning if projects directory has content
        if *mode == UninstallMode::FullData || *mode == UninstallMode::SystemRevert {
            let projects_path = self.devport_path.join("projects");
            if projects_path.exists() {
                if let Ok(entries) = fs::read_dir(&projects_path) {
                    let count = entries.count();
                    if count > 0 {
                        warnings.push(format!(
                            "Warning: {} project(s) will be permanently deleted from {}",
                            count,
                            projects_path.display()
                        ));
                    }
                }
            }
        }

        UninstallPreview {
            mode: mode.clone(),
            items,
            total_size_bytes: total_size,
            requires_admin,
            warnings,
        }
    }

    /// Get basic mode items (app + runtime bundles)
    fn get_basic_items(&self, total_size: &mut u64) -> Vec<UninstallPreviewItem> {
        let mut items = Vec::new();

        // ClickDevPort.exe
        let exe_path = self.devport_path.join("ClickDevPort.exe");
        items.push(self.create_preview_item(
            UninstallItemType::Executable,
            Some(exe_path.to_string_lossy().to_string()),
            "ClickDevPort.exe".to_string(),
            total_size,
        ));

        // runtime/ directory
        let runtime_path = self.devport_path.join("runtime");
        items.push(self.create_preview_item(
            UninstallItemType::Directory,
            Some(runtime_path.to_string_lossy().to_string()),
            "Runtime bundles (Apache, MariaDB, PHP, Node.js, Git)".to_string(),
            total_size,
        ));

        // tools/ directory
        let tools_path = self.devport_path.join("tools");
        items.push(self.create_preview_item(
            UninstallItemType::Directory,
            Some(tools_path.to_string_lossy().to_string()),
            "Tools (phpMyAdmin, Composer)".to_string(),
            total_size,
        ));

        // config/ directory
        let config_path = self.devport_path.join("config");
        items.push(self.create_preview_item(
            UninstallItemType::Directory,
            Some(config_path.to_string_lossy().to_string()),
            "Configuration files".to_string(),
            total_size,
        ));

        items
    }

    /// Get full data mode items
    fn get_full_data_items(&self, total_size: &mut u64) -> Vec<UninstallPreviewItem> {
        let mut items = Vec::new();

        // projects/ directory
        let projects_path = self.devport_path.join("projects");
        items.push(self.create_preview_item(
            UninstallItemType::Directory,
            Some(projects_path.to_string_lossy().to_string()),
            "All projects".to_string(),
            total_size,
        ));

        // backups/ directory
        let backups_path = self.devport_path.join("backups");
        items.push(self.create_preview_item(
            UninstallItemType::Directory,
            Some(backups_path.to_string_lossy().to_string()),
            "Database backups".to_string(),
            total_size,
        ));

        // logs/ directory
        let logs_path = self.devport_path.join("logs");
        items.push(self.create_preview_item(
            UninstallItemType::Directory,
            Some(logs_path.to_string_lossy().to_string()),
            "Log files".to_string(),
            total_size,
        ));

        // mariadb/data directory
        let mariadb_data_path = self.devport_path.join("runtime").join("mariadb").join("data");
        items.push(self.create_preview_item(
            UninstallItemType::Directory,
            Some(mariadb_data_path.to_string_lossy().to_string()),
            "MariaDB databases".to_string(),
            total_size,
        ));

        // %LOCALAPPDATA%/DevPort
        items.push(self.create_preview_item(
            UninstallItemType::Directory,
            Some(self.appdata_path.to_string_lossy().to_string()),
            "Application data and cache".to_string(),
            total_size,
        ));

        items
    }

    /// Get system revert mode items
    fn get_system_revert_items(&self) -> Vec<UninstallPreviewItem> {
        let mut items = Vec::new();

        // Task Scheduler entry
        items.push(UninstallPreviewItem {
            item_type: UninstallItemType::TaskScheduler,
            path: None,
            name: "ClickDevPort Auto-Start task".to_string(),
            size_bytes: None,
            exists: self.check_task_scheduler_exists(),
        });

        // Hosts entries
        items.push(UninstallPreviewItem {
            item_type: UninstallItemType::HostsEntry,
            path: Some("C:\\Windows\\System32\\drivers\\etc\\hosts".to_string()),
            name: "DevPort hosts entries (# DevPort BEGIN...END section)".to_string(),
            size_bytes: None,
            exists: self.check_hosts_entries_exist(),
        });

        // Firewall rules
        items.push(UninstallPreviewItem {
            item_type: UninstallItemType::FirewallRule,
            path: None,
            name: "DevPort-* firewall rules".to_string(),
            size_bytes: None,
            exists: self.check_firewall_rules_exist(),
        });

        // Start Menu shortcut
        let start_menu_path = dirs::data_dir()
            .map(|p| p.join("Microsoft\\Windows\\Start Menu\\Programs\\ClickDevPort.lnk"));
        items.push(UninstallPreviewItem {
            item_type: UninstallItemType::Shortcut,
            path: start_menu_path.clone().map(|p| p.to_string_lossy().to_string()),
            name: "Start Menu shortcut".to_string(),
            size_bytes: None,
            exists: start_menu_path.map(|p| p.exists()).unwrap_or(false),
        });

        // Desktop shortcut
        let desktop_path = dirs::desktop_dir()
            .map(|p| p.join("ClickDevPort.lnk"));
        items.push(UninstallPreviewItem {
            item_type: UninstallItemType::Shortcut,
            path: desktop_path.clone().map(|p| p.to_string_lossy().to_string()),
            name: "Desktop shortcut".to_string(),
            size_bytes: None,
            exists: desktop_path.map(|p| p.exists()).unwrap_or(false),
        });

        items
    }

    /// Create a preview item and update total size
    fn create_preview_item(
        &self,
        item_type: UninstallItemType,
        path: Option<String>,
        name: String,
        total_size: &mut u64,
    ) -> UninstallPreviewItem {
        let (exists, size) = if let Some(ref p) = path {
            let path_buf = PathBuf::from(p);
            if path_buf.exists() {
                let size = self.get_path_size(&path_buf);
                *total_size += size;
                (true, Some(size))
            } else {
                (false, None)
            }
        } else {
            (false, None)
        };

        UninstallPreviewItem {
            item_type,
            path,
            name,
            size_bytes: size,
            exists,
        }
    }

    /// Get the total size of a path (file or directory)
    fn get_path_size(&self, path: &PathBuf) -> u64 {
        if path.is_file() {
            fs::metadata(path).map(|m| m.len()).unwrap_or(0)
        } else if path.is_dir() {
            self.get_dir_size(path)
        } else {
            0
        }
    }

    /// Get the total size of a directory recursively
    fn get_dir_size(&self, path: &PathBuf) -> u64 {
        let mut size = 0;
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    size += fs::metadata(&entry_path).map(|m| m.len()).unwrap_or(0);
                } else if entry_path.is_dir() {
                    size += self.get_dir_size(&entry_path);
                }
            }
        }
        size
    }

    /// Perform basic uninstall (app + runtime bundles only)
    pub async fn uninstall_basic(&self) -> UninstallResult {
        let mut removed_items = Vec::new();
        let mut failed_items = Vec::new();

        // Remove ClickDevPort.exe
        self.remove_file(
            &self.devport_path.join("ClickDevPort.exe"),
            "ClickDevPort.exe",
            &mut removed_items,
            &mut failed_items,
        );

        // Remove runtime directory
        self.remove_directory(
            &self.devport_path.join("runtime"),
            "Runtime bundles",
            &mut removed_items,
            &mut failed_items,
        );

        // Remove tools directory
        self.remove_directory(
            &self.devport_path.join("tools"),
            "Tools directory",
            &mut removed_items,
            &mut failed_items,
        );

        // Remove config directory
        self.remove_directory(
            &self.devport_path.join("config"),
            "Configuration files",
            &mut removed_items,
            &mut failed_items,
        );

        UninstallResult {
            success: failed_items.is_empty(),
            mode: UninstallMode::Basic,
            removed_items,
            failed_items,
            services_stopped: true,
            projects_stopped: true,
            requires_reboot: false,
            error_message: None,
        }
    }

    /// Perform full data uninstall (basic + projects, backups, logs, mariadb data)
    pub async fn uninstall_full_data(&self) -> UninstallResult {
        let mut result = self.uninstall_basic().await;
        result.mode = UninstallMode::FullData;

        // Remove projects directory
        self.remove_directory(
            &self.devport_path.join("projects"),
            "Projects directory",
            &mut result.removed_items,
            &mut result.failed_items,
        );

        // Remove backups directory
        self.remove_directory(
            &self.devport_path.join("backups"),
            "Backups directory",
            &mut result.removed_items,
            &mut result.failed_items,
        );

        // Remove logs directory
        self.remove_directory(
            &self.devport_path.join("logs"),
            "Logs directory",
            &mut result.removed_items,
            &mut result.failed_items,
        );

        // Remove mariadb data directory
        self.remove_directory(
            &self.devport_path.join("runtime").join("mariadb").join("data"),
            "MariaDB data",
            &mut result.removed_items,
            &mut result.failed_items,
        );

        // Cleanup AppData
        self.cleanup_appdata(&mut result.removed_items, &mut result.failed_items);

        result.success = result.failed_items.is_empty();
        result
    }

    /// Perform system revert uninstall (full data + system changes)
    pub async fn uninstall_system_revert(&self) -> UninstallResult {
        let mut result = self.uninstall_full_data().await;
        result.mode = UninstallMode::SystemRevert;

        // Remove Task Scheduler entry
        self.remove_task_scheduler_entry(&mut result.removed_items, &mut result.failed_items);

        // Remove hosts entries
        self.remove_hosts_entries(&mut result.removed_items, &mut result.failed_items);

        // Remove firewall rules
        self.remove_firewall_rules(&mut result.removed_items, &mut result.failed_items);

        // Remove shortcuts
        self.remove_shortcuts(&mut result.removed_items, &mut result.failed_items);

        result.success = result.failed_items.is_empty();
        result
    }

    /// Stop all services (Apache, MariaDB) before uninstall
    #[cfg(windows)]
    pub fn stop_all_services(&self) -> Result<(), String> {
        // Stop Apache
        let apache_exe = self.devport_path.join("runtime").join("apache").join("bin").join("httpd.exe");
        if apache_exe.exists() {
            let _ = Command::new("taskkill")
                .args(["/F", "/IM", "httpd.exe"])
                .creation_flags(CREATE_NO_WINDOW)
                .output();
        }

        // Stop MariaDB
        let mariadb_exe = self.devport_path.join("runtime").join("mariadb").join("bin").join("mysqld.exe");
        if mariadb_exe.exists() {
            let _ = Command::new("taskkill")
                .args(["/F", "/IM", "mysqld.exe"])
                .creation_flags(CREATE_NO_WINDOW)
                .output();
        }

        // Give processes time to terminate
        std::thread::sleep(std::time::Duration::from_millis(1000));

        Ok(())
    }

    #[cfg(not(windows))]
    pub fn stop_all_services(&self) -> Result<(), String> {
        Err("Stop services is only supported on Windows".to_string())
    }

    /// Stop all running projects before uninstall
    #[cfg(windows)]
    pub fn stop_all_projects(&self) -> Result<(), String> {
        // Kill node.exe processes (for Node.js projects)
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "node.exe"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        // Kill php.exe processes (for PHP projects)
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "php.exe"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        // Kill python.exe processes (for Python projects)
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "python.exe"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        std::thread::sleep(std::time::Duration::from_millis(500));

        Ok(())
    }

    #[cfg(not(windows))]
    pub fn stop_all_projects(&self) -> Result<(), String> {
        Err("Stop projects is only supported on Windows".to_string())
    }

    /// Remove Task Scheduler entry
    #[cfg(windows)]
    pub fn remove_task_scheduler_entry(
        &self,
        removed: &mut Vec<RemovedItem>,
        failed: &mut Vec<FailedItem>,
    ) {
        let task_name = "ClickDevPort Auto-Start";

        let output = Command::new("schtasks")
            .args(["/delete", "/tn", task_name, "/f"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        match output {
            Ok(o) if o.status.success() => {
                removed.push(RemovedItem {
                    item_type: UninstallItemType::TaskScheduler,
                    path: None,
                    name: task_name.to_string(),
                });
            }
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr);
                // Task doesn't exist is not an error
                if !stderr.contains("does not exist") && !stderr.contains("cannot find") {
                    failed.push(FailedItem {
                        item_type: UninstallItemType::TaskScheduler,
                        path: None,
                        name: task_name.to_string(),
                        reason: stderr.to_string(),
                    });
                }
            }
            Err(e) => {
                failed.push(FailedItem {
                    item_type: UninstallItemType::TaskScheduler,
                    path: None,
                    name: task_name.to_string(),
                    reason: e.to_string(),
                });
            }
        }
    }

    #[cfg(not(windows))]
    pub fn remove_task_scheduler_entry(
        &self,
        _removed: &mut Vec<RemovedItem>,
        _failed: &mut Vec<FailedItem>,
    ) {
        // No-op on non-Windows
    }

    /// Remove DevPort entries from hosts file
    #[cfg(windows)]
    pub fn remove_hosts_entries(
        &self,
        removed: &mut Vec<RemovedItem>,
        failed: &mut Vec<FailedItem>,
    ) {
        let hosts_path = PathBuf::from("C:\\Windows\\System32\\drivers\\etc\\hosts");

        if !hosts_path.exists() {
            return;
        }

        match fs::read_to_string(&hosts_path) {
            Ok(content) => {
                let marker_begin = "# DevPort BEGIN";
                let marker_end = "# DevPort END";

                if !content.contains(marker_begin) {
                    return;
                }

                let mut new_lines: Vec<&str> = Vec::new();
                let mut in_devport_section = false;

                for line in content.lines() {
                    let trimmed = line.trim();

                    if trimmed == marker_begin {
                        in_devport_section = true;
                        continue;
                    }

                    if trimmed == marker_end {
                        in_devport_section = false;
                        continue;
                    }

                    if !in_devport_section {
                        new_lines.push(line);
                    }
                }

                let new_content = new_lines.join("\n");

                match fs::write(&hosts_path, new_content) {
                    Ok(_) => {
                        removed.push(RemovedItem {
                            item_type: UninstallItemType::HostsEntry,
                            path: Some(hosts_path.to_string_lossy().to_string()),
                            name: "DevPort hosts entries".to_string(),
                        });
                    }
                    Err(e) => {
                        failed.push(FailedItem {
                            item_type: UninstallItemType::HostsEntry,
                            path: Some(hosts_path.to_string_lossy().to_string()),
                            name: "DevPort hosts entries".to_string(),
                            reason: format!("Failed to write hosts file: {}. Try running as administrator.", e),
                        });
                    }
                }
            }
            Err(e) => {
                failed.push(FailedItem {
                    item_type: UninstallItemType::HostsEntry,
                    path: Some(hosts_path.to_string_lossy().to_string()),
                    name: "DevPort hosts entries".to_string(),
                    reason: format!("Failed to read hosts file: {}", e),
                });
            }
        }
    }

    #[cfg(not(windows))]
    pub fn remove_hosts_entries(
        &self,
        _removed: &mut Vec<RemovedItem>,
        _failed: &mut Vec<FailedItem>,
    ) {
        // No-op on non-Windows
    }

    /// Remove DevPort-* firewall rules
    #[cfg(windows)]
    pub fn remove_firewall_rules(
        &self,
        removed: &mut Vec<RemovedItem>,
        failed: &mut Vec<FailedItem>,
    ) {
        // List all DevPort firewall rules
        let output = Command::new("netsh")
            .args(["advfirewall", "firewall", "show", "rule", "name=all"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        let rules_to_remove: Vec<String> = match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                stdout
                    .lines()
                    .filter(|line| line.starts_with("Rule Name:") && line.contains("DevPort-"))
                    .map(|line| {
                        line.trim_start_matches("Rule Name:")
                            .trim()
                            .to_string()
                    })
                    .collect()
            }
            Err(_) => Vec::new(),
        };

        for rule_name in rules_to_remove {
            let delete_output = Command::new("netsh")
                .args([
                    "advfirewall",
                    "firewall",
                    "delete",
                    "rule",
                    &format!("name={}", rule_name),
                ])
                .creation_flags(CREATE_NO_WINDOW)
                .output();

            match delete_output {
                Ok(o) if o.status.success() => {
                    removed.push(RemovedItem {
                        item_type: UninstallItemType::FirewallRule,
                        path: None,
                        name: rule_name,
                    });
                }
                Ok(o) => {
                    failed.push(FailedItem {
                        item_type: UninstallItemType::FirewallRule,
                        path: None,
                        name: rule_name,
                        reason: String::from_utf8_lossy(&o.stderr).to_string(),
                    });
                }
                Err(e) => {
                    failed.push(FailedItem {
                        item_type: UninstallItemType::FirewallRule,
                        path: None,
                        name: rule_name,
                        reason: e.to_string(),
                    });
                }
            }
        }
    }

    #[cfg(not(windows))]
    pub fn remove_firewall_rules(
        &self,
        _removed: &mut Vec<RemovedItem>,
        _failed: &mut Vec<FailedItem>,
    ) {
        // No-op on non-Windows
    }

    /// Remove Start Menu and Desktop shortcuts
    pub fn remove_shortcuts(
        &self,
        removed: &mut Vec<RemovedItem>,
        failed: &mut Vec<FailedItem>,
    ) {
        // Start Menu shortcut
        if let Some(start_menu_path) = dirs::data_dir() {
            let shortcut_path = start_menu_path
                .join("Microsoft\\Windows\\Start Menu\\Programs\\ClickDevPort.lnk");
            self.remove_file(
                &shortcut_path,
                "Start Menu shortcut",
                removed,
                failed,
            );
        }

        // Desktop shortcut
        if let Some(desktop_path) = dirs::desktop_dir() {
            let shortcut_path = desktop_path.join("ClickDevPort.lnk");
            self.remove_file(
                &shortcut_path,
                "Desktop shortcut",
                removed,
                failed,
            );
        }
    }

    /// Cleanup %LOCALAPPDATA%/DevPort
    pub fn cleanup_appdata(
        &self,
        removed: &mut Vec<RemovedItem>,
        failed: &mut Vec<FailedItem>,
    ) {
        self.remove_directory(&self.appdata_path, "Application data", removed, failed);

        // Also clean up clickdevport directory (used by some config files)
        let devport_manager_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("clickdevport");
        self.remove_directory(&devport_manager_path, "ClickDevPort config", removed, failed);
    }

    /// Helper to remove a file
    fn remove_file(
        &self,
        path: &PathBuf,
        name: &str,
        removed: &mut Vec<RemovedItem>,
        failed: &mut Vec<FailedItem>,
    ) {
        if !path.exists() {
            return;
        }

        match fs::remove_file(path) {
            Ok(_) => {
                removed.push(RemovedItem {
                    item_type: UninstallItemType::File,
                    path: Some(path.to_string_lossy().to_string()),
                    name: name.to_string(),
                });
            }
            Err(e) => {
                failed.push(FailedItem {
                    item_type: UninstallItemType::File,
                    path: Some(path.to_string_lossy().to_string()),
                    name: name.to_string(),
                    reason: e.to_string(),
                });
            }
        }
    }

    /// Helper to remove a directory recursively
    fn remove_directory(
        &self,
        path: &PathBuf,
        name: &str,
        removed: &mut Vec<RemovedItem>,
        failed: &mut Vec<FailedItem>,
    ) {
        if !path.exists() {
            return;
        }

        match fs::remove_dir_all(path) {
            Ok(_) => {
                removed.push(RemovedItem {
                    item_type: UninstallItemType::Directory,
                    path: Some(path.to_string_lossy().to_string()),
                    name: name.to_string(),
                });
            }
            Err(e) => {
                failed.push(FailedItem {
                    item_type: UninstallItemType::Directory,
                    path: Some(path.to_string_lossy().to_string()),
                    name: name.to_string(),
                    reason: e.to_string(),
                });
            }
        }
    }

    /// Check if Task Scheduler entry exists
    #[cfg(windows)]
    fn check_task_scheduler_exists(&self) -> bool {
        let output = Command::new("schtasks")
            .args(["/query", "/tn", "ClickDevPort Auto-Start"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        match output {
            Ok(o) => o.status.success(),
            Err(_) => false,
        }
    }

    #[cfg(not(windows))]
    fn check_task_scheduler_exists(&self) -> bool {
        false
    }

    /// Check if hosts entries exist
    fn check_hosts_entries_exist(&self) -> bool {
        let hosts_path = PathBuf::from("C:\\Windows\\System32\\drivers\\etc\\hosts");
        if let Ok(content) = fs::read_to_string(hosts_path) {
            content.contains("# DevPort BEGIN")
        } else {
            false
        }
    }

    /// Check if firewall rules exist
    #[cfg(windows)]
    fn check_firewall_rules_exist(&self) -> bool {
        let output = Command::new("netsh")
            .args(["advfirewall", "firewall", "show", "rule", "name=all"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();

        match output {
            Ok(o) => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                stdout.contains("DevPort-")
            }
            Err(_) => false,
        }
    }

    #[cfg(not(windows))]
    fn check_firewall_rules_exist(&self) -> bool {
        false
    }
}

impl Default for UninstallManager {
    fn default() -> Self {
        Self::new()
    }
}
