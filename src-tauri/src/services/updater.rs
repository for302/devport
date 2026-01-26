use serde::{Deserialize, Serialize};
use thiserror::Error;

/// GitHub repository configuration for update checking
const GITHUB_OWNER: &str = "anthropics";
const GITHUB_REPO: &str = "devport-manager";

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Failed to parse response: {0}")]
    ParseError(String),
    #[error("No release found")]
    NoReleaseFound,
    #[error("Invalid version format: {0}")]
    InvalidVersion(String),
    #[error("Download failed: {0}")]
    DownloadError(String),
    #[error("IO error: {0}")]
    IoError(String),
}

/// Information about an available update
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub download_url: String,
    pub release_notes: String,
    pub published_at: String,
    pub is_prerelease: bool,
    pub asset_name: Option<String>,
    pub asset_size: Option<u64>,
}

/// GitHub release response structure
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    published_at: String,
    prerelease: bool,
    html_url: String,
    assets: Vec<GitHubAsset>,
}

/// GitHub asset structure
#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
    content_type: String,
}

/// Result of an update check
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckResult {
    pub update_available: bool,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub update_info: Option<UpdateInfo>,
    pub error: Option<String>,
    pub checked_at: String,
}

/// Download progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percentage: f64,
}

/// Update manager for checking and downloading updates
pub struct UpdateManager {
    client: reqwest::Client,
    github_owner: String,
    github_repo: String,
}

impl UpdateManager {
    /// Create a new UpdateManager instance
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent(format!("DevPort-Manager/{}", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            client,
            github_owner: GITHUB_OWNER.to_string(),
            github_repo: GITHUB_REPO.to_string(),
        }
    }

    /// Create UpdateManager with custom repository
    pub fn with_repo(owner: &str, repo: &str) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(format!("DevPort-Manager/{}", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            client,
            github_owner: owner.to_string(),
            github_repo: repo.to_string(),
        }
    }

    /// Get the current application version
    pub fn get_current_version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    /// Check for available updates
    pub async fn check_for_updates(&self) -> UpdateCheckResult {
        let current_version = self.get_current_version();
        let checked_at = chrono::Utc::now().to_rfc3339();

        match self.fetch_latest_release().await {
            Ok(update_info) => {
                let update_available =
                    self.compare_versions(&current_version, &update_info.version);

                UpdateCheckResult {
                    update_available,
                    current_version,
                    latest_version: Some(update_info.version.clone()),
                    update_info: if update_available {
                        Some(update_info)
                    } else {
                        None
                    },
                    error: None,
                    checked_at,
                }
            }
            Err(e) => UpdateCheckResult {
                update_available: false,
                current_version,
                latest_version: None,
                update_info: None,
                error: Some(e.to_string()),
                checked_at,
            },
        }
    }

    /// Fetch the latest release from GitHub
    async fn fetch_latest_release(&self) -> Result<UpdateInfo, UpdateError> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            self.github_owner, self.github_repo
        );

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| UpdateError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            if response.status().as_u16() == 404 {
                return Err(UpdateError::NoReleaseFound);
            }
            return Err(UpdateError::NetworkError(format!(
                "GitHub API returned status: {}",
                response.status()
            )));
        }

        let release: GitHubRelease = response
            .json()
            .await
            .map_err(|e| UpdateError::ParseError(e.to_string()))?;

        // Find the appropriate asset for Windows
        let windows_asset = release.assets.iter().find(|a| {
            let name = a.name.to_lowercase();
            name.ends_with(".msi") || name.ends_with(".exe") || name.contains("windows")
        });

        let version = release.tag_name.trim_start_matches('v').to_string();

        Ok(UpdateInfo {
            version,
            download_url: windows_asset
                .map(|a| a.browser_download_url.clone())
                .unwrap_or(release.html_url),
            release_notes: release.body.unwrap_or_default(),
            published_at: release.published_at,
            is_prerelease: release.prerelease,
            asset_name: windows_asset.map(|a| a.name.clone()),
            asset_size: windows_asset.map(|a| a.size),
        })
    }

    /// Compare two semantic versions
    /// Returns true if latest_version is newer than current_version
    pub fn compare_versions(&self, current: &str, latest: &str) -> bool {
        let current_parts = Self::parse_version(current);
        let latest_parts = Self::parse_version(latest);

        match (current_parts, latest_parts) {
            (Some(current), Some(latest)) => {
                for (c, l) in current.iter().zip(latest.iter()) {
                    if l > c {
                        return true;
                    }
                    if c > l {
                        return false;
                    }
                }
                // If we get here, they're equal up to the shortest length
                latest.len() > current.len()
            }
            _ => false,
        }
    }

    /// Parse a version string into numeric parts
    fn parse_version(version: &str) -> Option<Vec<u32>> {
        let cleaned = version.trim_start_matches('v');
        // Handle versions with suffixes like "1.0.0-beta"
        let base_version = cleaned.split('-').next()?;

        base_version
            .split('.')
            .map(|s| s.parse::<u32>().ok())
            .collect()
    }

    /// Download the update to a temporary location
    pub async fn download_update(&self, update_info: &UpdateInfo) -> Result<String, UpdateError> {
        let download_dir = dirs::download_dir()
            .or_else(dirs::home_dir)
            .ok_or_else(|| UpdateError::IoError("Could not find download directory".to_string()))?;

        let file_name = update_info
            .asset_name
            .clone()
            .unwrap_or_else(|| format!("devport-manager-{}.exe", update_info.version));

        let file_path = download_dir.join(&file_name);

        let response = self
            .client
            .get(&update_info.download_url)
            .send()
            .await
            .map_err(|e| UpdateError::DownloadError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(UpdateError::DownloadError(format!(
                "Download failed with status: {}",
                response.status()
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| UpdateError::DownloadError(e.to_string()))?;

        tokio::fs::write(&file_path, &bytes)
            .await
            .map_err(|e| UpdateError::IoError(e.to_string()))?;

        Ok(file_path.to_string_lossy().to_string())
    }

    /// Get the GitHub releases URL for manual download
    pub fn get_releases_url(&self) -> String {
        format!(
            "https://github.com/{}/{}/releases",
            self.github_owner, self.github_repo
        )
    }
}

impl Default for UpdateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_versions() {
        let manager = UpdateManager::new();

        assert!(manager.compare_versions("0.1.0", "0.2.0"));
        assert!(manager.compare_versions("0.1.0", "0.1.1"));
        assert!(manager.compare_versions("0.1.0", "1.0.0"));
        assert!(manager.compare_versions("1.0.0", "1.0.1"));
        assert!(manager.compare_versions("1.9.9", "2.0.0"));

        assert!(!manager.compare_versions("0.2.0", "0.1.0"));
        assert!(!manager.compare_versions("1.0.0", "0.9.9"));
        assert!(!manager.compare_versions("1.0.0", "1.0.0"));
    }

    #[test]
    fn test_parse_version() {
        assert_eq!(
            UpdateManager::parse_version("1.2.3"),
            Some(vec![1, 2, 3])
        );
        assert_eq!(
            UpdateManager::parse_version("v1.2.3"),
            Some(vec![1, 2, 3])
        );
        assert_eq!(
            UpdateManager::parse_version("1.2.3-beta"),
            Some(vec![1, 2, 3])
        );
        assert_eq!(UpdateManager::parse_version("0.1.0"), Some(vec![0, 1, 0]));
    }
}
