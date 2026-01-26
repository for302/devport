use crate::services::bundler::{Bundler, BundlePaths, BundleStatus, RuntimeInfo, RuntimeType};

/// Check the status of all bundled runtimes
#[tauri::command]
pub async fn check_bundle_status() -> Result<Vec<BundleStatus>, String> {
    Ok(Bundler::verify_all_integrity())
}

/// Get version information for all bundled runtimes
#[tauri::command]
pub async fn get_bundle_versions() -> Result<Vec<RuntimeInfo>, String> {
    Ok(Bundler::get_all_runtime_info().await)
}

/// Verify the integrity of all bundled runtimes
/// Returns detailed status including missing files
#[tauri::command]
pub async fn verify_bundle_integrity() -> Result<Vec<BundleStatus>, String> {
    Ok(Bundler::verify_all_integrity())
}

/// Get paths for all bundled runtimes
#[tauri::command]
pub fn get_bundle_paths() -> Result<BundlePaths, String> {
    Ok(Bundler::get_bundle_paths())
}

/// Check if DevPort base directory is installed
#[tauri::command]
pub fn is_devport_installed() -> bool {
    Bundler::is_devport_installed()
}

/// Get information for a specific runtime
#[tauri::command]
pub async fn get_runtime_info(runtime_type: RuntimeType) -> Result<RuntimeInfo, String> {
    Ok(Bundler::get_runtime_info(runtime_type).await)
}

/// Get the executable path for a specific runtime
#[tauri::command]
pub fn get_runtime_executable(runtime_type: RuntimeType) -> String {
    Bundler::get_executable_path(runtime_type)
}
