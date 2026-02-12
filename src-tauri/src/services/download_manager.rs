use crate::models::BundleComponent;
use crate::services::bundle_installer::BundleInstaller;
use crate::services::bundler::DEVPORT_BASE_PATH;
use crate::services::version_resolver::VersionResolver;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
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

    /// Download a component from URL, with dynamic version resolution and file validation
    pub async fn download_component(
        &mut self,
        component: &BundleComponent,
        app_handle: Option<&AppHandle>,
    ) -> Result<PathBuf, String> {
        // --- Dynamic version resolution ---
        let mut download_url = component
            .download_url
            .clone()
            .ok_or_else(|| "No download URL specified for component".to_string())?;

        let mut file_name = component
            .file_name
            .clone()
            .ok_or_else(|| "No file name specified for component".to_string())?;

        if component.resolve_strategy.is_some() {
            let resolver = VersionResolver::new();
            match resolver.resolve(&component.id).await {
                Some(resolved) => {
                    BundleInstaller::emit_log(
                        app_handle,
                        "info",
                        &format!("[{}] 최신 버전 감지: v{}", component.name, resolved.version),
                    );
                    download_url = resolved.download_url;
                    file_name = resolved.file_name;
                }
                None => {
                    BundleInstaller::emit_log(
                        app_handle,
                        "warn",
                        &format!("[{}] 동적 해석 실패, fallback URL 사용", component.name),
                    );
                }
            }
        }

        Self::ensure_directories()?;

        let target_path = Self::get_bundles_dir().join(&file_name);
        let temp_path = Self::get_downloads_dir().join(format!("{}.download", file_name));

        // Determine expected extension for validation
        let extension = Path::new(&file_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();

        // Check if already downloaded
        if target_path.exists() {
            if let Some(expected_hash) = &component.sha256 {
                if self.verify_file_hash(&target_path, expected_hash)? {
                    return Ok(target_path);
                }
                // Hash mismatch, re-download
                fs::remove_file(&target_path).ok();
            } else {
                // No SHA256 — validate with magic bytes + minimum size
                match Self::validate_downloaded_file(&target_path, &extension) {
                    Ok(()) => return Ok(target_path),
                    Err(e) => {
                        BundleInstaller::emit_log(
                            app_handle,
                            "warn",
                            &format!("캐시 파일 검증 실패 ({}), 재다운로드", e),
                        );
                        fs::remove_file(&target_path).ok();
                    }
                }
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
            .get(&download_url)
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

        // --- Post-download file validation ---
        Self::validate_downloaded_file(&temp_path, &extension).map_err(|e| {
            fs::remove_file(&temp_path).ok();
            BundleInstaller::emit_log(app_handle, "error", &format!("파일 검증 실패: {}", e));
            e
        })?;

        // Verify SHA256 if provided
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

    /// Validate a downloaded file's integrity (magic bytes + minimum size)
    fn validate_downloaded_file(path: &Path, expected_extension: &str) -> Result<(), String> {
        let metadata = fs::metadata(path)
            .map_err(|e| format!("파일 메타데이터 읽기 실패: {}", e))?;

        // 1. Minimum size check (< 10KB likely an error page)
        if metadata.len() < 10_240 {
            return Err(format!(
                "다운로드된 파일이 너무 작습니다 ({}바이트). 서버 에러 페이지일 수 있습니다.",
                metadata.len()
            ));
        }

        // 2. ZIP magic byte check (PK\x03\x04)
        if expected_extension == "zip" {
            let mut file = File::open(path)
                .map_err(|e| format!("파일 열기 실패: {}", e))?;
            let mut magic = [0u8; 4];
            file.read_exact(&mut magic)
                .map_err(|e| format!("파일 헤더 읽기 실패: {}", e))?;

            if magic != [0x50, 0x4B, 0x03, 0x04] {
                // Check if it's HTML (error page)
                file.seek(SeekFrom::Start(0))
                    .map_err(|e| format!("파일 탐색 실패: {}", e))?;
                let mut header = [0u8; 256];
                let n = file.read(&mut header).unwrap_or(0);
                let header_str = String::from_utf8_lossy(&header[..n]);
                if header_str.contains("<html") || header_str.contains("<!DOCTYPE") || header_str.contains("<HTML") {
                    return Err("다운로드 실패: 서버가 HTML 에러 페이지를 반환했습니다. URL이 유효하지 않을 수 있습니다.".into());
                }
                return Err("다운로드된 파일이 유효한 ZIP 형식이 아닙니다.".into());
            }
        }

        // 3. EXE magic byte check (MZ) for self-extracting archives
        if expected_extension == "exe" {
            let mut file = File::open(path)
                .map_err(|e| format!("파일 열기 실패: {}", e))?;
            let mut magic = [0u8; 2];
            file.read_exact(&mut magic)
                .map_err(|e| format!("파일 헤더 읽기 실패: {}", e))?;

            if magic != [0x4D, 0x5A] {
                // Check if it's HTML
                file.seek(SeekFrom::Start(0))
                    .map_err(|e| format!("파일 탐색 실패: {}", e))?;
                let mut header = [0u8; 256];
                let n = file.read(&mut header).unwrap_or(0);
                let header_str = String::from_utf8_lossy(&header[..n]);
                if header_str.contains("<html") || header_str.contains("<!DOCTYPE") {
                    return Err("다운로드 실패: 서버가 HTML 에러 페이지를 반환했습니다.".into());
                }
                return Err("다운로드된 파일이 유효한 실행 파일이 아닙니다.".into());
            }
        }

        Ok(())
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

    #[test]
    fn test_validate_small_file() {
        // Create a tiny file that should fail validation
        let temp_dir = std::env::temp_dir().join("devport_test_validate");
        let _ = fs::create_dir_all(&temp_dir);
        let small_file = temp_dir.join("small.zip");
        fs::write(&small_file, b"too small").unwrap();

        let result = DownloadManager::validate_downloaded_file(&small_file, "zip");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("너무 작습니다"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_validate_html_as_zip() {
        let temp_dir = std::env::temp_dir().join("devport_test_html");
        let _ = fs::create_dir_all(&temp_dir);
        let html_file = temp_dir.join("fake.zip");
        // Write enough bytes to pass size check but fail magic byte check
        let mut content = b"<!DOCTYPE html><html><body>404 Not Found</body></html>".to_vec();
        content.resize(11_000, b' '); // pad to > 10KB
        fs::write(&html_file, &content).unwrap();

        let result = DownloadManager::validate_downloaded_file(&html_file, "zip");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("HTML"));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
