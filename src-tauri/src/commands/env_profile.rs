use crate::services::env_manager::{
    EnvManager, EnvProfileType, EnvVariable, ProfileComparison, ProfileInfo,
};
use std::path::PathBuf;

#[tauri::command]
pub async fn list_profiles(project_path: String) -> Result<Vec<ProfileInfo>, String> {
    let manager = EnvManager::new(PathBuf::from(project_path));
    manager.list_profiles()
}

#[tauri::command]
pub async fn get_active_profile(project_path: String) -> Result<String, String> {
    let manager = EnvManager::new(PathBuf::from(project_path));
    manager.get_active_profile()
}

#[tauri::command]
pub async fn switch_profile(project_path: String, profile_file_name: String) -> Result<(), String> {
    let manager = EnvManager::new(PathBuf::from(project_path));
    manager.switch_profile(&profile_file_name)
}

#[tauri::command]
pub async fn create_profile(
    project_path: String,
    profile_type: String,
    custom_name: Option<String>,
    copy_from: Option<String>,
) -> Result<String, String> {
    let manager = EnvManager::new(PathBuf::from(project_path));

    let env_profile_type = match profile_type.to_lowercase().as_str() {
        "development" | "dev" => EnvProfileType::Development,
        "staging" | "stage" => EnvProfileType::Staging,
        "production" | "prod" => EnvProfileType::Production,
        "custom" => {
            let name = custom_name.ok_or("Custom profile requires a name")?;
            EnvProfileType::Custom(name)
        }
        _ => EnvProfileType::Custom(profile_type),
    };

    manager.create_profile(env_profile_type, copy_from.as_deref())
}

#[tauri::command]
pub async fn delete_profile(project_path: String, profile_file_name: String) -> Result<(), String> {
    let manager = EnvManager::new(PathBuf::from(project_path));
    manager.delete_profile(&profile_file_name)
}

#[tauri::command]
pub async fn export_profile(
    project_path: String,
    profile_file_name: String,
    export_path: String,
) -> Result<(), String> {
    let manager = EnvManager::new(PathBuf::from(project_path));
    manager.export_profile(&profile_file_name, &export_path)
}

#[tauri::command]
pub async fn import_profile(
    project_path: String,
    profile_type: String,
    custom_name: Option<String>,
    import_path: String,
) -> Result<String, String> {
    let manager = EnvManager::new(PathBuf::from(project_path));

    let env_profile_type = match profile_type.to_lowercase().as_str() {
        "development" | "dev" => EnvProfileType::Development,
        "staging" | "stage" => EnvProfileType::Staging,
        "production" | "prod" => EnvProfileType::Production,
        "custom" => {
            let name = custom_name.ok_or("Custom profile requires a name")?;
            EnvProfileType::Custom(name)
        }
        _ => EnvProfileType::Custom(profile_type),
    };

    manager.import_profile(env_profile_type, &import_path)
}

#[tauri::command]
pub async fn compare_profiles(
    project_path: String,
    profile_a: String,
    profile_b: String,
) -> Result<ProfileComparison, String> {
    let manager = EnvManager::new(PathBuf::from(project_path));
    manager.compare_profiles(&profile_a, &profile_b)
}

#[tauri::command]
pub async fn merge_profiles(
    project_path: String,
    source: String,
    target: String,
    overwrite: bool,
) -> Result<Vec<EnvVariable>, String> {
    let manager = EnvManager::new(PathBuf::from(project_path));
    manager.merge_profiles(&source, &target, overwrite)
}
