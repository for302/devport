use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProcessStatus {
    Running,
    Stopped,
    Starting,
    Stopping,
    Error,
}

impl Default for ProcessStatus {
    fn default() -> Self {
        ProcessStatus::Stopped
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
    pub project_id: String,
    pub pid: u32,
    pub status: ProcessStatus,
    pub started_at: Option<String>,
    pub port: u16,
    pub cpu_usage: Option<f32>,
    pub memory_usage: Option<u64>,
}

impl ProcessInfo {
    pub fn new(project_id: String, pid: u32, port: u16) -> Self {
        Self {
            project_id,
            pid,
            status: ProcessStatus::Running,
            started_at: Some(chrono::Utc::now().to_rfc3339()),
            port,
            cpu_usage: None,
            memory_usage: None,
        }
    }
}
