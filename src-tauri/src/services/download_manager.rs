use crate::models::BundleComponent;
use crate::services::bundle_installer::BundleInstaller;
use crate::services::bundler::DEVPORT_BASE_PATH;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

const DOWNLOADS_DIR: &str = "downloads";
const BUNDLES_DIR: &str = "bundles";

/// Download progress event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub component_id: String,
    pub component_name: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub progress_percent: u8,
    pub speed_bytes_per_sec: u64,
    pub eta_seconds: u64,
    pub status: DownloadStatus,
    pub error: Option<String>,
}

/// Download status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Verifying,
    Completed,
    Failed,
    Cancelled,
}

impl DownloadStatus {
    pub fn display_name(&self) -> &'static str {
        match self {
            DownloadStatus::Pending => "대기 중",
            DownloadStatus::Downloading => "다운로드 중",
            DownloadStatus::Verifying => "검증 중",
            DownloadStatus::Completed => "완료",
            DownloadStatus::Failed => "실패",
            DownloadStatus::Cancelled => "취소됨",
        }
    }
}

/// Download task information
#[derive(Debug, Clone)]
pub struct DownloadTask {
    pub component: BundleComponent,
    pub target_path: PathBuf,
    pub status: DownloadStatus,
    pub downloaded_bytes: u64,
    pub cancel_flag: bool,
}

pub struct DownloadManager {
    client: Client,
    active_downloads: Vec<DownloadTask>,
}

impl DownloadManager {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client,
            active_downloads: Vec::new(),
        }
    }

    /// Get the downloads directory
    pub fn get_downloads_dir() -> PathBuf {
        PathBuf::from(DEVPORT_BASE_PATH).join(DOWNLOADS_DIR)
    }

    /// Get the bundles directory
    pub fn get_bundles_dir() -> PathBuf {
        PathBuf::from(DEVPORT_BASE_PATH).join(BUNDLES_DIR)
    }

    /// Ensure directories exist
    pub fn ensure_directories() -> Result<(), String> {
        let downloads_dir = Self::get_downloads_dir();
        let bundles_dir = Self::get_bundles_dir();

        fs::create_dir_all(&downloads_dir)
            .map_err(|e| format!("Failed to create downloads directory: {}", e))?;
        fs::create_dir_all(&bundles_dir)
            .map_err(|e| format!("Failed to create bundles directory: {}", e))?;

        Ok(())
    }

    /// Check if a bundle file already exists
    pub fn bundle_exists(&self, component: &BundleComponent) -> bool {
        if let Some(file_name) = &component.file_name {
            let bundle_path = Self::get_bundles_dir().join(file_name);
            bundle_path.exists()
        } else {
            false
        }
    }

    /// Get the bundle file path for a component
    pub fn get_bundle_path(&self, component: &BundleComponent) -> Option<PathBuf> {
        component
            .file_name
            .as_ref()
            .map(|f| Self::get_bundles_dir().join(f))
    }

    /// Download a component from URL
    pub async fn download_component(
        &mut self,
        component: &BundleComponent,
        app_handle: Option<&AppHandle>,
    ) -> Result<PathBuf, String> {
        let download_url = component
            .download_url
            .as_ref()
            .ok_or_else(|| "No download URL specified for component".to_string())?;

        let file_name = component
            .file_name
            .as_ref()
            .ok_or_else(|| "No file name specified for component".to_string())?;

        Self::ensure_directories()?;

        let target_path = Self::get_bundles_dir().join(file_name);
        let temp_path = Self::get_downloads_dir().join(format!("{}.download", file_name));

        // Check if already downloaded
        if target_path.exists() {
            // Verify integrity if sha256 is provided
            if let Some(expected_hash) = &component.sha256 {
                if self.verify_file_hash(&target_path, expected_hash)? {
                    return Ok(target_path);
                }
                // Hash mismatch, re-download
                fs::remove_file(&target_path).ok();
            } else {
                return Ok(target_path);
            }
        }

        // Start download
        BundleInstaller::emit_log(app_handle, "info", &format!("[{}] 다운로드 시작", component.name));
        BundleInstaller::emit_log(app_handle, "info", &format!("URL: {}", download_url));

        self.emit_download_progress(
            app_handle,
            component,
            0,
            component.size_bytes,
            DownloadStatus::Downloading,
            None,
        );

        let response = self
            .client
            .get(download_url)
            .send()
            .await
            .map_err(|e| {
                BundleInstaller::emit_log(app_handle, "error", &format!("다운로드 시작 실패: {}", e));
                format!("Failed to start download: {}", e)
            })?;

        if !response.status().is_success() {
            return Err(format!(
                "Download failed with status: {}",
                response.status()
            ));
        }

        let total_size = response
            .content_length()
            .unwrap_or(component.size_bytes);

        // Create temp file
        let mut file = File::create(&temp_path)
            .map_err(|e| format!("Failed to create download file: {}", e))?;

        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();
        let start_time = std::time::Instant::now();

        use futures_util::StreamExt;

        while let Some(chunk_result) = stream.next().await {
            let chunk =
                chunk_result.map_err(|e| format!("Error downloading chunk: {}", e))?;

            file.write_all(&chunk)
                .map_err(|e| format!("Failed to write chunk: {}", e))?;

            downloaded += chunk.len() as u64;

            // Calculate speed and ETA
            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 {
                (downloaded as f64 / elapsed) as u64
            } else {
                0
            };
            let eta = if speed > 0 {
                ((total_size - downloaded) / speed) as u64
            } else {
                0
            };

            // Emit progress every 1%
            let progress = ((downloaded as f64 / total_size as f64) * 100.0) as u8;
            if downloaded % (total_size / 100).max(1) == 0 {
                self.emit_download_progress_full(
                    app_handle,
                    component,
                    downloaded,
                    total_size,
                    progress,
                    speed,
                    eta,
                    DownloadStatus::Downloading,
                    None,
                );
            }
        }

        file.flush()
            .map_err(|e| format!("Failed to flush file: {}", e))?;
        drop(file);

        // Verify downloaded file
        self.emit_download_progress(
            app_handle,
            component,
            downloaded,
            total_size,
            DownloadStatus::Verifying,
            None,
        );

        if let Some(expected_hash) = &component.sha256 {
            if !self.verify_file_hash(&temp_path, expected_hash)? {
                fs::remove_file(&temp_path).ok();
                return Err("Downloaded file hash mismatch".to_string());
            }
        }

        // Move to final location
        fs::rename(&temp_path, &target_path)
            .or_else(|_| fs::copy(&temp_path, &target_path).map(|_| ()))
            .map_err(|e| format!("Failed to move downloaded file: {}", e))?;

        fs::remove_file(&temp_path).ok();

        self.emit_download_progress(
            app_handle,
            component,
            total_size,
            total_size,
            DownloadStatus::Completed,
            None,
        );

        BundleInstaller::emit_log(app_handle, "success", &format!("[{}] 다운로드 완료: {:?}", component.name, target_path));

        Ok(target_path)
    }

    /// Verify file hash
    fn verify_file_hash(&self, path: &Path, expected_hash: &str) -> Result<bool, String> {
        let mut file =
            File::open(path).map_err(|e| format!("Failed to open file for verification: {}", e))?;

        let mut hasher = Sha256::new();
        std::io::copy(&mut file, &mut hasher)
            .map_err(|e| format!("Failed to read file for hashing: {}", e))?;

        let hash = hasher.finalize();
        let hash_hex = format!("{:x}", hash);

        Ok(hash_hex.eq_ignore_ascii_case(expected_hash))
    }

    /// Emit download progress event
    fn emit_download_progress(
        &self,
        app_handle: Option<&AppHandle>,
        component: &BundleComponent,
        downloaded: u64,
        total: u64,
        status: DownloadStatus,
        error: Option<String>,
    ) {
        let progress = if total > 0 {
            ((downloaded as f64 / total as f64) * 100.0) as u8
        } else {
            0
        };

        self.emit_download_progress_full(
            app_handle,
            component,
            downloaded,
            total,
            progress,
            0,
            0,
            status,
            error,
        );
    }

    /// Emit download progress event with full details
    fn emit_download_progress_full(
        &self,
        app_handle: Option<&AppHandle>,
        component: &BundleComponent,
        downloaded: u64,
        total: u64,
        progress: u8,
        speed: u64,
        eta: u64,
        status: DownloadStatus,
        error: Option<String>,
    ) {
        let progress_data = DownloadProgress {
            component_id: component.id.clone(),
            component_name: component.name.clone(),
            downloaded_bytes: downloaded,
            total_bytes: total,
            progress_percent: progress,
            speed_bytes_per_sec: speed,
            eta_seconds: eta,
            status,
            error,
        };

        if let Some(app) = app_handle {
            let _ = app.emit("download-progress", &progress_data);
        }
    }

    /// Download multiple components
    pub async fn download_components(
        &mut self,
        components: &[BundleComponent],
        app_handle: Option<&AppHandle>,
    ) -> Result<Vec<PathBuf>, String> {
        let mut paths = Vec::new();

        for component in components {
            if component.download_url.is_some() {
                let path = self.download_component(component, app_handle).await?;
                paths.push(path);
            }
        }

        Ok(paths)
    }

    /// Clean up incomplete downloads
    pub fn cleanup_incomplete_downloads(&self) -> Result<u64, String> {
        let downloads_dir = Self::get_downloads_dir();
        let mut cleaned_bytes = 0u64;

        if !downloads_dir.exists() {
            return Ok(0);
        }

        for entry in fs::read_dir(&downloads_dir)
            .map_err(|e| format!("Failed to read downloads directory: {}", e))?
        {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "download") {
                    if let Ok(metadata) = fs::metadata(&path) {
                        cleaned_bytes += metadata.len();
                    }
                    fs::remove_file(&path).ok();
                }
            }
        }

        Ok(cleaned_bytes)
    }

    /// Get total downloaded bundle size
    pub fn get_total_bundle_size(&self) -> u64 {
        let bundles_dir = Self::get_bundles_dir();
        let mut total = 0u64;

        if let Ok(entries) = fs::read_dir(&bundles_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        total += metadata.len();
                    }
                }
            }
        }

        total
    }

    /// List available bundle files
    pub fn list_bundle_files(&self) -> Vec<String> {
        let bundles_dir = Self::get_bundles_dir();
        let mut files = Vec::new();

        if let Ok(entries) = fs::read_dir(&bundles_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        if let Some(name) = entry.file_name().to_str() {
                            files.push(name.to_string());
                        }
                    }
                }
            }
        }

        files
    }

    /// Delete a bundle file
    pub fn delete_bundle(&self, file_name: &str) -> Result<(), String> {
        let path = Self::get_bundles_dir().join(file_name);
        if path.exists() {
            fs::remove_file(&path).map_err(|e| format!("Failed to delete bundle: {}", e))?;
        }
        Ok(())
    }
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared download manager for Tauri
pub type SharedDownloadManager = Arc<Mutex<DownloadManager>>;

pub fn init_download_manager() -> SharedDownloadManager {
    Arc::new(Mutex::new(DownloadManager::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_manager_creation() {
        let manager = DownloadManager::new();
        assert!(manager.active_downloads.is_empty());
    }

    #[test]
    fn test_get_directories() {
        let downloads = DownloadManager::get_downloads_dir();
        let bundles = DownloadManager::get_bundles_dir();

        assert!(downloads.to_string_lossy().contains("downloads"));
        assert!(bundles.to_string_lossy().contains("bundles"));
    }
}
