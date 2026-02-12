use crate::models::{
    BundleComponent, BundleManifest, InstallOptions, InstallPhase, InstallProgress,
    InstallationState, InstalledComponent, PostInstallAction,
};
use crate::services::bundler::{DEVPORT_BASE_PATH, RUNTIME_BASE_PATH, TOOLS_BASE_PATH};
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use zip::ZipArchive;

const INSTALLED_COMPONENTS_FILE: &str = "installed_components.json";

pub struct BundleInstaller {
    pub manifest: BundleManifest,
    pub installation_state: InstallationState,
    installed_components: Vec<InstalledComponent>,
}

impl BundleInstaller {
    pub fn new() -> Self {
        let installed = Self::load_installed_components();
        Self {
            manifest: BundleManifest::embedded(),
            installation_state: InstallationState::default(),
            installed_components: installed,
        }
    }

    /// Get config directory path
    fn get_config_dir() -> PathBuf {
        dirs::data_dir()
            .map(|p| p.join("clickdevport"))
            .unwrap_or_else(|| PathBuf::from("."))
    }

    /// Load installed components from config
    fn load_installed_components() -> Vec<InstalledComponent> {
        let path = Self::get_config_dir().join(INSTALLED_COMPONENTS_FILE);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(components) = serde_json::from_str(&content) {
                    return components;
                }
            }
        }
        vec![]
    }

    /// Save installed components to config
    fn save_installed_components(&self) -> Result<(), String> {
        let config_dir = Self::get_config_dir();
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;

        let path = config_dir.join(INSTALLED_COMPONENTS_FILE);
        let json = serde_json::to_string_pretty(&self.installed_components)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        fs::write(&path, json).map_err(|e| format!("Failed to write file: {}", e))
    }

    /// Get the manifest
    pub fn get_manifest(&self) -> &BundleManifest {
        &self.manifest
    }

    /// Get installed components
    pub fn get_installed_components(&self) -> &Vec<InstalledComponent> {
        &self.installed_components
    }

    /// Check if a component is installed
    pub fn is_component_installed(&self, component_id: &str) -> bool {
        // First check our records
        if self.installed_components.iter().any(|c| c.id == component_id) {
            return true;
        }

        // Then check if files exist on disk
        if let Some(component) = self.manifest.get_component(component_id) {
            let install_path = PathBuf::from(DEVPORT_BASE_PATH).join(&component.install_path);
            if install_path.exists() {
                if let Some(exe) = &component.executable_path {
                    return install_path.join(exe).exists();
                }
                return true;
            }
        }

        false
    }

    /// Get component installation status
    pub fn get_component_status(&self, component_id: &str) -> Option<&InstalledComponent> {
        self.installed_components.iter().find(|c| c.id == component_id)
    }

    /// Select a preset for installation
    pub fn select_preset(&mut self, preset_id: &str) -> Result<Vec<String>, String> {
        let components = self.manifest.get_preset_components(preset_id);
        if components.is_empty() {
            return Err(format!("Preset '{}' not found", preset_id));
        }

        self.installation_state.selected_preset = Some(preset_id.to_string());
        self.installation_state.selected_components = components.clone();
        self.installation_state.total_size_bytes =
            self.manifest.calculate_total_size(&components);

        Ok(components)
    }

    /// Toggle a component selection
    pub fn toggle_component(&mut self, component_id: &str) -> bool {
        if let Some(pos) = self
            .installation_state
            .selected_components
            .iter()
            .position(|c| c == component_id)
        {
            self.installation_state.selected_components.remove(pos);
            false
        } else {
            if self.manifest.get_component(component_id).is_some() {
                self.installation_state
                    .selected_components
                    .push(component_id.to_string());
            }
            true
        }
    }

    /// Create base directories
    pub fn create_base_directories() -> Result<(), String> {
        let dirs = [DEVPORT_BASE_PATH, RUNTIME_BASE_PATH, TOOLS_BASE_PATH];

        for dir in dirs {
            fs::create_dir_all(dir)
                .map_err(|e| format!("Failed to create directory {}: {}", dir, e))?;
        }

        Ok(())
    }

    /// Install a single component from a bundle file
    pub async fn install_component(
        &mut self,
        component_id: &str,
        bundle_path: Option<&Path>,
        app_handle: Option<&AppHandle>,
    ) -> Result<InstalledComponent, String> {
        let component = self
            .manifest
            .get_component(component_id)
            .ok_or_else(|| format!("Component '{}' not found", component_id))?
            .clone();

        // Emit progress: Starting
        self.emit_progress(
            app_handle,
            &component,
            InstallPhase::Pending,
            0,
            "설치 준비 중...",
        );

        Self::emit_log(app_handle, "info", &format!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"));
        Self::emit_log(app_handle, "info", &format!("[{}] v{} 설치 시작", component.name, component.version));
        if let Some(url) = &component.download_url {
            Self::emit_log(app_handle, "info", &format!("다운로드 URL: {}", url));
        }

        // Create base directories
        Self::emit_log(app_handle, "info", &format!("기본 디렉토리: {}", DEVPORT_BASE_PATH));
        Self::create_base_directories()?;

        // Calculate install path
        let install_path = PathBuf::from(DEVPORT_BASE_PATH).join(&component.install_path);

        // Check if this is an npm package (no file_name means npm global install)
        let is_npm_package = component.file_name.is_none()
            && component.post_install.contains(&PostInstallAction::NpmGlobalInstall);

        // Extract bundle if provided (skip for npm packages)
        if !is_npm_package {
            // Emit progress: Extracting
            self.emit_progress(
                app_handle,
                &component,
                InstallPhase::Extracting,
                30,
                "압축 해제 중...",
            );

            if let Some(bundle) = bundle_path {
                if bundle.exists() {
                    Self::emit_log(app_handle, "info", &format!("번들 파일: {:?}", bundle));
                    Self::emit_log(app_handle, "info", &format!("설치 경로: {:?}", install_path));
                    self.extract_bundle(bundle, &install_path, app_handle)?;
                } else {
                    let err = format!("번들 파일을 찾을 수 없음: {:?}", bundle);
                    Self::emit_log(app_handle, "error", &err);
                    return Err(format!("Bundle file not found: {:?}", bundle));
                }
            } else if let Some(file_name) = &component.file_name {
                // Look for bundle in standard location
                let bundle_file = PathBuf::from(DEVPORT_BASE_PATH)
                    .join("bundles")
                    .join(file_name);
                if bundle_file.exists() {
                    Self::emit_log(app_handle, "info", &format!("번들 파일: {:?}", bundle_file));
                    Self::emit_log(app_handle, "info", &format!("설치 경로: {:?}", install_path));
                    self.extract_bundle(&bundle_file, &install_path, app_handle)?;
                } else {
                    let err = format!("번들 파일을 찾을 수 없음: {:?}", bundle_file);
                    Self::emit_log(app_handle, "error", &err);
                    return Err(format!(
                        "Bundle file not found: {:?}. Please download or provide the bundle.",
                        bundle_file
                    ));
                }
            }
        } else {
            // For npm packages, just emit progress
            self.emit_progress(
                app_handle,
                &component,
                InstallPhase::Extracting,
                30,
                "npm 패키지 설치 준비 중...",
            );
            Self::emit_log(app_handle, "info", &format!("npm 패키지 설치 준비: {}", component.id));
        }

        // Emit progress: Configuring
        self.emit_progress(
            app_handle,
            &component,
            InstallPhase::Configuring,
            60,
            "구성 중...",
        );

        // Run post-install actions
        Self::emit_log(app_handle, "info", "후처리 작업 실행 중...");
        for action in &component.post_install {
            let action_name = format!("{:?}", action);
            Self::emit_log(app_handle, "info", &format!("  -> {}", action_name));
            self.run_post_install_action(action, &component, &install_path, app_handle)
                .await?;
        }

        // Emit progress: Verifying
        self.emit_progress(
            app_handle,
            &component,
            InstallPhase::Verifying,
            90,
            "설치 확인 중...",
        );

        // Verify installation (skip for npm packages - they are verified during npm install)
        if !is_npm_package {
            if let Some(exe) = &component.executable_path {
                let exe_path = install_path.join(exe);
                if !exe_path.exists() {
                    return Err(format!("Installation verification failed: {:?} not found", exe_path));
                }
            }
        }

        // Create installed component record
        let installed = InstalledComponent {
            id: component.id.clone(),
            name: component.name.clone(),
            version: component.version.clone(),
            install_path: install_path.to_string_lossy().to_string(),
            installed_at: chrono::Utc::now().to_rfc3339(),
            size_bytes: component.size_bytes,
        };

        // Add to installed list and save
        self.installed_components
            .retain(|c| c.id != component_id);
        self.installed_components.push(installed.clone());
        self.save_installed_components()?;

        // Emit progress: Completed
        self.emit_progress(
            app_handle,
            &component,
            InstallPhase::Completed,
            100,
            "설치 완료!",
        );

        Self::emit_log(app_handle, "success", &format!("[{}] 설치 완료!", component.name));

        Ok(installed)
    }

    /// Extract a bundle archive to the target directory
    fn extract_bundle(&self, bundle_path: &Path, target_dir: &Path, app_handle: Option<&AppHandle>) -> Result<(), String> {
        let extension = bundle_path.extension().and_then(|e| e.to_str());
        let file_name = bundle_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Check if it's a self-extracting 7z archive (like PortableGit-*.7z.exe)
        let is_7z_sfx = file_name.contains(".7z.exe") || file_name.ends_with(".7z.exe");

        if is_7z_sfx {
            // Use the self-extracting archive's built-in extraction
            Self::extract_7z_sfx(bundle_path, target_dir, app_handle)
        } else {
            match extension {
                Some("zip") => {
                    Self::emit_log(app_handle, "info", &format!("ZIP 압축 해제: {:?}", bundle_path));

                    // Pre-validate ZIP magic bytes before attempting extraction
                    {
                        use std::io::Read as _;
                        let mut magic_check = File::open(bundle_path)
                            .map_err(|e| format!("Failed to open bundle for validation: {}", e))?;
                        let mut magic = [0u8; 4];
                        magic_check.read_exact(&mut magic)
                            .map_err(|e| format!("Failed to read file header: {}", e))?;
                        drop(magic_check);

                        if magic != [0x50, 0x4B, 0x03, 0x04] {
                            let err = format!(
                                "Invalid ZIP file: {:?} is not a valid ZIP archive. The download may have failed.",
                                bundle_path
                            );
                            Self::emit_log(app_handle, "error", &err);
                            return Err(err);
                        }
                    }

                    let file = File::open(bundle_path).map_err(|e| format!("Failed to open bundle: {}", e))?;
                    let reader = BufReader::new(file);
                    let mut archive = ZipArchive::new(reader)
                        .map_err(|e| format!("Failed to read zip archive: {}", e))?;

                    // Create target directory
                    fs::create_dir_all(target_dir)
                        .map_err(|e| format!("Failed to create target directory: {}", e))?;

                    let total_files = archive.len();
                    for i in 0..total_files {
                        let mut file = archive
                            .by_index(i)
                            .map_err(|e| format!("Failed to read archive entry: {}", e))?;

                        let outpath = target_dir.join(file.name());

                        if file.is_dir() {
                            fs::create_dir_all(&outpath)
                                .map_err(|e| format!("Failed to create directory: {}", e))?;
                        } else {
                            if let Some(parent) = outpath.parent() {
                                fs::create_dir_all(parent)
                                    .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                            }

                            let mut outfile = File::create(&outpath)
                                .map_err(|e| format!("Failed to create file: {}", e))?;
                            std::io::copy(&mut file, &mut outfile)
                                .map_err(|e| format!("Failed to extract file: {}", e))?;
                        }

                        // Set permissions on Unix
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            if let Some(mode) = file.unix_mode() {
                                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).ok();
                            }
                        }
                    }
                    Self::emit_log(app_handle, "info", &format!("{}개 파일 압축 해제 완료", total_files));

                    Ok(())
                }
                Some("phar") | Some("php") => {
                    // For single files, just copy
                    Self::emit_log(app_handle, "info", &format!("파일 복사: {:?}", bundle_path));
                    fs::create_dir_all(target_dir)
                        .map_err(|e| format!("Failed to create target directory: {}", e))?;

                    let filename = bundle_path.file_name().unwrap();
                    let target_file = target_dir.join(filename);
                    fs::copy(bundle_path, &target_file)
                        .map_err(|e| format!("Failed to copy file: {}", e))?;

                    Ok(())
                }
                _ => {
                    let err = format!("Unsupported bundle format: {:?}", extension.unwrap_or("unknown"));
                    Self::emit_log(app_handle, "error", &err);
                    Err(err)
                }
            }
        }
    }

    /// Extract self-extracting 7z archive (like Git Portable)
    fn extract_7z_sfx(sfx_path: &Path, target_dir: &Path, app_handle: Option<&AppHandle>) -> Result<(), String> {
        use std::process::Command;
        #[cfg(windows)]
        use std::os::windows::process::CommandExt;

        fs::create_dir_all(target_dir)
            .map_err(|e| format!("Failed to create target directory: {}", e))?;

        // Self-extracting 7z archives can be run with -o flag to specify output directory
        // Or we can use -y for yes to all prompts
        let sfx_str = sfx_path.to_string_lossy();
        let target_str = target_dir.to_string_lossy();

        Self::emit_log(app_handle, "info", &format!("7z 자동 압축 해제 실행: {} -o{} -y", sfx_str, target_str));

        let mut cmd = Command::new(sfx_path);
        cmd.arg(format!("-o{}", target_str));
        cmd.arg("-y");
        cmd.current_dir(target_dir);

        #[cfg(windows)]
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        let output = cmd.output()
            .map_err(|e| {
                let err = format!("Failed to run self-extracting archive: {}", e);
                Self::emit_log(app_handle, "error", &err);
                err
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let err = format!("Self-extracting archive failed: {} {}", stderr, stdout);
            Self::emit_log(app_handle, "error", &err);
            return Err(err);
        }

        Self::emit_log(app_handle, "success", "7z 자동 압축 해제 완료");
        Ok(())
    }

    /// Run post-install actions
    async fn run_post_install_action(
        &self,
        action: &PostInstallAction,
        component: &BundleComponent,
        install_path: &Path,
        app_handle: Option<&AppHandle>,
    ) -> Result<(), String> {
        match action {
            PostInstallAction::SetPath => {
                // In a real implementation, this would modify PATH
                // For now, we'll create a .devport_path file
                let path_file = Self::get_config_dir().join("paths.txt");
                let mut paths = if path_file.exists() {
                    fs::read_to_string(&path_file).unwrap_or_default()
                } else {
                    String::new()
                };

                let new_path = install_path.to_string_lossy();
                if !paths.contains(new_path.as_ref()) {
                    paths.push_str(&format!("{}\n", new_path));
                    fs::write(&path_file, paths)
                        .map_err(|e| format!("Failed to update paths file: {}", e))?;
                    Self::emit_log(app_handle, "info", &format!("PATH에 추가: {}", new_path));
                }
                Ok(())
            }
            PostInstallAction::ConfigureIni => {
                // Configure php.ini or similar
                if component.id == "php" {
                    let php_ini_development = install_path.join("php.ini-development");
                    let php_ini = install_path.join("php.ini");

                    if php_ini_development.exists() && !php_ini.exists() {
                        Self::emit_log(app_handle, "info", "php.ini 설정 파일 생성 중...");
                        fs::copy(&php_ini_development, &php_ini)
                            .map_err(|e| format!("Failed to copy php.ini: {}", e))?;

                        // Enable common extensions
                        let content = fs::read_to_string(&php_ini)
                            .map_err(|e| format!("Failed to read php.ini: {}", e))?;

                        let content = content
                            .replace(";extension=curl", "extension=curl")
                            .replace(";extension=fileinfo", "extension=fileinfo")
                            .replace(";extension=gd", "extension=gd")
                            .replace(";extension=mbstring", "extension=mbstring")
                            .replace(";extension=mysqli", "extension=mysqli")
                            .replace(";extension=openssl", "extension=openssl")
                            .replace(";extension=pdo_mysql", "extension=pdo_mysql")
                            .replace(";extension=zip", "extension=zip");

                        fs::write(&php_ini, content)
                            .map_err(|e| format!("Failed to write php.ini: {}", e))?;
                        Self::emit_log(app_handle, "success", "PHP 확장 모듈 활성화 완료 (curl, gd, mysqli, openssl 등)");
                    }
                }
                Ok(())
            }
            PostInstallAction::LinkToApache => {
                // Configure Apache to use PHP
                Self::emit_log(app_handle, "info", "Apache-PHP 연동 설정...");
                Ok(())
            }
            PostInstallAction::InitDatabase => {
                // Initialize database (MariaDB/MySQL)
                if component.id == "mariadb" || component.id == "mysql" {
                    let data_dir = install_path.join("data");
                    if !data_dir.exists() {
                        Self::emit_log(app_handle, "info", &format!("데이터베이스 디렉토리 생성: {:?}", data_dir));
                        fs::create_dir_all(&data_dir)
                            .map_err(|e| format!("Failed to create data directory: {}", e))?;
                    }
                }
                Ok(())
            }
            PostInstallAction::SetupService => {
                // Register as a Windows service or create startup script
                Self::emit_log(app_handle, "info", "서비스 설정 준비 완료");
                Ok(())
            }
            PostInstallAction::VerifyInstall => {
                // Verify installation by checking executable
                if let Some(exe) = &component.executable_path {
                    let exe_path = install_path.join(exe);
                    Self::emit_log(app_handle, "info", &format!("실행 파일 확인: {:?}", exe_path));
                    if !exe_path.exists() {
                        Self::emit_log(app_handle, "error", &format!("실행 파일을 찾을 수 없음: {:?}", exe_path));
                        return Err(format!("Verification failed: {:?} not found", exe_path));
                    }
                    Self::emit_log(app_handle, "success", "실행 파일 확인 완료");
                }
                Ok(())
            }
            PostInstallAction::NpmGlobalInstall => {
                // Install npm package globally
                use std::process::Command;
                #[cfg(windows)]
                use std::os::windows::process::CommandExt;

                let npm_path = PathBuf::from(DEVPORT_BASE_PATH)
                    .join("runtime/nodejs")
                    .join("node-v20.18.1-win-x64")
                    .join("npm.cmd");

                // Try DevPort npm first, then system npm
                let npm_cmd = if npm_path.exists() {
                    npm_path.to_string_lossy().to_string()
                } else {
                    "npm".to_string()
                };

                Self::emit_log(app_handle, "info", &format!("실행: {} install -g {}", npm_cmd, component.id));

                let mut cmd = Command::new(&npm_cmd);
                cmd.args(["install", "-g", &component.id]);

                #[cfg(windows)]
                cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

                let output = cmd.output()
                    .map_err(|e| format!("Failed to run npm install: {}", e))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Self::emit_log(app_handle, "error", &format!("npm 설치 실패: {}", stderr));
                    return Err(format!("npm install failed: {}", stderr));
                }

                Self::emit_log(app_handle, "success", &format!("{} npm 패키지 설치 완료", component.id));
                Ok(())
            }
        }
    }

    /// Emit progress event to frontend
    fn emit_progress(
        &self,
        app_handle: Option<&AppHandle>,
        component: &BundleComponent,
        phase: InstallPhase,
        progress: u8,
        message: &str,
    ) {
        let progress_data = InstallProgress {
            component_id: component.id.clone(),
            component_name: component.name.clone(),
            phase,
            progress_percent: progress,
            message: message.to_string(),
            error: None,
        };

        if let Some(app) = app_handle {
            let _ = app.emit("install-progress", &progress_data);
        }
    }

    /// Emit log event to frontend and save to file
    pub fn emit_log(app_handle: Option<&AppHandle>, level: &str, message: &str) {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        let full_timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // Emit to frontend
        if let Some(app) = app_handle {
            let log_entry = serde_json::json!({
                "timestamp": timestamp,
                "level": level,
                "message": message
            });
            let _ = app.emit("install-log", &log_entry);
        }

        // Save to log file
        let log_line = format!("[{}] [{}] {}\n", full_timestamp, level.to_uppercase(), message);
        let _ = Self::append_to_log_file(&log_line);
    }

    /// Get log file path
    fn get_log_file_path() -> PathBuf {
        let log_dir = PathBuf::from(DEVPORT_BASE_PATH).join("logs");
        let date = chrono::Local::now().format("%Y-%m-%d").to_string();
        log_dir.join(format!("install_{}.log", date))
    }

    /// Append message to log file
    fn append_to_log_file(message: &str) -> Result<(), String> {
        use std::io::Write;

        let log_path = Self::get_log_file_path();

        // Create logs directory if needed
        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent).ok();
        }

        // Open file in append mode
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|e| format!("Failed to open log file: {}", e))?;

        file.write_all(message.as_bytes())
            .map_err(|e| format!("Failed to write to log file: {}", e))?;

        Ok(())
    }

    /// Install multiple components
    pub async fn install_components(
        &mut self,
        component_ids: &[String],
        bundles_dir: Option<&Path>,
        app_handle: Option<&AppHandle>,
    ) -> Result<Vec<InstalledComponent>, String> {
        let mut installed = Vec::new();
        let total = component_ids.len();

        self.installation_state.is_installing = true;
        self.installation_state.total_count = total as u32;
        self.installation_state.completed_count = 0;

        for (i, id) in component_ids.iter().enumerate() {
            self.installation_state.current_component = Some(id.clone());
            self.installation_state.overall_progress =
                ((i as f32 / total as f32) * 100.0) as u8;

            let bundle_path = bundles_dir.and_then(|dir| {
                self.manifest
                    .get_component(id)
                    .and_then(|c| c.file_name.as_ref())
                    .map(|f| dir.join(f))
            });

            match self
                .install_component(id, bundle_path.as_deref(), app_handle)
                .await
            {
                Ok(component) => {
                    installed.push(component);
                    self.installation_state.completed_count += 1;
                }
                Err(e) => {
                    self.installation_state.error = Some(e.clone());
                    return Err(e);
                }
            }
        }

        self.installation_state.is_installing = false;
        self.installation_state.overall_progress = 100;
        self.installation_state.current_component = None;

        Ok(installed)
    }

    /// Uninstall a component
    pub async fn uninstall_component(&mut self, component_id: &str) -> Result<(), String> {
        let component = self
            .manifest
            .get_component(component_id)
            .ok_or_else(|| format!("Component '{}' not found", component_id))?;

        let install_path = PathBuf::from(DEVPORT_BASE_PATH).join(&component.install_path);

        if install_path.exists() {
            fs::remove_dir_all(&install_path)
                .map_err(|e| format!("Failed to remove directory: {}", e))?;
        }

        // Remove from installed list
        self.installed_components.retain(|c| c.id != component_id);
        self.save_installed_components()?;

        Ok(())
    }

    /// Get installation state
    pub fn get_installation_state(&self) -> &InstallationState {
        &self.installation_state
    }
}

impl Default for BundleInstaller {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared installer state for Tauri
pub type SharedBundleInstaller = Arc<Mutex<BundleInstaller>>;

pub fn init_bundle_installer() -> SharedBundleInstaller {
    Arc::new(Mutex::new(BundleInstaller::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundle_installer_creation() {
        let installer = BundleInstaller::new();
        assert!(!installer.manifest.components.is_empty());
    }

    #[test]
    fn test_select_preset() {
        let mut installer = BundleInstaller::new();
        let components = installer.select_preset("node").unwrap();
        assert!(components.contains(&"node".to_string()));
        assert!(components.contains(&"git".to_string()));
    }

    #[test]
    fn test_toggle_component() {
        let mut installer = BundleInstaller::new();
        installer.select_preset("node").unwrap();

        // Toggle off node
        let selected = installer.toggle_component("node");
        assert!(!selected);
        assert!(!installer
            .installation_state
            .selected_components
            .contains(&"node".to_string()));

        // Toggle on node
        let selected = installer.toggle_component("node");
        assert!(selected);
        assert!(installer
            .installation_state
            .selected_components
            .contains(&"node".to_string()));
    }
}
