use crate::models::process_info::ProcessInfo;
use std::collections::HashMap;

pub struct AppState {
    pub running_processes: HashMap<String, ProcessInfo>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            running_processes: HashMap::new(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
