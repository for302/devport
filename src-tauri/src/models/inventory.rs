use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum InventoryCategory {
    Runtime,
    WebServer,
    Database,
    BuildTool,
    Framework,
    PackageManager,
    DevTool,
}

impl InventoryCategory {
    pub fn all() -> Vec<InventoryCategory> {
        vec![
            InventoryCategory::Runtime,
            InventoryCategory::WebServer,
            InventoryCategory::Database,
            InventoryCategory::BuildTool,
            InventoryCategory::Framework,
            InventoryCategory::PackageManager,
            InventoryCategory::DevTool,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum InstallSource {
    System,
    DevPort,
    Xampp,
    Laragon,
    Wamp,
    Scoop,
    Chocolatey,
    Manual,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryItem {
    pub id: String,
    pub name: String,
    pub category: InventoryCategory,
    pub is_installed: bool,
    pub version: Option<String>,
    pub executable_path: Option<String>,
    pub install_source: InstallSource,
    pub is_running: bool,
    pub port: Option<u16>,
}

impl InventoryItem {
    pub fn new(id: String, name: String, category: InventoryCategory) -> Self {
        Self {
            id,
            name,
            category,
            is_installed: false,
            version: None,
            executable_path: None,
            install_source: InstallSource::Unknown,
            is_running: false,
            port: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryResult {
    pub runtimes: Vec<InventoryItem>,
    pub web_servers: Vec<InventoryItem>,
    pub databases: Vec<InventoryItem>,
    pub build_tools: Vec<InventoryItem>,
    pub frameworks: Vec<InventoryItem>,
    pub package_managers: Vec<InventoryItem>,
    pub dev_tools: Vec<InventoryItem>,
    pub scanned_at: String,
    pub scan_duration_ms: u64,
}

impl InventoryResult {
    pub fn new() -> Self {
        Self {
            runtimes: Vec::new(),
            web_servers: Vec::new(),
            databases: Vec::new(),
            build_tools: Vec::new(),
            frameworks: Vec::new(),
            package_managers: Vec::new(),
            dev_tools: Vec::new(),
            scanned_at: String::new(),
            scan_duration_ms: 0,
        }
    }

    pub fn total_installed(&self) -> usize {
        self.runtimes.iter().filter(|i| i.is_installed).count()
            + self.web_servers.iter().filter(|i| i.is_installed).count()
            + self.databases.iter().filter(|i| i.is_installed).count()
            + self.build_tools.iter().filter(|i| i.is_installed).count()
            + self.frameworks.iter().filter(|i| i.is_installed).count()
            + self.package_managers.iter().filter(|i| i.is_installed).count()
            + self.dev_tools.iter().filter(|i| i.is_installed).count()
    }

    pub fn total_items(&self) -> usize {
        self.runtimes.len()
            + self.web_servers.len()
            + self.databases.len()
            + self.build_tools.len()
            + self.frameworks.len()
            + self.package_managers.len()
            + self.dev_tools.len()
    }
}

impl Default for InventoryResult {
    fn default() -> Self {
        Self::new()
    }
}
