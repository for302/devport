use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct LogManager {
    pub base_path: PathBuf,
    pub max_file_size: u64,
    pub max_files: u32,
    pub retention_days: u32,
}

impl LogManager {
    pub fn new() -> Self {
        let base_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("DevPort")
            .join("logs");

        Self {
            base_path,
            max_file_size: 50 * 1024 * 1024,
            max_files: 5,
            retention_days: 30,
        }
    }

    pub fn with_base_path(base_path: PathBuf) -> Self {
        Self {
            base_path,
            max_file_size: 50 * 1024 * 1024,
            max_files: 5,
            retention_days: 30,
        }
    }

    pub fn ensure_directories(&self) -> std::io::Result<()> {
        let dirs = ["apache", "mariadb", "projects"];
        for dir in dirs {
            fs::create_dir_all(self.base_path.join(dir))?;
        }
        Ok(())
    }

    pub fn get_log_path(&self, service_name: &str, log_type: &str) -> PathBuf {
        self.base_path.join(service_name).join(format!("{}.log", log_type))
    }

    pub fn get_project_log_path(&self, project_name: &str, log_type: &str) -> PathBuf {
        self.base_path
            .join("projects")
            .join(project_name)
            .join(format!("{}.log", log_type))
    }

    pub fn write_log(&self, path: &Path, message: &str) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        self.rotate_if_needed(path)?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        writeln!(file, "[{}] {}", timestamp, message)?;

        Ok(())
    }

    pub fn write_log_entry(&self, path: &Path, entry: &LogEntry) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        self.rotate_if_needed(path)?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        writeln!(
            file,
            "[{}] [{}] [{}] {}",
            entry.timestamp, entry.level, entry.source, entry.message
        )?;

        Ok(())
    }

    fn rotate_if_needed(&self, path: &Path) -> std::io::Result<()> {
        if !path.exists() {
            return Ok(());
        }

        let metadata = fs::metadata(path)?;
        if metadata.len() < self.max_file_size {
            return Ok(());
        }

        self.rotate_log(path)
    }

    fn rotate_log(&self, path: &Path) -> std::io::Result<()> {
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let ext = path.extension().unwrap_or_default().to_string_lossy();
        let parent = path.parent().unwrap_or(Path::new("."));

        for i in (1..self.max_files).rev() {
            let old_path = parent.join(format!("{}.{}.{}", stem, i, ext));
            let new_path = parent.join(format!("{}.{}.{}", stem, i + 1, ext));

            if old_path.exists() {
                if i + 1 >= self.max_files {
                    fs::remove_file(&old_path)?;
                } else {
                    fs::rename(&old_path, &new_path)?;
                }
            }
        }

        let rotated_path = parent.join(format!("{}.1.{}", stem, ext));
        fs::rename(path, rotated_path)?;

        Ok(())
    }

    pub fn read_log_tail(&self, path: &Path, lines: usize) -> std::io::Result<Vec<String>> {
        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let all_lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();

        let start = if all_lines.len() > lines {
            all_lines.len() - lines
        } else {
            0
        };

        Ok(all_lines[start..].to_vec())
    }

    pub fn read_log_entries(&self, path: &Path, lines: usize) -> std::io::Result<Vec<LogEntry>> {
        let raw_lines = self.read_log_tail(path, lines)?;
        let entries: Vec<LogEntry> = raw_lines
            .into_iter()
            .map(|line| self.parse_log_line(&line))
            .collect();

        Ok(entries)
    }

    pub fn parse_log_line(&self, line: &str) -> LogEntry {
        if let Some(timestamp_end) = line.find(']') {
            let timestamp = line[1..timestamp_end].to_string();
            let rest = &line[timestamp_end + 2..];

            if rest.starts_with('[') {
                if let Some(level_end) = rest.find(']') {
                    let level = rest[1..level_end].to_string();
                    let rest2 = &rest[level_end + 2..];

                    if rest2.starts_with('[') {
                        if let Some(source_end) = rest2.find(']') {
                            let source = rest2[1..source_end].to_string();
                            let message = rest2[source_end + 2..].to_string();

                            return LogEntry {
                                timestamp,
                                level,
                                source,
                                message,
                            };
                        }
                    }
                }
            }

            return LogEntry {
                timestamp,
                level: "INFO".to_string(),
                source: "system".to_string(),
                message: rest.to_string(),
            };
        }

        LogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: "INFO".to_string(),
            source: "system".to_string(),
            message: line.to_string(),
        }
    }

    pub fn cleanup_old_logs(&self) -> std::io::Result<()> {
        let cutoff = Utc::now() - chrono::Duration::days(self.retention_days as i64);

        self.cleanup_directory(&self.base_path, cutoff)?;

        Ok(())
    }

    fn cleanup_directory(&self, dir: &Path, cutoff: DateTime<Utc>) -> std::io::Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.cleanup_directory(&path, cutoff)?;
            } else if path.extension().map_or(false, |ext| ext == "log") {
                let metadata = fs::metadata(&path)?;
                if let Ok(modified) = metadata.modified() {
                    let modified: DateTime<Utc> = modified.into();
                    if modified < cutoff {
                        fs::remove_file(&path)?;
                    }
                }
            }
        }

        Ok(())
    }

    pub fn clear_log(&self, path: &Path) -> std::io::Result<()> {
        if path.exists() {
            fs::write(path, "")?;
        }
        Ok(())
    }

    pub fn get_log_size(&self, path: &Path) -> std::io::Result<u64> {
        if path.exists() {
            let metadata = fs::metadata(path)?;
            Ok(metadata.len())
        } else {
            Ok(0)
        }
    }

    /// Append a raw line to a log file (static method for use from threads)
    pub fn append_line_to_file(path: &Path, line: &str) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        writeln!(file, "[{}] {}", timestamp, line)?;

        Ok(())
    }
}

impl Default for LogManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Payload for log update events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogUpdatePayload {
    pub source: String,
    pub entries: Vec<LogEntry>,
}

/// Manages active log file watchers
pub struct LogStreamManager {
    /// Active watchers by source ID (service/project id + log type)
    active_streams: HashMap<String, mpsc::Sender<()>>,
    /// Log manager reference
    log_manager: LogManager,
}

impl LogStreamManager {
    pub fn new(log_manager: LogManager) -> Self {
        Self {
            active_streams: HashMap::new(),
            log_manager,
        }
    }

    /// Start watching a log file for changes
    pub fn start_stream<F>(
        &mut self,
        source_id: String,
        log_path: PathBuf,
        on_new_entries: F,
    ) -> bool
    where
        F: Fn(Vec<LogEntry>) + Send + Sync + 'static,
    {
        // Check if already streaming
        if self.active_streams.contains_key(&source_id) {
            return false;
        }

        let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);
        self.active_streams.insert(source_id.clone(), stop_tx);

        // Clone log manager for parsing
        let log_manager = self.log_manager.clone();

        // Spawn the file watcher task
        tokio::spawn(async move {
            let mut last_position: u64 = 0;
            let mut poll_interval = interval(Duration::from_millis(500));

            // Initialize position to end of file
            if let Ok(metadata) = fs::metadata(&log_path) {
                last_position = metadata.len();
            }

            loop {
                tokio::select! {
                    _ = poll_interval.tick() => {
                        if let Ok(new_entries) = Self::read_new_entries(
                            &log_path,
                            &mut last_position,
                            &log_manager,
                        ) {
                            if !new_entries.is_empty() {
                                on_new_entries(new_entries);
                            }
                        }
                    }
                    _ = stop_rx.recv() => {
                        break;
                    }
                }
            }
        });

        true
    }

    /// Stop watching a log file
    pub fn stop_stream(&mut self, source_id: &str) -> bool {
        if let Some(stop_tx) = self.active_streams.remove(source_id) {
            // Send stop signal (ignore if receiver is dropped)
            let _ = stop_tx.try_send(());
            true
        } else {
            false
        }
    }

    /// Check if a stream is active
    pub fn is_streaming(&self, source_id: &str) -> bool {
        self.active_streams.contains_key(source_id)
    }

    /// Stop all active streams
    pub fn stop_all(&mut self) {
        for (_, stop_tx) in self.active_streams.drain() {
            let _ = stop_tx.try_send(());
        }
    }

    /// Read new entries from the log file since last position
    fn read_new_entries(
        log_path: &Path,
        last_position: &mut u64,
        log_manager: &LogManager,
    ) -> std::io::Result<Vec<LogEntry>> {
        if !log_path.exists() {
            *last_position = 0;
            return Ok(Vec::new());
        }

        let metadata = fs::metadata(log_path)?;
        let current_size = metadata.len();

        // File was truncated or rotated
        if current_size < *last_position {
            *last_position = 0;
        }

        // No new data
        if current_size == *last_position {
            return Ok(Vec::new());
        }

        let file = File::open(log_path)?;
        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::Start(*last_position))?;

        let mut entries = Vec::new();
        let mut line = String::new();

        while reader.read_line(&mut line)? > 0 {
            let trimmed = line.trim_end();
            if !trimmed.is_empty() {
                entries.push(log_manager.parse_log_line(trimmed));
            }
            line.clear();
        }

        *last_position = current_size;
        Ok(entries)
    }

    /// Get the log path for a service
    pub fn get_service_log_path(&self, service_id: &str, log_type: &str) -> PathBuf {
        self.log_manager.get_log_path(service_id, log_type)
    }

    /// Get the log path for a project
    pub fn get_project_log_path(&self, project_name: &str, log_type: &str) -> PathBuf {
        self.log_manager.get_project_log_path(project_name, log_type)
    }
}

/// Thread-safe wrapper for LogStreamManager
pub type SharedLogStreamManager = Arc<RwLock<LogStreamManager>>;
