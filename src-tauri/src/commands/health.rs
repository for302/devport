use crate::services::health_checker::{HealthChecker, HealthStatus};

#[tauri::command]
pub async fn check_health(project_id: String, url: String) -> Result<HealthStatus, String> {
    Ok(HealthChecker::check_health(&project_id, &url, 5).await)
}
