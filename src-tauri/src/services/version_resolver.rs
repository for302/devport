use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Result of dynamically resolving a component's latest version
#[derive(Debug, Clone)]
pub struct ResolvedVersion {
    pub version: String,
    pub download_url: String,
    pub file_name: String,
    pub size_bytes: Option<u64>,
}

/// Resolves the latest version and download URL for each component
pub struct VersionResolver {
    client: Client,
}

// --- JSON response models for various APIs ---

#[derive(Deserialize)]
struct NodeDistEntry {
    version: String,
    lts: NodeLts,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum NodeLts {
    Name(#[allow(dead_code)] String),
    False(#[allow(dead_code)] bool),
}

impl NodeLts {
    fn is_lts(&self) -> bool {
        matches!(self, NodeLts::Name(_))
    }
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Deserialize)]
struct PhpMyAdminVersion {
    version: String,
}

#[derive(Deserialize)]
struct ComposerVersions {
    stable: Vec<ComposerRelease>,
}

#[derive(Deserialize)]
struct ComposerRelease {
    version: String,
}

impl VersionResolver {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .user_agent("DevPort-Manager/0.6")
            .build()
            .unwrap_or_else(|_| Client::new());
        Self { client }
    }

    /// Resolve the latest version for a component by its ID.
    /// Returns None on failure (caller should use hardcoded fallback).
    pub async fn resolve(&self, component_id: &str) -> Option<ResolvedVersion> {
        let result = match component_id {
            "node" => self.resolve_nodejs().await,
            "git" => self.resolve_git().await,
            "phpmyadmin" => self.resolve_phpmyadmin().await,
            "composer" => self.resolve_composer().await,
            "mariadb" => self.resolve_mariadb().await,
            "php" => self.resolve_php().await,
            "apache" => self.resolve_apache().await,
            _ => None,
        };

        if result.is_none() {
            eprintln!(
                "[VersionResolver] 동적 해석 실패: {}. fallback URL 사용",
                component_id
            );
        }

        result
    }

    // ── Node.js ──────────────────────────────────────────────────────

    async fn resolve_nodejs(&self) -> Option<ResolvedVersion> {
        let entries: Vec<NodeDistEntry> = self
            .client
            .get("https://nodejs.org/dist/index.json")
            .send()
            .await
            .ok()?
            .json()
            .await
            .ok()?;

        // Find latest LTS release
        let entry = entries.iter().find(|e| e.lts.is_lts())?;
        let version = entry.version.trim_start_matches('v');
        let file_name = format!("node-v{}-win-x64.zip", version);
        let download_url = format!(
            "https://nodejs.org/dist/v{}/{}",
            version, file_name
        );

        Some(ResolvedVersion {
            version: version.to_string(),
            download_url,
            file_name,
            size_bytes: None,
        })
    }

    // ── Git ──────────────────────────────────────────────────────────

    async fn resolve_git(&self) -> Option<ResolvedVersion> {
        let release: GithubRelease = self
            .client
            .get("https://api.github.com/repos/git-for-windows/git/releases/latest")
            .send()
            .await
            .ok()?
            .json()
            .await
            .ok()?;

        // Find the PortableGit 64-bit self-extracting archive
        let asset = release
            .assets
            .iter()
            .find(|a| a.name.contains("PortableGit") && a.name.contains("64-bit") && a.name.ends_with(".7z.exe"))?;

        // Extract version from tag like "v2.47.1.windows.1"
        let version = release
            .tag_name
            .trim_start_matches('v')
            .split(".windows")
            .next()
            .unwrap_or(&release.tag_name)
            .to_string();

        Some(ResolvedVersion {
            version,
            download_url: asset.browser_download_url.clone(),
            file_name: asset.name.clone(),
            size_bytes: Some(asset.size),
        })
    }

    // ── phpMyAdmin ───────────────────────────────────────────────────

    async fn resolve_phpmyadmin(&self) -> Option<ResolvedVersion> {
        let info: PhpMyAdminVersion = self
            .client
            .get("https://www.phpmyadmin.net/home_page/version.json")
            .send()
            .await
            .ok()?
            .json()
            .await
            .ok()?;

        let version = &info.version;
        let file_name = format!("phpMyAdmin-{}-all-languages.zip", version);
        let download_url = format!(
            "https://files.phpmyadmin.net/phpMyAdmin/{}/{}",
            version, file_name
        );

        Some(ResolvedVersion {
            version: version.clone(),
            download_url,
            file_name,
            size_bytes: None,
        })
    }

    // ── Composer ─────────────────────────────────────────────────────

    async fn resolve_composer(&self) -> Option<ResolvedVersion> {
        let versions: ComposerVersions = self
            .client
            .get("https://getcomposer.org/versions")
            .send()
            .await
            .ok()?
            .json()
            .await
            .ok()?;

        let latest = versions.stable.first()?;
        let version = &latest.version;
        let file_name = "composer.phar".to_string();
        let download_url = format!(
            "https://getcomposer.org/download/{}/composer.phar",
            version
        );

        Some(ResolvedVersion {
            version: version.clone(),
            download_url,
            file_name,
            size_bytes: None,
        })
    }

    // ── MariaDB ──────────────────────────────────────────────────────

    async fn resolve_mariadb(&self) -> Option<ResolvedVersion> {
        // Fetch the archive directory listing and find latest 11.4.x LTS
        let html = self
            .client
            .get("https://archive.mariadb.org/")
            .send()
            .await
            .ok()?
            .text()
            .await
            .ok()?;

        let re = Regex::new(r#"mariadb-(11\.4\.\d+)/"#).ok()?;
        let mut versions: Vec<&str> = re
            .captures_iter(&html)
            .filter_map(|c| c.get(1).map(|m| m.as_str()))
            .collect();

        // Sort by version descending
        versions.sort_by(|a, b| {
            version_cmp(b, a)
        });

        let version = versions.first()?;
        let file_name = format!("mariadb-{}-winx64.zip", version);
        let download_url = format!(
            "https://archive.mariadb.org/mariadb-{}/winx64-packages/{}",
            version, file_name
        );

        Some(ResolvedVersion {
            version: version.to_string(),
            download_url,
            file_name,
            size_bytes: None,
        })
    }

    // ── PHP ──────────────────────────────────────────────────────────

    async fn resolve_php(&self) -> Option<ResolvedVersion> {
        let html = self
            .client
            .get("https://windows.php.net/downloads/releases/")
            .send()
            .await
            .ok()?
            .text()
            .await
            .ok()?;

        // Match pattern: php-X.Y.Z-nts-Win32-vsNN-x64.zip
        let re = Regex::new(r#"(php-(\d+\.\d+\.\d+)-nts-Win32-vs\d+-x64\.zip)"#).ok()?;

        let mut best: Option<(String, String)> = None; // (file_name, version)
        for cap in re.captures_iter(&html) {
            if let (Some(fname_m), Some(ver_m)) = (cap.get(1), cap.get(2)) {
                let fname = fname_m.as_str();
                let ver = ver_m.as_str();
                if best.is_none() || version_cmp(ver, best.as_ref().unwrap().1.as_str()) == std::cmp::Ordering::Greater {
                    best = Some((fname.to_string(), ver.to_string()));
                }
            }
        }

        let (file_name, version) = best?;
        let download_url = format!(
            "https://windows.php.net/downloads/releases/{}",
            file_name
        );

        Some(ResolvedVersion {
            version,
            download_url,
            file_name,
            size_bytes: None,
        })
    }

    // ── Apache ───────────────────────────────────────────────────────

    async fn resolve_apache(&self) -> Option<ResolvedVersion> {
        let html = self
            .client
            .get("https://www.apachelounge.com/download/")
            .send()
            .await
            .ok()?
            .text()
            .await
            .ok()?;

        // Match pattern: httpd-X.Y.ZZ-YYMMDD-win64-VSNN.zip
        let re = Regex::new(r#"(httpd-([\d.]+)-\d+-win64-VS\d+\.zip)"#).ok()?;

        let mut best: Option<(String, String)> = None;
        for cap in re.captures_iter(&html) {
            if let (Some(fname_m), Some(ver_m)) = (cap.get(1), cap.get(2)) {
                let fname = fname_m.as_str();
                let ver = ver_m.as_str();
                if best.is_none() || version_cmp(ver, best.as_ref().unwrap().1.as_str()) == std::cmp::Ordering::Greater {
                    best = Some((fname.to_string(), ver.to_string()));
                }
            }
        }

        let (file_name, version) = best?;

        // Determine VS version from filename (e.g. VS17, VS18)
        let vs_re = Regex::new(r#"(VS\d+)"#).ok()?;
        let vs = vs_re.find(&file_name)?.as_str().to_string();

        let download_url = format!(
            "https://www.apachelounge.com/download/{}/binaries/{}",
            vs, file_name
        );

        Some(ResolvedVersion {
            version,
            download_url,
            file_name,
            size_bytes: None,
        })
    }
}

/// Compare two dotted version strings (e.g. "2.4.62" vs "2.4.66")
fn version_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |s: &str| -> Vec<u64> {
        s.split('.')
            .filter_map(|p| p.parse::<u64>().ok())
            .collect()
    };
    let va = parse(a);
    let vb = parse(b);
    va.cmp(&vb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_cmp() {
        assert_eq!(version_cmp("2.4.66", "2.4.62"), std::cmp::Ordering::Greater);
        assert_eq!(version_cmp("11.4.4", "11.4.4"), std::cmp::Ordering::Equal);
        assert_eq!(version_cmp("8.3.14", "8.4.2"), std::cmp::Ordering::Less);
        assert_eq!(version_cmp("22.0.0", "20.18.1"), std::cmp::Ordering::Greater);
    }

    #[test]
    fn test_resolver_creation() {
        let resolver = VersionResolver::new();
        // Just verify it doesn't panic
        let _ = resolver;
    }
}
