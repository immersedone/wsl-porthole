// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
fn main() {
    // Initialize file logging to %APPDATA%\WSL PortHole\wsl-porthole.log
    let log_dir = dirs::data_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from(".")))
        .join("WSL PortHole");
    let _ = std::fs::create_dir_all(&log_dir);
    let file_appender = tracing_appender::rolling::never(&log_dir, "wsl-porthole.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(false)
        .init();

    tracing::info!("WSL PortHole starting — config dir: {}", log_dir.display());

    // Self-test: can we execute wsl.exe from this process?
    {
        use wsl_porthole_core::sys_path;
        let wsl_path = sys_path::wsl();
        tracing::info!("WSL path resolved to: {wsl_path}");
        tracing::info!("WSL path exists: {}", std::path::Path::new(wsl_path).exists());

        match sys_path::command(wsl_path).args(["hostname", "-I"]).output() {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                tracing::info!("WSL self-test: status={}, stdout={:?}", out.status, stdout.trim());
            }
            Err(e) => {
                tracing::error!("WSL self-test FAILED: {e}");
            }
        }

        match sys_path::command(sys_path::ipconfig()).output() {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let ipv4_lines: Vec<&str> = stdout.lines().filter(|l| l.contains("IPv4")).collect();
                tracing::info!("ipconfig self-test: status={}, IPv4 lines={}", out.status, ipv4_lines.len());
            }
            Err(e) => {
                tracing::error!("ipconfig self-test FAILED: {e}");
            }
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
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

            // Screenshot automation: if SCREENSHOT_MODE env var is set,
            // cycle through all pages and take screenshots
            if std::env::var("WSL_PORTHOLE_SCREENSHOTS").is_ok() {
                let window = app.get_webview_window("main").unwrap();
                let out_dir = std::env::var("WSL_PORTHOLE_SCREENSHOT_DIR")
                    .unwrap_or_else(|_| r"C:\Users\Immersed\Desktop\wsl-screenshots".into());
                std::fs::create_dir_all(&out_dir).ok();

                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

                    let pages = [
                        "rules", "groups", "docker", "mcp", "lan", "firewall",
                        "distros", "startup", "service", "wslconfig", "audit",
                        "appearance", "updates", "settings",
                    ];
                    let names = [
                        "01-rules", "02-groups", "03-docker", "04-mcp", "05-lan",
                        "06-firewall", "07-distros", "08-startup", "09-service",
                        "10-wslconfig", "11-audit", "12-appearance", "13-updates",
                        "14-settings",
                    ];

                    for (page, name) in pages.iter().zip(names.iter()) {
                        let js = format!("window.__navigateTo && window.__navigateTo('{page}')");
                        let _ = window.eval(&js);
                        // Wait for Vue to render the page change
                        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

                        // Signal PowerShell to take screenshot
                        let marker = format!("{out_dir}\\__ready_{name}");
                        let _ = std::fs::write(&marker, "ready");
                        tracing::info!("Screenshot ready: {name}");
                        // Wait for PowerShell to capture before navigating again
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    }

                    // Signal completion
                    let _ = std::fs::write(format!("{out_dir}\\__done"), "done");
                    tracing::info!("Screenshot automation complete");
                });
            }

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
            commands::diagnose,
            commands::navigate_to,
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
            commands::download_and_install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running WSL PortHole");
}

