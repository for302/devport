use crate::services::env_manager::{EnvManager, EnvProfile, EnvVariable};
use std::path::PathBuf;

#[tauri::command]
pub async fn get_env_files(project_path: String) -> Result<Vec<String>, String> {
    let manager = EnvManager::new(PathBuf::from(project_path));
    Ok(manager.get_env_files())
}

#[tauri::command]
pub async fn read_env_file(
    project_path: String,
    file_name: String,
) -> Result<Vec<EnvVariable>, String> {
    let manager = EnvManager::new(PathBuf::from(project_path));
    manager.read_env_file(&file_name)
}

#[tauri::command]
pub async fn write_env_file(
    project_path: String,
    file_name: String,
    variables: Vec<EnvVariable>,
) -> Result<(), String> {
    let manager = EnvManager::new(PathBuf::from(project_path));
    manager.write_env_file(&file_name, &variables)
}

#[tauri::command]
pub async fn create_env_file(project_path: String, file_name: String) -> Result<(), String> {
    let manager = EnvManager::new(PathBuf::from(project_path));
    manager.create_env_file(&file_name)
}

#[tauri::command]
pub async fn delete_env_file(project_path: String, file_name: String) -> Result<(), String> {
    let manager = EnvManager::new(PathBuf::from(project_path));
    manager.delete_env_file(&file_name)
}

#[tauri::command]
pub async fn copy_env_file(
    project_path: String,
    source: String,
    destination: String,
) -> Result<(), String> {
    let manager = EnvManager::new(PathBuf::from(project_path));
    manager.copy_env_file(&source, &destination)
}

#[tauri::command]
pub async fn get_env_profiles(project_path: String) -> Result<Vec<EnvProfile>, String> {
    let manager = EnvManager::new(PathBuf::from(project_path));
    Ok(manager.get_profiles())
}

#[tauri::command]
pub async fn get_env_template(framework: String) -> Result<Vec<EnvVariable>, String> {
    Ok(EnvManager::get_template(&framework))
}

#[tauri::command]
pub async fn add_env_variable(
    project_path: String,
    file_name: String,
    variable: EnvVariable,
) -> Result<Vec<EnvVariable>, String> {
    let manager = EnvManager::new(PathBuf::from(project_path));
    let mut variables = manager.read_env_file(&file_name)?;

    if variables.iter().any(|v| v.key == variable.key) {
        return Err(format!("Variable {} already exists", variable.key));
    }

    variables.push(variable);
    manager.write_env_file(&file_name, &variables)?;

    Ok(variables)
}

#[tauri::command]
pub async fn update_env_variable(
    project_path: String,
    file_name: String,
    key: String,
    value: String,
) -> Result<Vec<EnvVariable>, String> {
    let manager = EnvManager::new(PathBuf::from(project_path));
    let mut variables = manager.read_env_file(&file_name)?;

    if let Some(var) = variables.iter_mut().find(|v| v.key == key) {
        var.value = value;
    } else {
        return Err(format!("Variable {} not found", key));
    }

    manager.write_env_file(&file_name, &variables)?;

    Ok(variables)
}

#[tauri::command]
pub async fn delete_env_variable(
    project_path: String,
    file_name: String,
    key: String,
) -> Result<Vec<EnvVariable>, String> {
    let manager = EnvManager::new(PathBuf::from(project_path));
    let mut variables = manager.read_env_file(&file_name)?;

    let initial_len = variables.len();
    variables.retain(|v| v.key != key);

    if variables.len() == initial_len {
        return Err(format!("Variable {} not found", key));
    }

    manager.write_env_file(&file_name, &variables)?;

    Ok(variables)
}
