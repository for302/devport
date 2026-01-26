//! Tauri commands for credential management
//!
//! These commands provide a secure way to store and retrieve database credentials
//! using Windows DPAPI encryption.

use crate::services::credential_manager::CredentialManager;

/// Saves a database credential securely using DPAPI encryption.
///
/// # Arguments
/// * `key` - A unique identifier for the credential (e.g., "mariadb_root", "project_myapp_db")
/// * `password` - The password to encrypt and store
#[tauri::command]
pub fn save_db_credential(key: String, password: String) -> Result<(), String> {
    let manager = CredentialManager::new()?;
    manager.save_credential(&key, &password)?;
    Ok(())
}

/// Retrieves a database credential.
///
/// # Arguments
/// * `key` - The unique identifier for the credential
///
/// # Returns
/// * `Ok(Some(password))` - If the credential exists
/// * `Ok(None)` - If the credential does not exist
/// * `Err(...)` - If decryption fails
#[tauri::command]
pub fn get_db_credential(key: String) -> Result<Option<String>, String> {
    let manager = CredentialManager::new()?;
    manager.load_credential(&key)
}

/// Deletes a database credential.
///
/// # Arguments
/// * `key` - The unique identifier for the credential to delete
#[tauri::command]
pub fn delete_db_credential(key: String) -> Result<(), String> {
    let manager = CredentialManager::new()?;
    manager.delete_credential(&key)
}

/// Checks if a database credential exists.
///
/// # Arguments
/// * `key` - The unique identifier for the credential
///
/// # Returns
/// * `true` if the credential exists, `false` otherwise
#[tauri::command]
pub fn has_db_credential(key: String) -> Result<bool, String> {
    let manager = CredentialManager::new()?;
    Ok(manager.has_credential(&key))
}

/// Lists all stored credential keys.
///
/// # Returns
/// * A list of all stored credential keys
#[tauri::command]
pub fn list_db_credentials() -> Result<Vec<String>, String> {
    let manager = CredentialManager::new()?;
    manager.list_credentials()
}
