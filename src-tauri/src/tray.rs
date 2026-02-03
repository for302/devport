use crate::services::ServiceManager;
use std::sync::Arc;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};
use tokio::sync::Mutex;

const TRAY_ID: &str = "main-tray";

pub fn setup_tray<R: Runtime>(app: &tauri::App<R>) -> Result<(), Box<dyn std::error::Error>> {
    let show_item = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let hide_item = MenuItem::with_id(app, "hide", "Hide Window", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_item, &hide_item, &separator, &quit_item])?;

    let icon = load_tray_icon()?;

    let _tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(icon)
        .menu(&menu)
        .tooltip("ClickDevPort")
        .on_menu_event(move |app, event| {
            handle_menu_event(app, event.id.as_ref());
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}

fn load_tray_icon() -> Result<Image<'static>, Box<dyn std::error::Error>> {
    // Load embedded PNG icon - convert to RGBA format
    // The 32x32 PNG is already included in the binary
    let icon_bytes = include_bytes!("../icons/32x32.png");

    // Decode the PNG to get raw RGBA pixels
    let img = image::load_from_memory(icon_bytes)
        .map_err(|e| format!("Failed to load icon: {}", e))?;
    let rgba_img = img.to_rgba8();
    let (width, height) = rgba_img.dimensions();
    let rgba_bytes = rgba_img.into_raw();

    Ok(Image::new_owned(rgba_bytes, width, height))
}

fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, menu_id: &str) {
    match menu_id {
        "show" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "hide" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }
        }
        "quit" => {
            // Exit without stopping services - let external services continue running
            app.exit(0);
        }
        _ => {}
    }
}

// Note: stop_all_services_and_quit function removed - app should not stop external services

/// Update the tray icon based on service status
/// Call this function when service status changes
#[allow(dead_code)]
pub async fn update_tray_status<R: Runtime>(app: &AppHandle<R>) {
    if let Some(service_manager) = app.try_state::<Arc<Mutex<ServiceManager>>>() {
        let manager = service_manager.lock().await;

        // Check if any service is running
        let any_running = manager
            .services
            .values()
            .any(|service| service.is_running());

        // Get the tray icon and update tooltip based on status
        if let Some(tray) = app.tray_by_id(TRAY_ID) {
            let tooltip = if any_running {
                "ClickDevPort - Services Running"
            } else {
                "ClickDevPort - No Services Running"
            };
            let _ = tray.set_tooltip(Some(tooltip));
        }
    }
}
