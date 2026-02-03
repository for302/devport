use crate::models::{
    BundleComponent, BundleManifest, ComponentCategory, InstallOptions,
    InstallationState, InstalledComponent,
};
use crate::services::{
    bundle_installer::BundleInstaller, inventory_scanner::InventoryScanner,
    SharedBundleInstaller, SharedDownloadManager,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

/// Component info for frontend display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentInfo {
    pub id: String,
    pub name: String,
    pub category: String,
    pub version: String,
    pub size_bytes: u64,
    pub size_mb: f64,
    pub description: String,
    pub icon: Option<String>,
    pub is_installed: bool,
    pub installed_version: Option<String>,
    pub installed_source: Option<String>,
    pub has_bundle: bool,
    pub dependencies: Vec<String>,
}

impl ComponentInfo {
    pub fn from_component(
        component: &BundleComponent,
        is_installed: bool,
        installed_version: Option<String>,
        installed_source: Option<String>,
        has_bundle: bool,
    ) -> Self {
        Self {
            id: component.id.clone(),
            name: component.name.clone(),
            category: format!("{:?}", component.category),
            version: component.version.clone(),
            size_bytes: component.size_bytes,
            size_mb: component.size_mb(),
            description: component.description.clone(),
            icon: component.icon.clone(),
            is_installed,
            installed_version,
            installed_source,
            has_bundle,
            dependencies: component.dependencies.clone(),
        }
    }
}

/// Preset info for frontend display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub components: Vec<String>,
    pub optional_components: Vec<String>,
    pub total_size_bytes: u64,
    pub total_size_mb: f64,
}

/// Category group for organized display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryGroup {
    pub category: String,
    pub display_name: String,
    pub components: Vec<ComponentInfo>,
}

// ============================================================================
// Manifest Commands
// ============================================================================

/// Get the bundle manifest with all available components and presets
#[tauri::command]
pub async fn get_bundle_manifest(
    installer: State<'_, SharedBundleInstaller>,
) -> Result<BundleManifest, String> {
    let installer = installer.lock().await;
    Ok(installer.get_manifest().clone())
}

/// Get all available presets
#[tauri::command]
pub async fn get_install_presets(
    installer: State<'_, SharedBundleInstaller>,
) -> Result<Vec<PresetInfo>, String> {
    let installer = installer.lock().await;
    let manifest = installer.get_manifest();

    let mut presets = Vec::new();
    for (id, preset) in &manifest.presets {
        let total_size = manifest.calculate_total_size(&preset.components);
        presets.push(PresetInfo {
            id: id.clone(),
            name: preset.name.clone(),
            description: preset.description.clone(),
            icon: preset.icon.clone(),
            components: preset.components.clone(),
            optional_components: preset.optional_components.clone(),
            total_size_bytes: total_size,
            total_size_mb: total_size as f64 / 1_048_576.0,
        });
    }

    Ok(presets)
}

/// Get all components grouped by category
#[tauri::command]
pub async fn get_components_by_category(
    installer: State<'_, SharedBundleInstaller>,
    download_manager: State<'_, SharedDownloadManager>,
) -> Result<Vec<CategoryGroup>, String> {
    // Run inventory scan to detect system-installed software
    let inventory = InventoryScanner::scan_all().await;
    let all_inventory_items: Vec<_> = inventory.runtimes.iter()
        .chain(inventory.web_servers.iter())
        .chain(inventory.databases.iter())
        .chain(inventory.build_tools.iter())
        .chain(inventory.frameworks.iter())
        .chain(inventory.package_managers.iter())
        .chain(inventory.dev_tools.iter())
        .collect();

    let installer = installer.lock().await;
    let download_manager = download_manager.lock().await;
    let manifest = installer.get_manifest();

    let categories = vec![
        (ComponentCategory::WebServer, "Web Servers"),
        (ComponentCategory::Database, "Databases"),
        (ComponentCategory::Runtime, "Runtimes"),
        (ComponentCategory::PackageManager, "Package Managers"),
        (ComponentCategory::DevTool, "Dev Tools"),
        (ComponentCategory::BuildTool, "Build Tools"),
    ];

    let mut groups = Vec::new();

    for (category, display_name) in categories {
        let components: Vec<ComponentInfo> = manifest
            .get_components_by_category(&category)
            .iter()
            .map(|c| {
                let devport_installed = installer.is_component_installed(&c.id);
                let devport_version = installer
                    .get_component_status(&c.id)
                    .map(|i| i.version.clone());
                let has_bundle = download_manager.bundle_exists(c);

                // Check inventory scan results for system-level installation
                let inventory_item = all_inventory_items.iter().find(|item| item.id == c.id);
                let system_installed = inventory_item
                    .map(|item| item.is_installed)
                    .unwrap_or(false);

                let is_installed = devport_installed || system_installed;
                let installed_version = if devport_installed {
                    devport_version
                } else if system_installed {
                    inventory_item.and_then(|item| item.version.clone())
                } else {
                    None
                };
                let installed_source = if devport_installed {
                    Some("DevPort".to_string())
                } else if system_installed {
                    inventory_item.map(|item| format!("{:?}", item.install_source))
                } else {
                    None
                };

                ComponentInfo::from_component(c, is_installed, installed_version, installed_source, has_bundle)
            })
            .collect();

        if !components.is_empty() {
            groups.push(CategoryGroup {
                category: format!("{:?}", category),
                display_name: display_name.to_string(),
                components,
            });
        }
    }

    Ok(groups)
}

/// Get a single component's info
#[tauri::command]
pub async fn get_component_info(
    installer: State<'_, SharedBundleInstaller>,
    download_manager: State<'_, SharedDownloadManager>,
    component_id: String,
) -> Result<ComponentInfo, String> {
    let installer = installer.lock().await;
    let download_manager = download_manager.lock().await;
    let manifest = installer.get_manifest();

    let component = manifest
        .get_component(&component_id)
        .ok_or_else(|| format!("Component '{}' not found", component_id))?;

    let devport_installed = installer.is_component_installed(&component_id);
    let devport_version = installer
        .get_component_status(&component_id)
        .map(|i| i.version.clone());
    let has_bundle = download_manager.bundle_exists(component);

    // Check system inventory for this specific component
    let inventory_item = InventoryScanner::refresh_item(&component_id).await;
    let system_installed = inventory_item
        .as_ref()
        .map(|item| item.is_installed)
        .unwrap_or(false);

    let is_installed = devport_installed || system_installed;
    let installed_version = if devport_installed {
        devport_version
    } else if system_installed {
        inventory_item.as_ref().and_then(|item| item.version.clone())
    } else {
        None
    };
    let installed_source = if devport_installed {
        Some("DevPort".to_string())
    } else if system_installed {
        inventory_item.map(|item| format!("{:?}", item.install_source))
    } else {
        None
    };

    Ok(ComponentInfo::from_component(
        component,
        is_installed,
        installed_version,
        installed_source,
        has_bundle,
    ))
}

// ============================================================================
// Installation State Commands
// ============================================================================

/// Get current installation state
#[tauri::command]
pub async fn get_installation_state(
    installer: State<'_, SharedBundleInstaller>,
) -> Result<InstallationState, String> {
    let installer = installer.lock().await;
    Ok(installer.get_installation_state().clone())
}

/// Select a preset for installation
#[tauri::command]
pub async fn select_install_preset(
    installer: State<'_, SharedBundleInstaller>,
    preset_id: String,
) -> Result<Vec<String>, String> {
    let mut installer = installer.lock().await;
    installer.select_preset(&preset_id)
}

/// Toggle a component in the selection
#[tauri::command]
pub async fn toggle_component_selection(
    installer: State<'_, SharedBundleInstaller>,
    component_id: String,
) -> Result<bool, String> {
    let mut installer = installer.lock().await;
    Ok(installer.toggle_component(&component_id))
}

/// Get the list of installed components
#[tauri::command]
pub async fn get_installed_components(
    installer: State<'_, SharedBundleInstaller>,
) -> Result<Vec<InstalledComponent>, String> {
    let installer = installer.lock().await;
    Ok(installer.get_installed_components().clone())
}

/// Check if a component is installed
#[tauri::command]
pub async fn is_component_installed(
    installer: State<'_, SharedBundleInstaller>,
    component_id: String,
) -> Result<bool, String> {
    let installer = installer.lock().await;
    Ok(installer.is_component_installed(&component_id))
}

// ============================================================================
// Installation Commands
// ============================================================================

/// Install selected components (downloads bundles if needed)
#[tauri::command]
pub async fn install_selected_components(
    installer: State<'_, SharedBundleInstaller>,
    download_manager: State<'_, SharedDownloadManager>,
    app_handle: AppHandle,
) -> Result<Vec<InstalledComponent>, String> {
    use std::path::PathBuf;

    // Get selected components
    let components = {
        let installer_guard = installer.lock().await;
        let components = installer_guard.installation_state.selected_components.clone();
        if components.is_empty() {
            return Err("No components selected for installation".to_string());
        }
        components
    };

    let mut installed = Vec::new();
    let total = components.len();

    // Update installation state
    {
        let mut installer_guard = installer.lock().await;
        installer_guard.installation_state.is_installing = true;
        installer_guard.installation_state.total_count = total as u32;
        installer_guard.installation_state.completed_count = 0;
    }

    for (i, component_id) in components.iter().enumerate() {
        // Update progress
        {
            let mut installer_guard = installer.lock().await;
            installer_guard.installation_state.current_component = Some(component_id.clone());
            installer_guard.installation_state.overall_progress =
                ((i as f32 / total as f32) * 100.0) as u8;
        }

        // Get component info and check if we need to download
        let bundle_path = {
            let installer_guard = installer.lock().await;
            let component = installer_guard
                .get_manifest()
                .get_component(component_id)
                .ok_or_else(|| format!("Component '{}' not found", component_id))?
                .clone();

            if let Some(file_name) = &component.file_name {
                let bundle_file = PathBuf::from("C:\\DevPort\\bundles").join(file_name);

                if !bundle_file.exists() {
                    // Need to download - release installer lock first
                    drop(installer_guard);

                    // Download the bundle
                    let mut dm = download_manager.lock().await;
                    dm.download_component(&component, Some(&app_handle)).await?
                } else {
                    bundle_file
                }
            } else {
                // No file_name, install without bundle (e.g., npm packages)
                PathBuf::new()
            }
        };

        // Install the component
        let bundle_opt = if bundle_path.as_os_str().is_empty() {
            None
        } else {
            Some(bundle_path.as_path())
        };

        let mut installer_guard = installer.lock().await;
        match installer_guard
            .install_component(component_id, bundle_opt, Some(&app_handle))
            .await
        {
            Ok(component) => {
                installed.push(component);
                installer_guard.installation_state.completed_count += 1;
            }
            Err(e) => {
                installer_guard.installation_state.is_installing = false;
                installer_guard.installation_state.error = Some(e.clone());
                return Err(e);
            }
        }
    }

    // Mark installation as complete
    {
        let mut installer_guard = installer.lock().await;
        installer_guard.installation_state.is_installing = false;
        installer_guard.installation_state.overall_progress = 100;
    }

    Ok(installed)
}

/// Install a single component (downloads bundle if needed)
#[tauri::command]
pub async fn install_component(
    installer: State<'_, SharedBundleInstaller>,
    download_manager: State<'_, SharedDownloadManager>,
    app_handle: AppHandle,
    component_id: String,
    options: Option<InstallOptions>,
) -> Result<InstalledComponent, String> {
    use std::path::PathBuf;

    // First, check if bundle exists. If not, download it.
    let bundle_path = {
        let installer_guard = installer.lock().await;
        let component = installer_guard
            .get_manifest()
            .get_component(&component_id)
            .ok_or_else(|| format!("Component '{}' not found", component_id))?
            .clone();

        if let Some(file_name) = &component.file_name {
            let bundle_file = PathBuf::from("C:\\DevPort\\bundles").join(file_name);

            if !bundle_file.exists() {
                // Need to download - release installer lock first
                drop(installer_guard);

                // Download the bundle
                let mut dm = download_manager.lock().await;
                dm.download_component(&component, Some(&app_handle)).await?
            } else {
                bundle_file
            }
        } else {
            // No file_name, install without bundle (e.g., npm packages)
            PathBuf::new()
        }
    };

    // Now install with the bundle
    let mut installer_guard = installer.lock().await;
    let bundle_opt = if bundle_path.as_os_str().is_empty() {
        None
    } else {
        Some(bundle_path.as_path())
    };

    installer_guard
        .install_component(&component_id, bundle_opt, Some(&app_handle))
        .await
}

/// Uninstall a component
#[tauri::command]
pub async fn uninstall_component(
    installer: State<'_, SharedBundleInstaller>,
    component_id: String,
) -> Result<(), String> {
    let mut installer = installer.lock().await;
    installer.uninstall_component(&component_id).await
}

// ============================================================================
// Download Commands
// ============================================================================

/// Download a component's bundle file
#[tauri::command]
pub async fn download_component_bundle(
    installer: State<'_, SharedBundleInstaller>,
    download_manager: State<'_, SharedDownloadManager>,
    app_handle: AppHandle,
    component_id: String,
) -> Result<String, String> {
    let installer = installer.lock().await;
    let mut download_manager = download_manager.lock().await;

    let component = installer
        .get_manifest()
        .get_component(&component_id)
        .ok_or_else(|| format!("Component '{}' not found", component_id))?
        .clone();

    let path = download_manager
        .download_component(&component, Some(&app_handle))
        .await?;

    Ok(path.to_string_lossy().to_string())
}

/// Check if a bundle file exists for a component
#[tauri::command]
pub async fn has_component_bundle(
    installer: State<'_, SharedBundleInstaller>,
    download_manager: State<'_, SharedDownloadManager>,
    component_id: String,
) -> Result<bool, String> {
    let installer = installer.lock().await;
    let download_manager = download_manager.lock().await;

    let component = installer
        .get_manifest()
        .get_component(&component_id)
        .ok_or_else(|| format!("Component '{}' not found", component_id))?;

    Ok(download_manager.bundle_exists(component))
}

/// List all available bundle files
#[tauri::command]
pub async fn list_bundle_files(
    download_manager: State<'_, SharedDownloadManager>,
) -> Result<Vec<String>, String> {
    let download_manager = download_manager.lock().await;
    Ok(download_manager.list_bundle_files())
}

/// Get total bundle storage size
#[tauri::command]
pub async fn get_bundle_storage_size(
    download_manager: State<'_, SharedDownloadManager>,
) -> Result<u64, String> {
    let download_manager = download_manager.lock().await;
    Ok(download_manager.get_total_bundle_size())
}

/// Delete a bundle file
#[tauri::command]
pub async fn delete_bundle_file(
    download_manager: State<'_, SharedDownloadManager>,
    file_name: String,
) -> Result<(), String> {
    let download_manager = download_manager.lock().await;
    download_manager.delete_bundle(&file_name)
}

/// Clean up incomplete downloads
#[tauri::command]
pub async fn cleanup_incomplete_downloads(
    download_manager: State<'_, SharedDownloadManager>,
) -> Result<u64, String> {
    let download_manager = download_manager.lock().await;
    download_manager.cleanup_incomplete_downloads()
}

// ============================================================================
// Utility Commands
// ============================================================================

/// Calculate total size for selected components
#[tauri::command]
pub async fn calculate_selection_size(
    installer: State<'_, SharedBundleInstaller>,
    component_ids: Vec<String>,
) -> Result<u64, String> {
    let installer = installer.lock().await;
    Ok(installer.get_manifest().calculate_total_size(&component_ids))
}

/// Get components required by a preset (including dependencies)
#[tauri::command]
pub async fn get_preset_components(
    installer: State<'_, SharedBundleInstaller>,
    preset_id: String,
) -> Result<Vec<String>, String> {
    let installer = installer.lock().await;
    Ok(installer.get_manifest().get_preset_components(&preset_id))
}

/// Create base directories for DevPort installation
#[tauri::command]
pub async fn create_devport_directories() -> Result<(), String> {
    BundleInstaller::create_base_directories()
}

/// Get installation summary (counts and sizes)
#[tauri::command]
pub async fn get_installation_summary(
    installer: State<'_, SharedBundleInstaller>,
) -> Result<InstallationSummary, String> {
    let installer = installer.lock().await;
    let manifest = installer.get_manifest();
    let installed = installer.get_installed_components();

    let total_components = manifest.components.len();
    let installed_count = installed.len();
    let installed_size: u64 = installed.iter().map(|c| c.size_bytes).sum();

    Ok(InstallationSummary {
        total_components,
        installed_count,
        installed_size_bytes: installed_size,
        installed_size_mb: installed_size as f64 / 1_048_576.0,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallationSummary {
    pub total_components: usize,
    pub installed_count: usize,
    pub installed_size_bytes: u64,
    pub installed_size_mb: f64,
}
