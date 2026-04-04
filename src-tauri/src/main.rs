// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
use tauri_plugin_updater::UpdaterExt;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // If a second instance is launched, focus the existing window
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            // Build tray menu
            let show_i = MenuItem::with_id(app, "show", "Open WSL PortHole", true, None::<&str>)?;
            let sync_i = MenuItem::with_id(app, "sync", "Sync Now", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Exit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &sync_i, &quit_i])?;

            // Create tray icon
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("WSL PortHole")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "sync" => {
                        // Trigger sync via the existing command
                        let _ = commands::apply_rules();
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
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
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Check for updates on startup (non-blocking)
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                check_for_updates(handle).await;
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            // Minimize to tray on close instead of exiting
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_rules,
            commands::save_rules,
            commands::add_rule,
            commands::update_rule,
            commands::delete_rule,
            commands::toggle_rule,
            commands::apply_rules,
            commands::remove_applied_rule,
            commands::get_status,
            commands::detect_ips,
            commands::sync_now,
            commands::list_active_proxies,
            commands::preview_netsh_command,
            commands::import_netsh_script,
            commands::list_docker_containers,
            commands::detect_mcp_servers,
            commands::check_health,
            commands::detect_conflicts,
            commands::export_netsh_script,
            commands::list_distros,
            commands::write_hosts_entry,
            commands::inject_env_var,
            commands::get_firewall_rules,
            commands::get_settings,
            commands::save_settings,
            commands::read_wslconfig,
            commands::write_wslconfig,
            commands::install_service,
            commands::uninstall_service,
            commands::get_service_status,
            commands::check_for_app_updates,
        ])
        .run(tauri::generate_context!())
        .expect("error while running WSL PortHole");
}

/// Check for updates using the Tauri updater plugin.
async fn check_for_updates(app: tauri::AppHandle) {
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let updater = match app.updater() {
        Ok(u) => u,
        Err(e) => {
            tracing::debug!("Updater not available: {e}");
            return;
        }
    };

    match updater.check().await {
        Ok(Some(update)) => {
            tracing::info!("Update available: v{}", update.version);
        }
        Ok(None) => {
            tracing::debug!("No updates available");
        }
        Err(e) => {
            tracing::debug!("Update check failed: {e}");
        }
    }
}
