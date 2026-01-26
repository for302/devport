use crate::models::Service;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhpMyAdminStatus {
    pub is_available: bool,
    pub status_code: Option<u16>,
    pub response_time_ms: Option<u64>,
    pub error: Option<String>,
    pub checked_at: String,
    pub url: String,
}

/// Get the configured Apache port from Service model
fn get_apache_port() -> u16 {
    // Use the Apache service default port from the model
    // In the future, this could read from persisted configuration
    Service::apache().port
}

/// Checks if phpMyAdmin endpoint is accessible.
/// Reads Apache port from configuration.
#[tauri::command]
pub async fn check_phpmyadmin_status() -> Result<PhpMyAdminStatus, String> {
    let port = get_apache_port();
    let url = format!("http://localhost:{}/phpmyadmin/", port);
    let start = std::time::Instant::now();
    let checked_at = chrono::Utc::now().to_rfc3339();

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return Ok(PhpMyAdminStatus {
                is_available: false,
                status_code: None,
                response_time_ms: None,
                error: Some(format!("Failed to create HTTP client: {}", e)),
                checked_at,
                url: url.clone(),
            });
        }
    };

    match client.get(&url).send().await {
        Ok(response) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let status = response.status();
            // phpMyAdmin typically returns 200 or redirects (302/303)
            let is_available = status.is_success() || status.is_redirection();

            Ok(PhpMyAdminStatus {
                is_available,
                status_code: Some(status.as_u16()),
                response_time_ms: Some(elapsed),
                error: if !is_available {
                    Some(format!("HTTP {}", status.as_u16()))
                } else {
                    None
                },
                checked_at,
                url,
            })
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            Ok(PhpMyAdminStatus {
                is_available: false,
                status_code: None,
                response_time_ms: Some(elapsed),
                error: Some(e.to_string()),
                checked_at,
                url,
            })
        }
    }
}
