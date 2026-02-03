use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use chrono::Local;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseCredentials {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseInfo {
    pub name: String,
    pub size: Option<u64>,
    pub tables_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    pub file_name: String,
    pub file_path: String,
    pub size: u64,
    pub created_at: String,
    pub database: String,
}

pub struct DatabaseManager {
    mysql_path: PathBuf,
    mysqldump_path: PathBuf,
    backup_dir: PathBuf,
    root_credentials: Option<DatabaseCredentials>,
}

impl DatabaseManager {
    pub fn new() -> Self {
        let base_path = PathBuf::from("C:\\DevPort\\runtime\\mariadb\\bin");

        Self {
            mysql_path: base_path.join("mysql.exe"),
            mysqldump_path: base_path.join("mysqldump.exe"),
            backup_dir: PathBuf::from("C:\\DevPort\\backups"),
            root_credentials: None,
        }
    }

    pub fn set_root_credentials(&mut self, username: String, password: String) {
        self.root_credentials = Some(DatabaseCredentials {
            host: "localhost".to_string(),
            port: 3306,
            username,
            password,
            database: "mysql".to_string(),
        });
    }

    fn get_root_creds(&self) -> Result<&DatabaseCredentials, String> {
        self.root_credentials
            .as_ref()
            .ok_or_else(|| "Root credentials not set".to_string())
    }

    pub fn execute_sql(&self, sql: &str) -> Result<String, String> {
        let creds = self.get_root_creds()?;

        let mut cmd = Command::new(&self.mysql_path);
        cmd.args([
            "-h", &creds.host,
            "-P", &creds.port.to_string(),
            "-u", &creds.username,
            &format!("-p{}", creds.password),
            "-e", sql,
        ]);

        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let output = cmd.output().map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    pub fn create_database(&self, db_name: &str) -> Result<(), String> {
        let sql = format!("CREATE DATABASE IF NOT EXISTS `{}`", db_name);
        self.execute_sql(&sql)?;
        Ok(())
    }

    pub fn create_user(&self, username: &str, password: &str, db_name: &str) -> Result<(), String> {
        let create_user = format!(
            "CREATE USER IF NOT EXISTS '{}'@'localhost' IDENTIFIED BY '{}'",
            username, password
        );
        self.execute_sql(&create_user)?;

        let grant = format!(
            "GRANT ALL PRIVILEGES ON `{}`.* TO '{}'@'localhost'",
            db_name, username
        );
        self.execute_sql(&grant)?;

        self.execute_sql("FLUSH PRIVILEGES")?;

        Ok(())
    }

    pub fn create_database_with_user(
        &self,
        db_name: &str,
        username: &str,
        password: &str,
    ) -> Result<DatabaseCredentials, String> {
        self.create_database(db_name)?;
        self.create_user(username, password, db_name)?;

        Ok(DatabaseCredentials {
            host: "localhost".to_string(),
            port: 3306,
            username: username.to_string(),
            password: password.to_string(),
            database: db_name.to_string(),
        })
    }

    pub fn drop_database(&self, db_name: &str) -> Result<(), String> {
        let sql = format!("DROP DATABASE IF EXISTS `{}`", db_name);
        self.execute_sql(&sql)?;
        Ok(())
    }

    pub fn drop_user(&self, username: &str) -> Result<(), String> {
        let sql = format!("DROP USER IF EXISTS '{}'@'localhost'", username);
        self.execute_sql(&sql)?;
        Ok(())
    }

    pub fn reset_password(&self, username: &str, new_password: &str) -> Result<(), String> {
        let sql = format!(
            "ALTER USER '{}'@'localhost' IDENTIFIED BY '{}'",
            username, new_password
        );
        self.execute_sql(&sql)?;
        self.execute_sql("FLUSH PRIVILEGES")?;
        Ok(())
    }

    pub fn list_databases(&self) -> Result<Vec<String>, String> {
        let output = self.execute_sql("SHOW DATABASES")?;
        let databases: Vec<String> = output
            .lines()
            .skip(1)
            .filter(|line| {
                !["information_schema", "performance_schema", "mysql", "sys"]
                    .contains(&line.trim())
            })
            .map(|s| s.trim().to_string())
            .collect();

        Ok(databases)
    }

    pub fn dump_database(&self, db_name: &str, project_name: &str) -> Result<BackupInfo, String> {
        let creds = self.get_root_creds()?;

        let project_backup_dir = self.backup_dir.join(project_name);
        fs::create_dir_all(&project_backup_dir).map_err(|e| e.to_string())?;

        let timestamp = Local::now().format("%Y-%m-%d_%H%M%S").to_string();
        let file_name = format!("{}_{}.sql", timestamp, db_name);
        let file_path = project_backup_dir.join(&file_name);

        let mut cmd = Command::new(&self.mysqldump_path);
        cmd.args([
            "-h", &creds.host,
            "-P", &creds.port.to_string(),
            "-u", &creds.username,
            &format!("-p{}", creds.password),
            "--routines",
            "--triggers",
            "--single-transaction",
            db_name,
        ])
        .stdout(Stdio::piped());

        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let output = cmd.output().map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        fs::write(&file_path, &output.stdout).map_err(|e| e.to_string())?;

        let latest_path = project_backup_dir.join("latest.txt");
        fs::write(&latest_path, &file_name).map_err(|e| e.to_string())?;

        let size = output.stdout.len() as u64;

        Ok(BackupInfo {
            file_name,
            file_path: file_path.to_string_lossy().to_string(),
            size,
            created_at: Local::now().to_rfc3339(),
            database: db_name.to_string(),
        })
    }

    pub fn restore_database(&self, db_name: &str, backup_path: &str) -> Result<(), String> {
        let creds = self.get_root_creds()?;

        self.create_database(db_name)?;

        let sql_content = fs::read_to_string(backup_path).map_err(|e| e.to_string())?;

        let mut cmd = Command::new(&self.mysql_path);
        cmd.args([
            "-h", &creds.host,
            "-P", &creds.port.to_string(),
            "-u", &creds.username,
            &format!("-p{}", creds.password),
            db_name,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let mut child = cmd.spawn().map_err(|e| e.to_string())?;

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(sql_content.as_bytes()).map_err(|e| e.to_string())?;
        }

        let output = child.wait_with_output().map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        Ok(())
    }

    pub fn get_backups(&self, project_name: &str) -> Result<Vec<BackupInfo>, String> {
        let project_backup_dir = self.backup_dir.join(project_name);

        if !project_backup_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();

        for entry in fs::read_dir(&project_backup_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "sql") {
                let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();

                let parts: Vec<&str> = file_name.split('_').collect();
                let database = if parts.len() >= 3 {
                    parts[2].trim_end_matches(".sql").to_string()
                } else {
                    "unknown".to_string()
                };

                backups.push(BackupInfo {
                    file_name,
                    file_path: path.to_string_lossy().to_string(),
                    size: metadata.len(),
                    created_at: metadata
                        .created()
                        .map(|t| {
                            let datetime: chrono::DateTime<chrono::Utc> = t.into();
                            datetime.to_rfc3339()
                        })
                        .unwrap_or_default(),
                    database,
                });
            }
        }

        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(backups)
    }

    pub fn test_connection(&self) -> Result<bool, String> {
        match self.execute_sql("SELECT 1") {
            Ok(_) => Ok(true),
            Err(e) => Err(e),
        }
    }

    pub fn test_credentials(&self, creds: &DatabaseCredentials) -> Result<bool, String> {
        let mut cmd = Command::new(&self.mysql_path);
        cmd.args([
            "-h", &creds.host,
            "-P", &creds.port.to_string(),
            "-u", &creds.username,
            &format!("-p{}", creds.password),
            "-e", "SELECT 1",
        ]);

        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let output = cmd.output().map_err(|e| e.to_string())?;

        Ok(output.status.success())
    }

    pub fn generate_password(length: usize) -> String {
        use std::collections::HashSet;

        let charset: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*";
        let mut password = String::with_capacity(length);

        for _ in 0..length {
            let idx = (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                % charset.len() as u128) as usize;
            password.push(charset[idx] as char);
            std::thread::sleep(std::time::Duration::from_nanos(1));
        }

        password
    }
}

impl Default for DatabaseManager {
    fn default() -> Self {
        Self::new()
    }
}
