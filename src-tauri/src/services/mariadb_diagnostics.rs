use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DiagnosticIssue {
    PortConflict {
        port: u16,
        pid: u32,
        process_name: String,
    },
    CorruptedData {
        details: String,
    },
    ConfigError {
        message: String,
    },
    PermissionDenied {
        path: String,
    },
    MissingExecutable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryStep {
    pub id: String,
    pub description: String,
    pub action: RecoveryAction,
    pub risk_level: String, // "safe" | "warning" | "danger"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecoveryAction {
    KillProcess { pid: u32 },
    ReinitData,
    FixConfig { key: String, value: String },
    ChangePort { new_port: u16 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticReport {
    pub issues: Vec<DiagnosticIssue>,
    pub recovery_steps: Vec<RecoveryStep>,
    pub summary: String,
}

pub struct MariaDbDiagnostics;

impl MariaDbDiagnostics {
    /// Diagnose MariaDB failure based on error message and system state
    pub fn diagnose(last_error: &str) -> DiagnosticReport {
        let mut issues = Vec::new();
        let error_lower = last_error.to_lowercase();

        // Check for port conflicts
        if error_lower.contains("port already in use")
            || error_lower.contains("address already in use")
            || error_lower.contains("bind on tcp/ip port")
        {
            if let Some((pid, name)) = Self::find_port_user(3306) {
                issues.push(DiagnosticIssue::PortConflict {
                    port: 3306,
                    pid,
                    process_name: name,
                });
            }
        }

        // Check for corrupted data
        if error_lower.contains("can't open file")
            || error_lower.contains("table is marked as crashed")
            || error_lower.contains("corrupted")
            || error_lower.contains("ibdata1")
        {
            issues.push(DiagnosticIssue::CorruptedData {
                details: "Database data files may be corrupted".to_string(),
            });
        }

        // Check for permission issues
        if error_lower.contains("access denied")
            || error_lower.contains("permission denied")
        {
            let data_dir = PathBuf::from("C:\\DevPort\\runtime\\mariadb\\data");
            issues.push(DiagnosticIssue::PermissionDenied {
                path: data_dir.display().to_string(),
            });
        }

        // Check for missing executable
        if error_lower.contains("executable not found")
            || error_lower.contains("not found")
        {
            let mysqld = Path::new("C:\\DevPort\\runtime\\mariadb\\bin\\mysqld.exe");
            if !mysqld.exists() {
                issues.push(DiagnosticIssue::MissingExecutable);
            }
        }

        // Check for config errors
        if error_lower.contains("unknown variable")
            || error_lower.contains("can't read dir")
        {
            issues.push(DiagnosticIssue::ConfigError {
                message: last_error.to_string(),
            });
        }

        // If no specific issues found, check general state
        if issues.is_empty() {
            // Check if executable exists
            let mysqld = Path::new("C:\\DevPort\\runtime\\mariadb\\bin\\mysqld.exe");
            if !mysqld.exists() {
                issues.push(DiagnosticIssue::MissingExecutable);
            }

            // Check if data dir exists
            let data_dir = Path::new("C:\\DevPort\\runtime\\mariadb\\data");
            if !data_dir.exists() {
                issues.push(DiagnosticIssue::CorruptedData {
                    details: "Data directory does not exist".to_string(),
                });
            }

            // Check port
            if let Some((pid, name)) = Self::find_port_user(3306) {
                issues.push(DiagnosticIssue::PortConflict {
                    port: 3306,
                    pid,
                    process_name: name,
                });
            }
        }

        let recovery_steps = Self::suggest_recovery(&issues);
        let summary = Self::generate_summary(&issues);

        DiagnosticReport {
            issues,
            recovery_steps,
            summary,
        }
    }

    fn suggest_recovery(issues: &[DiagnosticIssue]) -> Vec<RecoveryStep> {
        let mut steps = Vec::new();
        let mut step_id = 1;

        for issue in issues {
            match issue {
                DiagnosticIssue::PortConflict { pid, process_name, .. } => {
                    steps.push(RecoveryStep {
                        id: format!("step_{}", step_id),
                        description: format!(
                            "포트 3306을 사용 중인 프로세스 '{}' (PID: {})을 종료합니다.",
                            process_name, pid
                        ),
                        action: RecoveryAction::KillProcess { pid: *pid },
                        risk_level: "warning".to_string(),
                    });
                    step_id += 1;
                    steps.push(RecoveryStep {
                        id: format!("step_{}", step_id),
                        description: "대체 포트 3307로 변경합니다.".to_string(),
                        action: RecoveryAction::ChangePort { new_port: 3307 },
                        risk_level: "safe".to_string(),
                    });
                    step_id += 1;
                }
                DiagnosticIssue::CorruptedData { .. } => {
                    steps.push(RecoveryStep {
                        id: format!("step_{}", step_id),
                        description: "데이터 디렉토리를 재초기화합니다. (백업 후 진행)".to_string(),
                        action: RecoveryAction::ReinitData,
                        risk_level: "danger".to_string(),
                    });
                    step_id += 1;
                }
                DiagnosticIssue::ConfigError { .. } => {
                    steps.push(RecoveryStep {
                        id: format!("step_{}", step_id),
                        description: "설정 파일을 기본값으로 복원합니다.".to_string(),
                        action: RecoveryAction::FixConfig {
                            key: "reset".to_string(),
                            value: "default".to_string(),
                        },
                        risk_level: "warning".to_string(),
                    });
                    step_id += 1;
                }
                DiagnosticIssue::PermissionDenied { .. } => {
                    steps.push(RecoveryStep {
                        id: format!("step_{}", step_id),
                        description: "데이터 디렉토리 권한을 재설정합니다.".to_string(),
                        action: RecoveryAction::FixConfig {
                            key: "permissions".to_string(),
                            value: "reset".to_string(),
                        },
                        risk_level: "safe".to_string(),
                    });
                    step_id += 1;
                }
                DiagnosticIssue::MissingExecutable => {
                    steps.push(RecoveryStep {
                        id: format!("step_{}", step_id),
                        description: "MariaDB가 설치되지 않았습니다. 설치 관리자에서 MariaDB를 설치해주세요.".to_string(),
                        action: RecoveryAction::FixConfig {
                            key: "install".to_string(),
                            value: "mariadb".to_string(),
                        },
                        risk_level: "safe".to_string(),
                    });
                    step_id += 1;
                }
            }
        }

        steps
    }

    fn generate_summary(issues: &[DiagnosticIssue]) -> String {
        if issues.is_empty() {
            return "진단 결과 특별한 문제가 발견되지 않았습니다.".to_string();
        }

        let mut parts = Vec::new();
        for issue in issues {
            match issue {
                DiagnosticIssue::PortConflict { port, process_name, .. } => {
                    parts.push(format!("포트 {}이(가) '{}'에 의해 사용 중", port, process_name));
                }
                DiagnosticIssue::CorruptedData { .. } => {
                    parts.push("데이터 파일 손상 감지".to_string());
                }
                DiagnosticIssue::ConfigError { .. } => {
                    parts.push("설정 파일 오류".to_string());
                }
                DiagnosticIssue::PermissionDenied { .. } => {
                    parts.push("파일 권한 문제".to_string());
                }
                DiagnosticIssue::MissingExecutable => {
                    parts.push("MariaDB 실행 파일 없음".to_string());
                }
            }
        }

        format!("발견된 문제: {}", parts.join(", "))
    }

    /// Find which process is using a given port
    fn find_port_user(port: u16) -> Option<(u32, String)> {
        #[cfg(windows)]
        let output = Command::new("netstat")
            .args(["-ano"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .ok()?;

        #[cfg(not(windows))]
        let output = Command::new("netstat")
            .args(["-ano"])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if (line.contains(&format!(":{} ", port)) || line.contains(&format!(":{}\t", port)))
                && line.contains("LISTENING")
            {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(pid_str) = parts.last() {
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        let name = Self::get_process_name(pid).unwrap_or_else(|| "unknown".to_string());
                        return Some((pid, name));
                    }
                }
            }
        }
        None
    }

    fn get_process_name(pid: u32) -> Option<String> {
        #[cfg(windows)]
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .ok()?;

        #[cfg(not(windows))]
        let output = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "comm="])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.lines().next()?;
        let parts: Vec<&str> = line.split(',').collect();
        if parts.is_empty() {
            return None;
        }
        Some(parts[0].trim_matches('"').to_string())
    }
}

/// Reinitialize MariaDB data directory
pub fn reinitialize_data_directory(create_backup: bool) -> Result<String, String> {
    let data_dir = PathBuf::from("C:\\DevPort\\runtime\\mariadb\\data");
    let mariadb_bin = PathBuf::from("C:\\DevPort\\runtime\\mariadb\\bin");
    let install_db = mariadb_bin.join("mysql_install_db.exe");

    if !install_db.exists() {
        return Err("mysql_install_db.exe not found. MariaDB may not be installed.".to_string());
    }

    // Create backup if requested
    if create_backup && data_dir.exists() {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_dir = PathBuf::from(format!("C:\\DevPort\\backups\\mariadb_data_{}", timestamp));
        fs::create_dir_all(&backup_dir)
            .map_err(|e| format!("Failed to create backup directory: {}", e))?;

        copy_dir_recursive(&data_dir, &backup_dir)
            .map_err(|e| format!("Failed to backup data: {}", e))?;
    }

    // Remove existing data directory
    if data_dir.exists() {
        fs::remove_dir_all(&data_dir)
            .map_err(|e| format!("Failed to remove data directory: {}", e))?;
    }

    // Run mysql_install_db
    let mut cmd = Command::new(&install_db);
    cmd.args(["--datadir", &data_dir.to_string_lossy()]);

    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd.output()
        .map_err(|e| format!("Failed to run mysql_install_db: {}", e))?;

    if output.status.success() {
        Ok("Data directory reinitialized successfully".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("mysql_install_db failed: {}", stderr))
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    let entries = fs::read_dir(src).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let dest = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_recursive(&path, &dest)?;
        } else {
            fs::copy(&path, &dest).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}
