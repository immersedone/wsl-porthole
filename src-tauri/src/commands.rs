//! Tauri commands — bridge between React frontend and wsl-porthole-core.

use serde::Serialize;
use std::path::PathBuf;
use wsl_porthole_core::config::{self, RuleConfig};
use wsl_porthole_core::rules::{Direction, Rule};

fn config_path() -> PathBuf {
    // Use app-local config in production; fallback to CWD for dev
    dirs_or_cwd().join("wsl-porthole-rules.json")
}

fn dirs_or_cwd() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

// ---------- Rule CRUD ----------

#[tauri::command]
pub fn get_rules() -> Result<Vec<Rule>, String> {
    let cfg = config::load_rules(&config_path()).map_err(|e| e.to_string())?;
    Ok(cfg.rules)
}

#[tauri::command]
pub fn save_rules(rules: Vec<Rule>) -> Result<(), String> {
    let cfg = RuleConfig {
        version: 1,
        distro: "auto".into(),
        rules,
    };
    config::save_rules(&config_path(), &cfg).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_rule(
    name: String,
    direction: String,
    listen_addr: String,
    listen_port: u16,
    connect_port: u16,
    connect_addr: String,
    lan: bool,
) -> Result<Rule, String> {
    let dir = match direction.as_str() {
        "WslToWin" => Direction::WslToWin,
        _ => Direction::WinToWsl,
    };
    let mut rule = Rule::new(&name, dir, listen_port, connect_port);
    rule.listen_addr = listen_addr;
    rule.connect_addr = connect_addr;
    rule.lan = lan;

    let mut cfg = config::load_rules(&config_path()).map_err(|e| e.to_string())?;
    cfg.rules.push(rule.clone());
    config::save_rules(&config_path(), &cfg).map_err(|e| e.to_string())?;
    Ok(rule)
}

#[tauri::command]
pub fn update_rule(rule: Rule) -> Result<(), String> {
    let mut cfg = config::load_rules(&config_path()).map_err(|e| e.to_string())?;
    if let Some(existing) = cfg.rules.iter_mut().find(|r| r.id == rule.id) {
        *existing = rule;
    } else {
        return Err("Rule not found".into());
    }
    config::save_rules(&config_path(), &cfg).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_rule(id: String) -> Result<(), String> {
    let mut cfg = config::load_rules(&config_path()).map_err(|e| e.to_string())?;
    cfg.rules.retain(|r| r.id != id);
    config::save_rules(&config_path(), &cfg).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_rule(id: String) -> Result<bool, String> {
    let mut cfg = config::load_rules(&config_path()).map_err(|e| e.to_string())?;
    let rule = cfg
        .rules
        .iter_mut()
        .find(|r| r.id == id)
        .ok_or("Rule not found")?;
    rule.enabled = !rule.enabled;
    let new_state = rule.enabled;
    config::save_rules(&config_path(), &cfg).map_err(|e| e.to_string())?;
    Ok(new_state)
}

// ---------- Netsh operations ----------

#[tauri::command]
pub fn apply_rules() -> Result<String, String> {
    let cfg = config::load_rules(&config_path()).map_err(|e| e.to_string())?;
    let wsl_ip = wsl_porthole_core::ip::detect_wsl_ip().map_err(|e| e.to_string())?;
    let host_gw = wsl_porthole_core::ip::detect_host_gateway().unwrap_or_default();

    let mut applied = 0;
    for rule in &cfg.rules {
        if rule.enabled {
            wsl_porthole_core::netsh::apply_rule(rule, &wsl_ip, &host_gw)
                .map_err(|e| e.to_string())?;
            applied += 1;
        }
    }
    Ok(format!("Applied {applied} rules (WSL IP: {wsl_ip})"))
}

#[tauri::command]
pub fn remove_applied_rule(id: String) -> Result<(), String> {
    let cfg = config::load_rules(&config_path()).map_err(|e| e.to_string())?;
    let rule = cfg
        .rules
        .iter()
        .find(|r| r.id == id)
        .ok_or("Rule not found")?;
    wsl_porthole_core::netsh::remove_rule(rule).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_active_proxies() -> Result<Vec<ActiveProxyInfo>, String> {
    let proxies = wsl_porthole_core::netsh::list_active().map_err(|e| e.to_string())?;
    Ok(proxies
        .into_iter()
        .map(|p| ActiveProxyInfo {
            listen_addr: p.listen_addr,
            listen_port: p.listen_port,
            connect_addr: p.connect_addr,
            connect_port: p.connect_port,
        })
        .collect())
}

#[derive(Serialize)]
pub struct ActiveProxyInfo {
    listen_addr: String,
    listen_port: u16,
    connect_addr: String,
    connect_port: u16,
}

#[tauri::command]
pub fn preview_netsh_command(id: String) -> Result<Vec<String>, String> {
    let cfg = config::load_rules(&config_path()).map_err(|e| e.to_string())?;
    let rule = cfg
        .rules
        .iter()
        .find(|r| r.id == id)
        .ok_or("Rule not found")?;
    let wsl_ip = wsl_porthole_core::ip::detect_wsl_ip().unwrap_or_else(|_| "?.?.?.?".into());
    let host_gw = wsl_porthole_core::ip::detect_host_gateway().unwrap_or_default();
    Ok(wsl_porthole_core::netsh::preview_command(
        rule, &wsl_ip, &host_gw,
    ))
}

// ---------- Status / IP detection ----------

#[derive(Serialize)]
pub struct StatusInfo {
    wsl_ip: Option<String>,
    host_ip: Option<String>,
    host_gw: Option<String>,
    active_rules: usize,
    lan_rules: usize,
    total_rules: usize,
}

#[tauri::command]
pub fn get_status() -> Result<StatusInfo, String> {
    let cfg = config::load_rules(&config_path()).map_err(|e| e.to_string())?;
    let active = cfg.rules.iter().filter(|r| r.enabled).count();
    let lan = cfg.rules.iter().filter(|r| r.enabled && r.lan).count();

    Ok(StatusInfo {
        wsl_ip: wsl_porthole_core::ip::detect_wsl_ip().ok(),
        host_ip: wsl_porthole_core::ip::detect_host_ip().ok(),
        host_gw: wsl_porthole_core::ip::detect_host_gateway().ok(),
        active_rules: active,
        lan_rules: lan,
        total_rules: cfg.rules.len(),
    })
}

#[derive(Serialize)]
pub struct IpInfo {
    wsl_ip: Option<String>,
    host_ip: Option<String>,
    host_gw: Option<String>,
}

#[tauri::command]
pub fn detect_ips() -> IpInfo {
    IpInfo {
        wsl_ip: wsl_porthole_core::ip::detect_wsl_ip().ok(),
        host_ip: wsl_porthole_core::ip::detect_host_ip().ok(),
        host_gw: wsl_porthole_core::ip::detect_host_gateway().ok(),
    }
}

#[tauri::command]
pub fn sync_now() -> Result<String, String> {
    apply_rules()
}

// ---------- Import ----------

#[tauri::command]
pub fn import_netsh_script(script: String) -> Result<Vec<Rule>, String> {
    Ok(wsl_porthole_core::import::parse_netsh_script(&script))
}

// ---------- Docker ----------

#[derive(Serialize)]
pub struct ContainerSummary {
    id: String,
    name: String,
    image: String,
    status: String,
    ports: Vec<PortSummary>,
    compose_project: Option<String>,
}

#[derive(Serialize)]
pub struct PortSummary {
    host_port: u16,
    container_port: u16,
    protocol: String,
}

#[tauri::command]
pub async fn list_docker_containers() -> Result<Vec<ContainerSummary>, String> {
    let containers = wsl_porthole_core::docker::list_wsl_containers()
        .await
        .map_err(|e| e.to_string())?;

    Ok(containers
        .into_iter()
        .map(|c| ContainerSummary {
            id: c.id,
            name: c.name,
            image: c.image,
            status: c.status,
            ports: c
                .ports
                .into_iter()
                .map(|p| PortSummary {
                    host_port: p.host_port,
                    container_port: p.container_port,
                    protocol: p.protocol,
                })
                .collect(),
            compose_project: c.compose_project,
        })
        .collect())
}

#[derive(Serialize)]
pub struct McpServerInfo {
    container_name: String,
    image: String,
    port: u16,
    host_port: u16,
    detection_reason: String,
}

#[tauri::command]
pub async fn detect_mcp_servers() -> Result<Vec<McpServerInfo>, String> {
    let containers = wsl_porthole_core::docker::list_windows_containers()
        .await
        .map_err(|e| e.to_string())?;

    let servers = wsl_porthole_core::mcp::detect_mcp_servers(&containers);
    Ok(servers
        .into_iter()
        .map(|s| McpServerInfo {
            container_name: s.container_name,
            image: s.image,
            port: s.port,
            host_port: s.host_port,
            detection_reason: format!("{:?}", s.detection_reason),
        })
        .collect())
}

// ---------- Health checks ----------

#[tauri::command]
pub async fn check_health() -> Result<Vec<wsl_porthole_core::health::HealthResult>, String> {
    let cfg = config::load_rules(&config_path()).map_err(|e| e.to_string())?;
    Ok(wsl_porthole_core::health::check_all_async(&cfg.rules).await)
}

// ---------- Conflict detection ----------

#[tauri::command]
pub fn detect_conflicts() -> Result<Vec<wsl_porthole_core::conflict::Conflict>, String> {
    let cfg = config::load_rules(&config_path()).map_err(|e| e.to_string())?;
    wsl_porthole_core::conflict::detect_conflicts(&cfg.rules).map_err(|e| e.to_string())
}

// ---------- Export ----------

#[tauri::command]
pub fn export_netsh_script() -> Result<String, String> {
    let cfg = config::load_rules(&config_path()).map_err(|e| e.to_string())?;
    Ok(wsl_porthole_core::export::export_netsh_script(&cfg.rules))
}

// ---------- Inject ----------

#[tauri::command]
pub fn write_hosts_entry(hostname: String, ip: String) -> Result<(), String> {
    wsl_porthole_core::inject::write_hosts_entry(&hostname, &ip, None).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn inject_env_var(name: String, value: String) -> Result<(), String> {
    wsl_porthole_core::inject::inject_env_var(&name, &value, None).map_err(|e| e.to_string())
}

// ---------- Firewall ----------

#[tauri::command]
pub fn get_firewall_rules() -> Result<Vec<String>, String> {
    wsl_porthole_core::firewall::list_rules().map_err(|e| e.to_string())
}

// ---------- Service management ----------

#[tauri::command]
pub fn install_service() -> Result<String, String> {
    let status = std::process::Command::new("wsl-porthole-service")
        .arg("install")
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok("Service installed".into())
    } else {
        Err("Failed to install service".into())
    }
}

#[tauri::command]
pub fn uninstall_service() -> Result<String, String> {
    let status = std::process::Command::new("wsl-porthole-service")
        .arg("uninstall")
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok("Service uninstalled".into())
    } else {
        Err("Failed to uninstall service".into())
    }
}

#[tauri::command]
pub fn get_service_status() -> Result<String, String> {
    let output = std::process::Command::new("sc")
        .args(["query", "WslPortHole"])
        .output()
        .map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("RUNNING") {
        Ok("running".into())
    } else if stdout.contains("STOPPED") {
        Ok("stopped".into())
    } else {
        Ok("not_installed".into())
    }
}
