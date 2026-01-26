use crate::services::database_manager::{BackupInfo, DatabaseCredentials, DatabaseManager};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

#[tauri::command]
pub async fn set_database_credentials(
    db_manager: State<'_, Arc<Mutex<DatabaseManager>>>,
    username: String,
    password: String,
) -> Result<(), String> {
    let mut manager = db_manager.lock().await;
    manager.set_root_credentials(username, password);
    Ok(())
}

#[tauri::command]
pub async fn test_database_connection(
    db_manager: State<'_, Arc<Mutex<DatabaseManager>>>,
) -> Result<bool, String> {
    let manager = db_manager.lock().await;
    manager.test_connection()
}

#[tauri::command]
pub async fn create_project_database(
    db_manager: State<'_, Arc<Mutex<DatabaseManager>>>,
    db_name: String,
    username: String,
    password: Option<String>,
) -> Result<DatabaseCredentials, String> {
    let manager = db_manager.lock().await;
    let password = password.unwrap_or_else(|| DatabaseManager::generate_password(16));
    manager.create_database_with_user(&db_name, &username, &password)
}

#[tauri::command]
pub async fn drop_project_database(
    db_manager: State<'_, Arc<Mutex<DatabaseManager>>>,
    db_name: String,
    username: String,
) -> Result<(), String> {
    let manager = db_manager.lock().await;
    manager.drop_database(&db_name)?;
    manager.drop_user(&username)?;
    Ok(())
}

#[tauri::command]
pub async fn list_databases(
    db_manager: State<'_, Arc<Mutex<DatabaseManager>>>,
) -> Result<Vec<String>, String> {
    let manager = db_manager.lock().await;
    manager.list_databases()
}

#[tauri::command]
pub async fn dump_database(
    db_manager: State<'_, Arc<Mutex<DatabaseManager>>>,
    db_name: String,
    project_name: String,
) -> Result<BackupInfo, String> {
    let manager = db_manager.lock().await;
    manager.dump_database(&db_name, &project_name)
}

#[tauri::command]
pub async fn restore_database(
    db_manager: State<'_, Arc<Mutex<DatabaseManager>>>,
    db_name: String,
    backup_path: String,
) -> Result<(), String> {
    let manager = db_manager.lock().await;
    manager.restore_database(&db_name, &backup_path)
}

#[tauri::command]
pub async fn get_database_backups(
    db_manager: State<'_, Arc<Mutex<DatabaseManager>>>,
    project_name: String,
) -> Result<Vec<BackupInfo>, String> {
    let manager = db_manager.lock().await;
    manager.get_backups(&project_name)
}

#[tauri::command]
pub async fn reset_database_password(
    db_manager: State<'_, Arc<Mutex<DatabaseManager>>>,
    username: String,
    new_password: Option<String>,
) -> Result<String, String> {
    let manager = db_manager.lock().await;
    let password = new_password.unwrap_or_else(|| DatabaseManager::generate_password(16));
    manager.reset_password(&username, &password)?;
    Ok(password)
}

#[tauri::command]
pub async fn test_database_credentials(
    db_manager: State<'_, Arc<Mutex<DatabaseManager>>>,
    host: String,
    port: u16,
    username: String,
    password: String,
    database: String,
) -> Result<bool, String> {
    let manager = db_manager.lock().await;
    let creds = DatabaseCredentials {
        host,
        port,
        username,
        password,
        database,
    };
    manager.test_credentials(&creds)
}

#[tauri::command]
pub async fn generate_database_password() -> Result<String, String> {
    Ok(DatabaseManager::generate_password(16))
}
