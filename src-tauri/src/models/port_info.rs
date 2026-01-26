use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortInfo {
    pub port: u16,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
    pub project_id: Option<String>,
    pub protocol: String,
    pub state: String,
    pub local_address: String,
}

impl PortInfo {
    pub fn new(port: u16, protocol: String, state: String, local_address: String) -> Self {
        Self {
            port,
            pid: None,
            process_name: None,
            project_id: None,
            protocol,
            state,
            local_address,
        }
    }
}
