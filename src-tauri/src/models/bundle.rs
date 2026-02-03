use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Component category for organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ComponentCategory {
    WebServer,
    Database,
    Runtime,
    PackageManager,
    DevTool,
    BuildTool,
}

impl ComponentCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            ComponentCategory::WebServer => "Web Servers",
            ComponentCategory::Database => "Databases",
            ComponentCategory::Runtime => "Runtimes",
            ComponentCategory::PackageManager => "Package Managers",
            ComponentCategory::DevTool => "Dev Tools",
            ComponentCategory::BuildTool => "Build Tools",
        }
    }
}

/// Post-install action types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PostInstallAction {
    SetPath,
    ConfigureIni,
    LinkToApache,
    InitDatabase,
    SetupService,
    VerifyInstall,
    NpmGlobalInstall, // npm install -g로 설치
}

/// A bundled component definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleComponent {
    pub id: String,
    pub name: String,
    pub category: ComponentCategory,
    pub version: String,
    pub file_name: Option<String>,
    pub download_url: Option<String>,
    pub size_bytes: u64,
    pub sha256: Option<String>,
    pub install_path: String,
    pub executable_path: Option<String>,
    pub post_install: Vec<PostInstallAction>,
    pub dependencies: Vec<String>,
    pub description: String,
    pub icon: Option<String>,
}

impl BundleComponent {
    pub fn size_mb(&self) -> f64 {
        self.size_bytes as f64 / 1_048_576.0
    }
}

/// Installation preset (Node.js Frontend / PHP+MySQL Backend)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPreset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub components: Vec<String>,
    pub optional_components: Vec<String>,
}

/// The full bundle manifest (bundles/manifest.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleManifest {
    pub version: String,
    pub components: HashMap<String, BundleComponent>,
    pub presets: HashMap<String, InstallPreset>,
}

impl Default for BundleManifest {
    fn default() -> Self {
        Self::embedded()
    }
}

impl BundleManifest {
    /// Load embedded manifest with default component definitions
    pub fn embedded() -> Self {
        let mut components = HashMap::new();

        // Web Servers
        components.insert("apache".to_string(), BundleComponent {
            id: "apache".to_string(),
            name: "Apache HTTP Server".to_string(),
            category: ComponentCategory::WebServer,
            version: "2.4.62".to_string(),
            file_name: Some("httpd-2.4.62-250111-win64-VS17.zip".to_string()),
            download_url: Some("https://www.apachelounge.com/download/VS17/binaries/httpd-2.4.62-250111-win64-VS17.zip".to_string()),
            size_bytes: 52_428_800, // 50MB
            sha256: None,
            install_path: "runtime/apache".to_string(),
            executable_path: Some("Apache24/bin/httpd.exe".to_string()),
            post_install: vec![PostInstallAction::SetupService, PostInstallAction::VerifyInstall],
            dependencies: vec![],
            description: "Apache 웹 서버".to_string(),
            icon: Some("globe".to_string()),
        });

        // Databases
        components.insert("mariadb".to_string(), BundleComponent {
            id: "mariadb".to_string(),
            name: "MariaDB".to_string(),
            category: ComponentCategory::Database,
            version: "11.4.4".to_string(),
            file_name: Some("mariadb-11.4.4-winx64.zip".to_string()),
            download_url: Some("https://archive.mariadb.org/mariadb-11.4.4/winx64-packages/mariadb-11.4.4-winx64.zip".to_string()),
            size_bytes: 100_000_000, // ~100MB
            sha256: None,
            install_path: "runtime/mariadb".to_string(),
            executable_path: Some("mariadb-11.4.4-winx64/bin/mysqld.exe".to_string()),
            post_install: vec![PostInstallAction::InitDatabase, PostInstallAction::SetupService],
            dependencies: vec![],
            description: "MySQL 호환 오픈소스 데이터베이스".to_string(),
            icon: Some("database".to_string()),
        });

        // Runtimes
        // ID를 inventory_scanner와 일치시킴: "node"
        components.insert("node".to_string(), BundleComponent {
            id: "node".to_string(),
            name: "Node.js".to_string(),
            category: ComponentCategory::Runtime,
            version: "20.18.1".to_string(),
            file_name: Some("node-v20.18.1-win-x64.zip".to_string()),
            download_url: Some("https://nodejs.org/dist/v20.18.1/node-v20.18.1-win-x64.zip".to_string()),
            size_bytes: 30_000_000, // ~30MB
            sha256: None,
            install_path: "runtime/nodejs".to_string(),
            executable_path: Some("node-v20.18.1-win-x64/node.exe".to_string()),
            post_install: vec![PostInstallAction::SetPath, PostInstallAction::VerifyInstall],
            dependencies: vec![],
            description: "JavaScript 런타임".to_string(),
            icon: Some("hexagon".to_string()),
        });

        components.insert("php".to_string(), BundleComponent {
            id: "php".to_string(),
            name: "PHP".to_string(),
            category: ComponentCategory::Runtime,
            version: "8.3.14".to_string(),
            file_name: Some("php-8.3.14-nts-Win32-vs16-x64.zip".to_string()),
            download_url: Some("https://windows.php.net/downloads/releases/php-8.3.14-nts-Win32-vs16-x64.zip".to_string()),
            size_bytes: 31_457_280, // 30MB
            sha256: None,
            install_path: "runtime/php".to_string(),
            executable_path: Some("php.exe".to_string()),
            post_install: vec![PostInstallAction::ConfigureIni, PostInstallAction::LinkToApache, PostInstallAction::SetPath],
            dependencies: vec![],
            description: "서버 사이드 스크립팅 언어".to_string(),
            icon: Some("code".to_string()),
        });

        // Package Managers
        components.insert("pnpm".to_string(), BundleComponent {
            id: "pnpm".to_string(),
            name: "pnpm".to_string(),
            category: ComponentCategory::PackageManager,
            version: "latest".to_string(),
            file_name: None, // npm으로 설치
            download_url: None,
            size_bytes: 1_048_576, // 1MB (npm 설치)
            sha256: None,
            install_path: "".to_string(),
            executable_path: None,
            post_install: vec![PostInstallAction::NpmGlobalInstall],
            dependencies: vec!["node".to_string()],
            description: "빠르고 효율적인 패키지 매니저".to_string(),
            icon: Some("package".to_string()),
        });

        components.insert("composer".to_string(), BundleComponent {
            id: "composer".to_string(),
            name: "Composer".to_string(),
            category: ComponentCategory::PackageManager,
            version: "2.7.1".to_string(),
            file_name: Some("composer.phar".to_string()),
            download_url: Some("https://getcomposer.org/download/2.7.1/composer.phar".to_string()),
            size_bytes: 2_097_152, // 2MB
            sha256: None,
            install_path: "tools/composer".to_string(),
            executable_path: Some("composer.phar".to_string()),
            post_install: vec![PostInstallAction::SetPath],
            dependencies: vec!["php".to_string()],
            description: "PHP 의존성 관리자".to_string(),
            icon: Some("music".to_string()),
        });

        // Dev Tools
        components.insert("git".to_string(), BundleComponent {
            id: "git".to_string(),
            name: "Git".to_string(),
            category: ComponentCategory::DevTool,
            version: "2.47.1".to_string(),
            file_name: Some("PortableGit-2.47.1-64-bit.7z.exe".to_string()),
            download_url: Some("https://github.com/git-for-windows/git/releases/download/v2.47.1.windows.1/PortableGit-2.47.1-64-bit.7z.exe".to_string()),
            size_bytes: 60_000_000, // ~60MB
            sha256: None,
            install_path: "runtime/git".to_string(),
            executable_path: Some("bin/git.exe".to_string()),
            post_install: vec![PostInstallAction::SetPath],
            dependencies: vec![],
            description: "분산 버전 관리 시스템".to_string(),
            icon: Some("git-branch".to_string()),
        });

        components.insert("phpmyadmin".to_string(), BundleComponent {
            id: "phpmyadmin".to_string(),
            name: "phpMyAdmin".to_string(),
            category: ComponentCategory::DevTool,
            version: "5.2.1".to_string(),
            file_name: Some("phpMyAdmin-5.2.1-all-languages.zip".to_string()),
            download_url: Some("https://files.phpmyadmin.net/phpMyAdmin/5.2.1/phpMyAdmin-5.2.1-all-languages.zip".to_string()),
            size_bytes: 15_728_640, // 15MB
            sha256: None,
            install_path: "tools/phpmyadmin".to_string(),
            executable_path: None,
            post_install: vec![PostInstallAction::ConfigureIni],
            dependencies: vec!["php".to_string(), "mariadb".to_string()],
            description: "웹 기반 MySQL 관리 도구".to_string(),
            icon: Some("table".to_string()),
        });

        // Presets (순서: node -> php -> all)
        let mut presets = HashMap::new();

        // Node.js 프리셋
        presets.insert("node".to_string(), InstallPreset {
            id: "node".to_string(),
            name: "Node.js".to_string(),
            description: "React, Vue, Next.js 개발".to_string(),
            icon: "hexagon".to_string(),
            components: vec![
                "node".to_string(),
                "pnpm".to_string(),
                "git".to_string(),
            ],
            optional_components: vec![],
        });

        // PHP 프리셋
        presets.insert("php".to_string(), InstallPreset {
            id: "php".to_string(),
            name: "PHP + MySQL".to_string(),
            description: "Laravel, WordPress, PHP 개발".to_string(),
            icon: "code".to_string(),
            components: vec![
                "apache".to_string(),
                "php".to_string(),
                "mariadb".to_string(),
                "phpmyadmin".to_string(),
                "composer".to_string(),
                "git".to_string(),
            ],
            optional_components: vec![
                "node".to_string(),
                "pnpm".to_string(),
            ],
        });

        // 전체 설치 프리셋
        presets.insert("all".to_string(), InstallPreset {
            id: "all".to_string(),
            name: "전체 설치".to_string(),
            description: "모든 개발 도구 설치".to_string(),
            icon: "layers".to_string(),
            components: vec![
                "node".to_string(),
                "pnpm".to_string(),
                "apache".to_string(),
                "php".to_string(),
                "mariadb".to_string(),
                "phpmyadmin".to_string(),
                "composer".to_string(),
                "git".to_string(),
            ],
            optional_components: vec![],
        });

        BundleManifest {
            version: "1.0.0".to_string(),
            components,
            presets,
        }
    }

    /// Get component by ID
    pub fn get_component(&self, id: &str) -> Option<&BundleComponent> {
        self.components.get(id)
    }

    /// Get preset by ID
    pub fn get_preset(&self, id: &str) -> Option<&InstallPreset> {
        self.presets.get(id)
    }

    /// Get all components in a category
    pub fn get_components_by_category(&self, category: &ComponentCategory) -> Vec<&BundleComponent> {
        self.components
            .values()
            .filter(|c| &c.category == category)
            .collect()
    }

    /// Calculate total size for selected components
    pub fn calculate_total_size(&self, component_ids: &[String]) -> u64 {
        component_ids
            .iter()
            .filter_map(|id| self.components.get(id))
            .map(|c| c.size_bytes)
            .sum()
    }

    /// Get components required by a preset (including dependencies)
    pub fn get_preset_components(&self, preset_id: &str) -> Vec<String> {
        let preset = match self.presets.get(preset_id) {
            Some(p) => p,
            None => return vec![],
        };

        let mut result = preset.components.clone();

        // Resolve dependencies
        let mut to_check = result.clone();
        while let Some(id) = to_check.pop() {
            if let Some(component) = self.components.get(&id) {
                for dep in &component.dependencies {
                    if !result.contains(dep) {
                        result.push(dep.clone());
                        to_check.push(dep.clone());
                    }
                }
            }
        }

        result
    }
}

/// Installation progress tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallProgress {
    pub component_id: String,
    pub component_name: String,
    pub phase: InstallPhase,
    pub progress_percent: u8,
    pub message: String,
    pub error: Option<String>,
}

/// Installation phase
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum InstallPhase {
    Pending,
    Downloading,
    Extracting,
    Configuring,
    Verifying,
    Completed,
    Failed,
}

impl InstallPhase {
    pub fn display_name(&self) -> &'static str {
        match self {
            InstallPhase::Pending => "대기 중",
            InstallPhase::Downloading => "다운로드 중",
            InstallPhase::Extracting => "압축 해제 중",
            InstallPhase::Configuring => "구성 중",
            InstallPhase::Verifying => "검증 중",
            InstallPhase::Completed => "완료",
            InstallPhase::Failed => "실패",
        }
    }
}

/// Overall installation state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallationState {
    pub is_installing: bool,
    pub selected_preset: Option<String>,
    pub selected_components: Vec<String>,
    pub progress: Vec<InstallProgress>,
    pub current_component: Option<String>,
    pub overall_progress: u8,
    pub total_size_bytes: u64,
    pub completed_count: u32,
    pub total_count: u32,
    pub error: Option<String>,
}

impl Default for InstallationState {
    fn default() -> Self {
        Self {
            is_installing: false,
            selected_preset: None,
            selected_components: vec![],
            progress: vec![],
            current_component: None,
            overall_progress: 0,
            total_size_bytes: 0,
            completed_count: 0,
            total_count: 0,
            error: None,
        }
    }
}

/// Installed component record (stored in config)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledComponent {
    pub id: String,
    pub name: String,
    pub version: String,
    pub install_path: String,
    pub installed_at: String,
    pub size_bytes: u64,
}

/// Installation options for a single component
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallOptions {
    pub component_id: String,
    pub version: Option<String>,
    pub auto_configure: bool,
    pub add_to_path: bool,
    pub link_to_apache: bool,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            component_id: String::new(),
            version: None,
            auto_configure: true,
            add_to_path: true,
            link_to_apache: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_manifest() {
        let manifest = BundleManifest::embedded();
        assert!(!manifest.components.is_empty());
        assert!(!manifest.presets.is_empty());

        // Check Node.js preset (ID is "node")
        let node_preset = manifest.get_preset("node").unwrap();
        assert!(node_preset.components.contains(&"node".to_string()));
        assert!(node_preset.components.contains(&"git".to_string()));
    }

    #[test]
    fn test_calculate_total_size() {
        let manifest = BundleManifest::embedded();
        let components = vec!["node".to_string(), "git".to_string()];
        let size = manifest.calculate_total_size(&components);
        assert!(size > 0);
    }

    #[test]
    fn test_preset_components_with_deps() {
        let manifest = BundleManifest::embedded();
        let components = manifest.get_preset_components("php");
        // PHP preset should include composer which depends on php
        assert!(components.contains(&"php".to_string()));
        assert!(components.contains(&"composer".to_string()));
    }
}
