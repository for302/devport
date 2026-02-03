//! Windows DPAPI-based credential encryption manager
//!
//! This module provides secure credential storage using Windows Data Protection API (DPAPI).
//! DPAPI encrypts data using the user's Windows account credentials, making it secure
//! and tied to the user's login session.

use std::fs;
use std::path::PathBuf;

#[cfg(windows)]
use windows::Win32::Foundation::LocalFree;
#[cfg(windows)]
use windows::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN,
};

/// CredentialManager handles secure storage and retrieval of credentials using Windows DPAPI.
pub struct CredentialManager {
    credentials_dir: PathBuf,
}

impl CredentialManager {
    /// Creates a new CredentialManager instance.
    /// The credentials are stored in %APPDATA%/clickdevport/credentials/
    pub fn new() -> Result<Self, String> {
        let credentials_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("clickdevport")
            .join("credentials");

        fs::create_dir_all(&credentials_dir)
            .map_err(|e| format!("Failed to create credentials directory: {}", e))?;

        Ok(Self { credentials_dir })
    }

    /// Encrypts credential data using Windows DPAPI.
    /// The encrypted data can only be decrypted by the same Windows user.
    #[cfg(windows)]
    pub fn encrypt_credential(&self, data: &str) -> Result<Vec<u8>, String> {
        let data_bytes = data.as_bytes();

        let input_blob = CRYPT_INTEGER_BLOB {
            cbData: data_bytes.len() as u32,
            pbData: data_bytes.as_ptr() as *mut u8,
        };

        let mut output_blob = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: std::ptr::null_mut(),
        };

        unsafe {
            let result = CryptProtectData(
                &input_blob,
                None,                       // description (optional)
                None,                       // optional entropy
                None,                       // reserved
                None,                       // prompt struct
                CRYPTPROTECT_UI_FORBIDDEN,  // flags - no UI prompts
                &mut output_blob,
            );

            if result.is_err() {
                return Err(format!("DPAPI encryption failed: {:?}", result));
            }

            if output_blob.pbData.is_null() || output_blob.cbData == 0 {
                return Err("DPAPI encryption returned empty data".to_string());
            }

            // Copy the encrypted data to a Vec
            let encrypted_data =
                std::slice::from_raw_parts(output_blob.pbData, output_blob.cbData as usize)
                    .to_vec();

            // Free the memory allocated by CryptProtectData using LocalFree
            let _ = LocalFree(windows::Win32::Foundation::HLOCAL(output_blob.pbData as *mut _));

            Ok(encrypted_data)
        }
    }

    /// Decrypts credential data using Windows DPAPI.
    /// Only works for data encrypted by the same Windows user.
    #[cfg(windows)]
    pub fn decrypt_credential(&self, encrypted: &[u8]) -> Result<String, String> {
        if encrypted.is_empty() {
            return Err("Cannot decrypt empty data".to_string());
        }

        let input_blob = CRYPT_INTEGER_BLOB {
            cbData: encrypted.len() as u32,
            pbData: encrypted.as_ptr() as *mut u8,
        };

        let mut output_blob = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: std::ptr::null_mut(),
        };

        unsafe {
            let result = CryptUnprotectData(
                &input_blob,
                None,                       // description output (optional)
                None,                       // optional entropy
                None,                       // reserved
                None,                       // prompt struct
                CRYPTPROTECT_UI_FORBIDDEN,  // flags - no UI prompts
                &mut output_blob,
            );

            if result.is_err() {
                return Err(format!("DPAPI decryption failed: {:?}", result));
            }

            if output_blob.pbData.is_null() || output_blob.cbData == 0 {
                return Err("DPAPI decryption returned empty data".to_string());
            }

            // Convert decrypted bytes to string
            let decrypted_bytes =
                std::slice::from_raw_parts(output_blob.pbData, output_blob.cbData as usize);
            let decrypted_string = String::from_utf8(decrypted_bytes.to_vec())
                .map_err(|e| format!("Failed to convert decrypted data to string: {}", e))?;

            // Free the memory allocated by CryptUnprotectData using LocalFree
            let _ = LocalFree(windows::Win32::Foundation::HLOCAL(output_blob.pbData as *mut _));

            Ok(decrypted_string)
        }
    }

    /// Non-Windows stub for encrypt_credential
    #[cfg(not(windows))]
    pub fn encrypt_credential(&self, _data: &str) -> Result<Vec<u8>, String> {
        Err("DPAPI is only available on Windows".to_string())
    }

    /// Non-Windows stub for decrypt_credential
    #[cfg(not(windows))]
    pub fn decrypt_credential(&self, _encrypted: &[u8]) -> Result<String, String> {
        Err("DPAPI is only available on Windows".to_string())
    }

    /// Saves an encrypted credential to disk.
    /// The key is used as the filename (sanitized for filesystem safety).
    pub fn save_credential(&self, key: &str, value: &str) -> Result<(), String> {
        let encrypted = self.encrypt_credential(value)?;
        let file_path = self.get_credential_path(key);

        fs::write(&file_path, &encrypted)
            .map_err(|e| format!("Failed to save credential '{}': {}", key, e))?;

        Ok(())
    }

    /// Loads and decrypts a credential from disk.
    /// Returns None if the credential does not exist.
    pub fn load_credential(&self, key: &str) -> Result<Option<String>, String> {
        let file_path = self.get_credential_path(key);

        if !file_path.exists() {
            return Ok(None);
        }

        let encrypted = fs::read(&file_path)
            .map_err(|e| format!("Failed to read credential '{}': {}", key, e))?;

        let decrypted = self.decrypt_credential(&encrypted)?;
        Ok(Some(decrypted))
    }

    /// Deletes a credential from disk.
    pub fn delete_credential(&self, key: &str) -> Result<(), String> {
        let file_path = self.get_credential_path(key);

        if file_path.exists() {
            fs::remove_file(&file_path)
                .map_err(|e| format!("Failed to delete credential '{}': {}", key, e))?;
        }

        Ok(())
    }

    /// Checks if a credential exists.
    pub fn has_credential(&self, key: &str) -> bool {
        self.get_credential_path(key).exists()
    }

    /// Gets the file path for a credential key.
    /// The key is sanitized to be filesystem-safe.
    fn get_credential_path(&self, key: &str) -> PathBuf {
        // Sanitize the key for filesystem safety
        let safe_key: String = key
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();

        self.credentials_dir.join(format!("{}.cred", safe_key))
    }

    /// Lists all stored credential keys.
    pub fn list_credentials(&self) -> Result<Vec<String>, String> {
        let mut keys = Vec::new();

        let entries = fs::read_dir(&self.credentials_dir)
            .map_err(|e| format!("Failed to read credentials directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "cred") {
                if let Some(stem) = path.file_stem() {
                    keys.push(stem.to_string_lossy().to_string());
                }
            }
        }

        Ok(keys)
    }
}

impl Default for CredentialManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize credential manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(windows)]
    fn test_encrypt_decrypt_roundtrip() {
        let manager = CredentialManager::new().unwrap();
        let original = "test_password_123!@#";

        let encrypted = manager.encrypt_credential(original).unwrap();
        assert!(!encrypted.is_empty());
        assert_ne!(encrypted, original.as_bytes());

        let decrypted = manager.decrypt_credential(&encrypted).unwrap();
        assert_eq!(decrypted, original);
    }

    #[test]
    #[cfg(windows)]
    fn test_save_load_delete_credential() {
        let manager = CredentialManager::new().unwrap();
        let key = "test_credential_key";
        let value = "test_secret_value";

        // Save
        manager.save_credential(key, value).unwrap();
        assert!(manager.has_credential(key));

        // Load
        let loaded = manager.load_credential(key).unwrap();
        assert_eq!(loaded, Some(value.to_string()));

        // Delete
        manager.delete_credential(key).unwrap();
        assert!(!manager.has_credential(key));

        let loaded_after_delete = manager.load_credential(key).unwrap();
        assert_eq!(loaded_after_delete, None);
    }
}
