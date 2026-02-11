mod commands;
pub mod error;
mod models;
mod services;
mod state;
mod tray;

use services::{
    DatabaseManager, LogManager, LogStreamManager, ServiceManager, init_project_watcher,
    init_bundle_installer, init_download_manager,
};
use state::AppState;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::{Mutex, RwLock};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = Arc::new(Mutex::new(AppState::new()));
    let service_manager = Arc::new(Mutex::new(ServiceManager::new()));
    let log_manager_instance = LogManager::new();
    let log_stream_manager = Arc::new(RwLock::new(LogStreamManager::new(log_manager_instance.clone())));
    let log_manager = Arc::new(Mutex::new(log_manager_instance));
    let database_manager = Arc::new(Mutex::new(DatabaseManager::new()));
    let bundle_installer = init_bundle_installer();
    let download_manager = init_download_manager();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .manage(app_state)
        .manage(service_manager)
        .manage(log_manager)
        .manage(log_stream_manager)
        .manage(database_manager)
        .manage(bundle_installer)
        .manage(download_manager)
        .setup(|app| {
            tray::setup_tray(app)?;

            // Initialize project watcher for auto-detection
            let project_watcher = init_project_watcher(app.handle());
            app.manage(project_watcher);

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Prevent closing, minimize to tray instead
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            // Project commands
            commands::project::get_projects,
            commands::project::create_project,
            commands::project::update_project,
            commands::project::delete_project,
            commands::project::detect_project_type,
            // Process commands
            commands::process::start_project,
            commands::process::stop_project,
            commands::process::restart_project,
            commands::process::get_running_processes,
            // Port commands
            commands::port::scan_ports,
            commands::port::check_port_available,
            commands::port::suggest_available_port,
            commands::port::get_process_details,
            commands::port::kill_process_by_pid,
            // Health commands
            commands::health::check_health,
            // Service commands
            commands::service::get_services,
            commands::service::get_service,
            commands::service::start_service,
            commands::service::stop_service,
            commands::service::restart_service,
            commands::service::check_service_health,
            commands::service::get_all_service_statuses,
            commands::service::set_service_auto_start,
            commands::service::set_service_auto_restart,
            commands::service::get_service_config,
            commands::service::save_service_config,
            commands::service::get_service_config_list,
            // Log commands
            commands::log::get_service_logs,
            commands::log::get_project_logs,
            commands::log::clear_service_logs,
            commands::log::clear_project_logs,
            commands::log::cleanup_old_logs,
            commands::log::get_log_file_path,
            commands::log::write_log_entry,
            commands::log::start_log_stream,
            commands::log::stop_log_stream,
            // Env commands
            commands::env::get_env_files,
            commands::env::read_env_file,
            commands::env::write_env_file,
            commands::env::create_env_file,
            commands::env::delete_env_file,
            commands::env::copy_env_file,
            commands::env::get_env_profiles,
            commands::env::get_env_template,
            commands::env::add_env_variable,
            commands::env::update_env_variable,
            commands::env::delete_env_variable,
            // Env profile commands
            commands::env_profile::list_profiles,
            commands::env_profile::get_active_profile,
            commands::env_profile::switch_profile,
            commands::env_profile::create_profile,
            commands::env_profile::delete_profile,
            commands::env_profile::export_profile,
            commands::env_profile::import_profile,
            commands::env_profile::compare_profiles,
            commands::env_profile::merge_profiles,
            // Hosts commands
            commands::hosts::get_hosts_entries,
            commands::hosts::get_devport_hosts_entries,
            commands::hosts::add_hosts_entry,
            commands::hosts::remove_hosts_entry,
            commands::hosts::update_hosts_entry,
            commands::hosts::check_domain_available,
            commands::hosts::check_domain_conflict,
            commands::hosts::validate_domain,
            commands::hosts::suggest_domain,
            commands::hosts::cleanup_devport_hosts,
            commands::hosts::get_hosts_file_path,
            commands::hosts::get_orphan_hosts,
            commands::hosts::delete_orphan_hosts,
            // Database commands
            commands::database::set_database_credentials,
            commands::database::test_database_connection,
            commands::database::create_project_database,
            commands::database::drop_project_database,
            commands::database::list_databases,
            commands::database::dump_database,
            commands::database::restore_database,
            commands::database::get_database_backups,
            commands::database::reset_database_password,
            commands::database::test_database_credentials,
            commands::database::generate_database_password,
            // Open commands
            commands::open::open_in_vscode,
            commands::open::open_in_terminal,
            commands::open::open_in_browser,
            commands::open::open_file_explorer,
            // Config commands
            commands::config::get_apache_config,
            commands::config::save_apache_config,
            commands::config::get_mariadb_config,
            commands::config::save_mariadb_config,
            commands::config::get_php_config,
            commands::config::save_php_config,
            commands::config::restore_config_backup,
            commands::config::validate_apache_config,
            commands::config::get_apache_ports,
            // Apache VHost CRUD commands
            commands::config::create_apache_vhost,
            commands::config::update_apache_vhost,
            commands::config::delete_apache_vhost,
            commands::config::add_listen_port,
            commands::config::remove_listen_port,
            commands::config::check_document_root,
            commands::config::create_document_root,
            commands::config::get_apache_base_path,
            commands::config::get_site_title,
            // Scaffold commands
            commands::scaffold::scaffold_project,
            commands::scaffold::install_dependencies,
            commands::scaffold::check_python_available,
            commands::scaffold::get_framework_templates,
            // Tray commands
            commands::tray::update_tray_status,
            commands::tray::get_tray_status,
            // Recovery commands
            commands::recovery::init_recovery,
            commands::recovery::save_session_state,
            commands::recovery::get_auto_start_services,
            commands::recovery::get_services_to_restore,
            commands::recovery::clear_session_state,
            commands::recovery::is_pid_running,
            commands::recovery::is_port_in_use,
            commands::recovery::get_pid_on_port,
            commands::recovery::kill_process,
            commands::recovery::diagnose_mariadb,
            commands::recovery::execute_recovery_step,
            commands::recovery::check_python_installed,
            // Scheduler commands
            commands::scheduler::register_auto_start,
            commands::scheduler::unregister_auto_start,
            commands::scheduler::is_auto_start_enabled,
            commands::scheduler::get_scheduler_auto_start_services,
            commands::scheduler::set_scheduler_auto_start_services,
            // phpMyAdmin commands
            commands::phpmyadmin::check_phpmyadmin_status,
            // Credential commands
            commands::credentials::save_db_credential,
            commands::credentials::get_db_credential,
            commands::credentials::delete_db_credential,
            commands::credentials::has_db_credential,
            commands::credentials::list_db_credentials,
            // Updater commands
            commands::updater::check_for_updates,
            commands::updater::get_current_version,
            commands::updater::download_update,
            commands::updater::download_update_with_progress,
            commands::updater::get_releases_url,
            commands::updater::install_update,
            commands::updater::install_update_and_quit,
            // Bundler commands
            commands::bundler::check_bundle_status,
            commands::bundler::get_bundle_versions,
            commands::bundler::verify_bundle_integrity,
            commands::bundler::get_bundle_paths,
            commands::bundler::is_devport_installed,
            commands::bundler::get_runtime_info,
            commands::bundler::get_runtime_executable,
            // Uninstaller commands
            commands::uninstaller::get_uninstall_preview,
            commands::uninstaller::perform_uninstall,
            commands::uninstaller::stop_all_for_uninstall,
            commands::uninstaller::check_running_processes,
            commands::uninstaller::reboot_system,
            // Inventory commands
            commands::inventory::scan_inventory,
            commands::inventory::refresh_inventory_item,
            // Installer commands
            commands::installer::get_bundle_manifest,
            commands::installer::get_install_presets,
            commands::installer::get_components_by_category,
            commands::installer::get_component_info,
            commands::installer::get_installation_state,
            commands::installer::select_install_preset,
            commands::installer::toggle_component_selection,
            commands::installer::get_installed_components,
            commands::installer::is_component_installed,
            commands::installer::install_selected_components,
            commands::installer::install_component,
            commands::installer::uninstall_component,
            commands::installer::download_component_bundle,
            commands::installer::has_component_bundle,
            commands::installer::list_bundle_files,
            commands::installer::get_bundle_storage_size,
            commands::installer::delete_bundle_file,
            commands::installer::cleanup_incomplete_downloads,
            commands::installer::calculate_selection_size,
            commands::installer::get_preset_components,
            commands::installer::create_devport_directories,
            commands::installer::get_installation_summary,
            // Backup commands
            commands::backup::get_config_path,
            commands::backup::list_config_files,
            commands::backup::create_backup,
            commands::backup::restore_backup,
            commands::backup::list_backups,
            commands::backup::delete_backup,
            commands::backup::open_backup_folder,
            commands::backup::open_config_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
