// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
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
            commands::write_hosts_entry,
            commands::inject_env_var,
            commands::get_firewall_rules,
            commands::install_service,
            commands::uninstall_service,
            commands::get_service_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running WSL PortHole");
}
