use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const HOSTS_MARKER_BEGIN: &str = "# DevPort BEGIN";
const HOSTS_MARKER_END: &str = "# DevPort END";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostEntry {
    pub domain: String,
    pub ip: String,
    pub comment: Option<String>,
    pub is_devport: bool,
}

pub struct HostsManager {
    hosts_path: PathBuf,
}

impl HostsManager {
    pub fn new() -> Self {
        #[cfg(windows)]
        let hosts_path = PathBuf::from("C:\\Windows\\System32\\drivers\\etc\\hosts");

        #[cfg(not(windows))]
        let hosts_path = PathBuf::from("/etc/hosts");

        Self { hosts_path }
    }

    pub fn get_hosts_path(&self) -> &PathBuf {
        &self.hosts_path
    }

    pub fn read_entries(&self) -> Result<Vec<HostEntry>, String> {
        let content = fs::read_to_string(&self.hosts_path).map_err(|e| e.to_string())?;
        let mut entries = Vec::new();
        let mut in_devport_section = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed == HOSTS_MARKER_BEGIN {
                in_devport_section = true;
                continue;
            }

            if trimmed == HOSTS_MARKER_END {
                in_devport_section = false;
                continue;
            }

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                let ip = parts[0].to_string();
                let domain = parts[1].to_string();
                let comment = if parts.len() > 2 && parts[2].starts_with('#') {
                    Some(parts[2..].join(" ").trim_start_matches('#').trim().to_string())
                } else {
                    None
                };

                entries.push(HostEntry {
                    domain,
                    ip,
                    comment,
                    is_devport: in_devport_section,
                });
            }
        }

        Ok(entries)
    }

    pub fn get_devport_entries(&self) -> Result<Vec<HostEntry>, String> {
        let entries = self.read_entries()?;
        Ok(entries.into_iter().filter(|e| e.is_devport).collect())
    }

    pub fn add_entry(&self, domain: &str, ip: &str, comment: Option<&str>) -> Result<(), String> {
        let mut content = fs::read_to_string(&self.hosts_path).map_err(|e| e.to_string())?;

        let entry_line = if let Some(c) = comment {
            format!("{}\t{}\t# {}", ip, domain, c)
        } else {
            format!("{}\t{}", ip, domain)
        };

        if content.find(HOSTS_MARKER_BEGIN).is_some() {
            if let Some(end_pos) = content.find(HOSTS_MARKER_END) {
                let insert_pos = end_pos;
                content.insert_str(insert_pos, &format!("{}\n", entry_line));
            } else {
                content.push_str(&format!("\n{}\n{}", entry_line, HOSTS_MARKER_END));
            }
        } else {
            content.push_str(&format!(
                "\n{}\n{}\n{}\n",
                HOSTS_MARKER_BEGIN, entry_line, HOSTS_MARKER_END
            ));
        }

        self.write_hosts_content(&content)
    }

    pub fn remove_entry(&self, domain: &str) -> Result<(), String> {
        let content = fs::read_to_string(&self.hosts_path).map_err(|e| e.to_string())?;
        let mut new_lines: Vec<&str> = Vec::new();
        let mut in_devport_section = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed == HOSTS_MARKER_BEGIN {
                in_devport_section = true;
                new_lines.push(line);
                continue;
            }

            if trimmed == HOSTS_MARKER_END {
                in_devport_section = false;
                new_lines.push(line);
                continue;
            }

            if in_devport_section {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 && parts[1] == domain {
                    continue;
                }
            }

            new_lines.push(line);
        }

        let new_content = new_lines.join("\n");
        self.write_hosts_content(&new_content)
    }

    pub fn update_entry(
        &self,
        domain: &str,
        new_ip: &str,
        new_comment: Option<&str>,
    ) -> Result<(), String> {
        self.remove_entry(domain)?;
        self.add_entry(domain, new_ip, new_comment)
    }

    pub fn check_domain_exists(&self, domain: &str) -> Result<bool, String> {
        let entries = self.read_entries()?;
        Ok(entries.iter().any(|e| e.domain == domain))
    }

    pub fn check_domain_conflict(&self, domain: &str) -> Result<Option<HostEntry>, String> {
        let entries = self.read_entries()?;
        Ok(entries.into_iter().find(|e| e.domain == domain && !e.is_devport))
    }

    pub fn validate_domain(domain: &str) -> Result<(), String> {
        if domain.is_empty() {
            return Err("Domain cannot be empty".to_string());
        }

        if !domain.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-') {
            return Err("Domain contains invalid characters".to_string());
        }

        // Safe TLDs for local development
        // - .test: IANA reserved for testing (RECOMMENDED)
        // - .localhost: IANA reserved, always 127.0.0.1
        // - .local: mDNS/Bonjour (may conflict but generally safe)
        // NOTE: .dev is NOT safe - Google owns it and browsers force HTTPS (HSTS preloaded)
        let valid_tlds = ["test", "localhost", "local"];
        let has_valid_tld = valid_tlds.iter().any(|tld| domain.ends_with(&format!(".{}", tld)));

        if !has_valid_tld {
            return Err(
                "Domain must use .test, .localhost, or .local TLD. (.dev is not supported due to HSTS)".to_string()
            );
        }

        // Prevent dangerous domains
        let dangerous_domains = [
            "localhost",
            "google.com", "facebook.com", "github.com", "microsoft.com",
            "apple.com", "amazon.com", "naver.com", "daum.net"
        ];
        let domain_lower = domain.to_lowercase();
        for dangerous in dangerous_domains {
            if domain_lower == dangerous || domain_lower.ends_with(&format!(".{}", dangerous)) {
                return Err(format!("Cannot use '{}' - this domain is reserved or dangerous", domain));
            }
        }

        Ok(())
    }

    pub fn suggest_domain(project_name: &str) -> String {
        let safe_name = project_name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>();

        format!("{}.test", safe_name)
    }

    pub fn cleanup_devport_entries(&self) -> Result<(), String> {
        let content = fs::read_to_string(&self.hosts_path).map_err(|e| e.to_string())?;
        let mut new_lines: Vec<&str> = Vec::new();
        let mut in_devport_section = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed == HOSTS_MARKER_BEGIN {
                in_devport_section = true;
                continue;
            }

            if trimmed == HOSTS_MARKER_END {
                in_devport_section = false;
                continue;
            }

            if !in_devport_section {
                new_lines.push(line);
            }
        }

        let new_content = new_lines.join("\n");
        self.write_hosts_content(&new_content)
    }

    /// Write content to hosts file, falling back to elevated write on permission error
    fn write_hosts_content(&self, content: &str) -> Result<(), String> {
        match fs::write(&self.hosts_path, content) {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                // Try elevated write via PowerShell UAC prompt
                self.write_hosts_elevated(content)
            }
            Err(e) => Err(e.to_string()),
        }
    }

    /// Write hosts file content with elevated (admin) privileges via UAC
    #[cfg(windows)]
    fn write_hosts_elevated(&self, content: &str) -> Result<(), String> {
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("devport_hosts_temp");

        fs::write(&temp_path, content)
            .map_err(|e| format!("Failed to write temp file: {}", e))?;

        let hosts_path = self.hosts_path.to_string_lossy().replace('\\', "\\\\");
        let temp_path_str = temp_path.to_string_lossy().replace('\\', "\\\\");

        let ps_script = format!(
            "Copy-Item '{}' '{}' -Force",
            temp_path_str, hosts_path
        );

        let output = Command::new("powershell")
            .args([
                "-Command",
                &format!(
                    "Start-Process powershell -Verb RunAs -Wait -ArgumentList '-Command {}' -WindowStyle Hidden",
                    ps_script
                ),
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("Failed to request admin privileges: {}", e))?;

        // Cleanup temp file
        let _ = fs::remove_file(&temp_path);

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to write hosts file with admin privileges: {}", stderr))
        }
    }

    #[cfg(not(windows))]
    fn write_hosts_elevated(&self, _content: &str) -> Result<(), String> {
        Err("Elevated write is only supported on Windows".to_string())
    }
}

impl Default for HostsManager {
    fn default() -> Self {
        Self::new()
    }
}
