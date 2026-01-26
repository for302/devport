use crate::services::hosts_manager::{HostEntry, HostsManager};
use crate::services::storage::Storage;

#[tauri::command]
pub async fn get_hosts_entries() -> Result<Vec<HostEntry>, String> {
    let manager = HostsManager::new();
    manager.read_entries()
}

#[tauri::command]
pub async fn get_devport_hosts_entries() -> Result<Vec<HostEntry>, String> {
    let manager = HostsManager::new();
    manager.get_devport_entries()
}

#[tauri::command]
pub async fn add_hosts_entry(
    domain: String,
    ip: Option<String>,
    comment: Option<String>,
) -> Result<(), String> {
    HostsManager::validate_domain(&domain)?;

    let manager = HostsManager::new();

    if let Some(conflict) = manager.check_domain_conflict(&domain)? {
        return Err(format!(
            "Domain {} is already registered by another program (IP: {})",
            domain, conflict.ip
        ));
    }

    let ip = ip.unwrap_or_else(|| "127.0.0.1".to_string());
    manager.add_entry(&domain, &ip, comment.as_deref())
}

#[tauri::command]
pub async fn remove_hosts_entry(domain: String) -> Result<(), String> {
    let manager = HostsManager::new();
    manager.remove_entry(&domain)
}

#[tauri::command]
pub async fn update_hosts_entry(
    domain: String,
    new_ip: String,
    new_comment: Option<String>,
) -> Result<(), String> {
    let manager = HostsManager::new();
    manager.update_entry(&domain, &new_ip, new_comment.as_deref())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainCheckResult {
    pub available: bool,
    pub valid: bool,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn check_domain_available(domain: String) -> Result<DomainCheckResult, String> {
    // First validate the domain format
    if let Err(e) = HostsManager::validate_domain(&domain) {
        return Ok(DomainCheckResult {
            available: false,
            valid: false,
            error: Some(e),
        });
    }

    // Then check if it exists
    let manager = HostsManager::new();
    let exists = manager.check_domain_exists(&domain)?;

    Ok(DomainCheckResult {
        available: !exists,
        valid: true,
        error: if exists { Some("Domain already registered".to_string()) } else { None },
    })
}

#[tauri::command]
pub async fn check_domain_conflict(domain: String) -> Result<Option<HostEntry>, String> {
    let manager = HostsManager::new();
    manager.check_domain_conflict(&domain)
}

#[tauri::command]
pub async fn validate_domain(domain: String) -> Result<(), String> {
    HostsManager::validate_domain(&domain)
}

#[tauri::command]
pub async fn suggest_domain(project_name: String) -> Result<String, String> {
    Ok(HostsManager::suggest_domain(&project_name))
}

#[tauri::command]
pub async fn cleanup_devport_hosts() -> Result<(), String> {
    let manager = HostsManager::new();
    manager.cleanup_devport_entries()
}

#[tauri::command]
pub async fn get_hosts_file_path() -> Result<String, String> {
    let manager = HostsManager::new();
    Ok(manager.get_hosts_path().to_string_lossy().to_string())
}

/// Get orphan hosts entries (DevPort entries without matching projects)
#[tauri::command]
pub async fn get_orphan_hosts() -> Result<Vec<HostEntry>, String> {
    let hosts_manager = HostsManager::new();
    let storage = Storage::new().map_err(|e| e.to_string())?;

    // Get all DevPort hosts entries
    let devport_entries = hosts_manager.get_devport_entries()?;

    // Get all registered projects' domains
    let projects = storage.load_projects().map_err(|e| e.to_string())?;
    let project_domains: Vec<String> = projects
        .iter()
        .filter_map(|p| p.domain.clone())
        .filter(|d| !d.is_empty())
        .collect();

    // Find orphan entries (entries not in any project)
    let orphans: Vec<HostEntry> = devport_entries
        .into_iter()
        .filter(|entry| !project_domains.contains(&entry.domain))
        .collect();

    Ok(orphans)
}

/// Delete specific orphan hosts entries
#[tauri::command]
pub async fn delete_orphan_hosts(domains: Vec<String>) -> Result<u32, String> {
    let hosts_manager = HostsManager::new();
    let mut deleted_count = 0u32;

    for domain in domains {
        if let Err(e) = hosts_manager.remove_entry(&domain) {
            eprintln!("Failed to remove orphan host {}: {}", domain, e);
        } else {
            deleted_count += 1;
        }
    }

    Ok(deleted_count)
}
