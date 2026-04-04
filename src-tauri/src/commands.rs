//! Tauri commands — bridge between Vue frontend and wsl-porthole-core.

use serde::Serialize;
use std::path::PathBuf;
use wsl_porthole_core::config::{self, RuleConfig};
use wsl_porthole_core::rules::{Direction, Rule};
use wsl_porthole_core::settings::{self, AppSettings};

/// Stable config directory: %APPDATA%\WSL PortHole\ on Windows.
/// Never uses current_dir() which is unpredictable for GUI apps.
fn app_data_dir() -> PathBuf {
    let base = dirs::data_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")));
    let dir = base.join("WSL PortHole");
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    dir
}

fn config_path() -> PathBuf {
    app_data_dir().join("wsl-porthole-rules.json")
}

fn settings_path() -> PathBuf {
    app_data_dir().join("wsl-porthole-settings.json")
}

fn log_path() -> PathBuf {
    app_data_dir().join("wsl-porthole.log")
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
    let rule = cfg.rules.iter_mut().find(|r| r.id == id).ok_or("Rule not found")?;
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
            wsl_porthole_core::netsh::apply_rule(rule, &wsl_ip, &host_gw).map_err(|e| e.to_string())?;
            applied += 1;
        }
    }
    Ok(format!("Applied {applied} rules (WSL IP: {wsl_ip})"))
}

#[tauri::command]
pub fn remove_applied_rule(id: String) -> Result<(), String> {
    let cfg = config::load_rules(&config_path()).map_err(|e| e.to_string())?;
    let rule = cfg.rules.iter().find(|r| r.id == id).ok_or("Rule not found")?;
    wsl_porthole_core::netsh::remove_rule(rule).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_active_proxies() -> Result<Vec<ActiveProxyInfo>, String> {
    let proxies = wsl_porthole_core::netsh::list_active().map_err(|e| e.to_string())?;
    Ok(proxies.into_iter().map(|p| ActiveProxyInfo {
        listen_addr: p.listen_addr, listen_port: p.listen_port,
        connect_addr: p.connect_addr, connect_port: p.connect_port,
    }).collect())
}

#[derive(Serialize)]
pub struct ActiveProxyInfo {
    listen_addr: String, listen_port: u16,
    connect_addr: String, connect_port: u16,
}

#[tauri::command]
pub fn preview_netsh_command(id: String) -> Result<Vec<String>, String> {
    let cfg = config::load_rules(&config_path()).map_err(|e| e.to_string())?;
    let rule = cfg.rules.iter().find(|r| r.id == id).ok_or("Rule not found")?;
    let wsl_ip = wsl_porthole_core::ip::detect_wsl_ip().unwrap_or_else(|_| "?.?.?.?".into());
    let host_gw = wsl_porthole_core::ip::detect_host_gateway().unwrap_or_default();
    Ok(wsl_porthole_core::netsh::preview_command(rule, &wsl_ip, &host_gw))
}

// ---------- Status ----------

#[derive(Serialize)]
pub struct StatusInfo {
    wsl_ip: Option<String>,
    host_ip: Option<String>,
    host_gw: Option<String>,
    active_rules: usize,
    lan_rules: usize,
    total_rules: usize,
    wsl_error: Option<String>,
    host_error: Option<String>,
    config_dir: String,
}

#[tauri::command]
pub fn get_status() -> Result<StatusInfo, String> {
    let cfg = config::load_rules(&config_path()).map_err(|e| e.to_string())?;

    let wsl_result = wsl_porthole_core::ip::detect_wsl_ip();
    let host_result = wsl_porthole_core::ip::detect_host_ip();

    Ok(StatusInfo {
        wsl_ip: wsl_result.as_ref().ok().cloned(),
        host_ip: host_result.as_ref().ok().cloned(),
        host_gw: wsl_porthole_core::ip::detect_host_gateway().ok(),
        active_rules: cfg.rules.iter().filter(|r| r.enabled).count(),
        lan_rules: cfg.rules.iter().filter(|r| r.enabled && r.lan).count(),
        total_rules: cfg.rules.len(),
        wsl_error: wsl_result.err().map(|e| e.to_string()),
        host_error: host_result.err().map(|e| e.to_string()),
        config_dir: app_data_dir().to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub fn detect_ips() -> serde_json::Value {
    serde_json::json!({
        "wsl_ip": wsl_porthole_core::ip::detect_wsl_ip().ok(),
        "host_ip": wsl_porthole_core::ip::detect_host_ip().ok(),
        "host_gw": wsl_porthole_core::ip::detect_host_gateway().ok(),
    })
}

#[tauri::command]
pub fn sync_now() -> Result<String, String> { apply_rules() }

// ---------- Diagnostics ----------

#[tauri::command]
pub fn diagnose() -> serde_json::Value {
    use std::process::Command;
    use wsl_porthole_core::sys_path;

    let wsl_path = sys_path::wsl();
    let wsl_exists = std::path::Path::new(wsl_path).exists();
    let wsl_test = Command::new(wsl_path).arg("--version").output();
    let wsl_version = match &wsl_test {
        Ok(o) if o.status.success() => {
            let raw = &o.stdout;
            // Try UTF-16LE then UTF-8
            let text = if raw.len() >= 2 {
                let u16s: Vec<u16> = raw.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
                String::from_utf16(&u16s).unwrap_or_else(|_| String::from_utf8_lossy(raw).to_string())
            } else {
                String::from_utf8_lossy(raw).to_string()
            };
            text.trim().to_string()
        }
        Ok(o) => format!("exit code: {}, stderr: {}", o.status, String::from_utf8_lossy(&o.stderr).trim()),
        Err(e) => format!("failed to execute: {e}"),
    };

    let netsh_path = sys_path::netsh();
    let netsh_exists = std::path::Path::new(netsh_path).exists();

    let ps_path = sys_path::powershell();
    let ps_exists = std::path::Path::new(ps_path).exists();

    let docker_test = tokio::runtime::Handle::try_current()
        .map(|_| "async runtime available".to_string())
        .unwrap_or_else(|_| "no async runtime".to_string());

    let sc_path = sys_path::sc();
    let service_test = Command::new(sc_path).args(["query", "WslPortHole"]).output();
    let service_status = match &service_test {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            if stdout.contains("RUNNING") { "running".to_string() }
            else if stdout.contains("STOPPED") { "stopped".to_string() }
            else { format!("unknown: {}", stdout.trim()) }
        }
        Err(e) => format!("query failed: {e}"),
    };

    serde_json::json!({
        "config_dir": app_data_dir().to_string_lossy(),
        "config_exists": config_path().exists(),
        "settings_exists": settings_path().exists(),
        "wsl": {
            "path": wsl_path,
            "exists": wsl_exists,
            "version": wsl_version,
        },
        "netsh": {
            "path": netsh_path,
            "exists": netsh_exists,
        },
        "powershell": {
            "path": ps_path,
            "exists": ps_exists,
        },
        "docker": docker_test,
        "service": service_status,
        "log_file": log_path().to_string_lossy(),
    })
}

// ---------- Import / Export ----------

#[tauri::command]
pub fn import_netsh_script(script: String) -> Result<Vec<Rule>, String> {
    Ok(wsl_porthole_core::import::parse_netsh_script(&script))
}

#[tauri::command]
pub fn export_netsh_script() -> Result<String, String> {
    let cfg = config::load_rules(&config_path()).map_err(|e| e.to_string())?;
    Ok(wsl_porthole_core::export::export_netsh_script(&cfg.rules))
}

// ---------- WSL Distros ----------

#[tauri::command]
pub fn list_distros() -> Result<Vec<wsl_porthole_core::ip::DistroInfo>, String> {
    wsl_porthole_core::ip::list_distros().map_err(|e| e.to_string())
}

// ---------- Docker ----------

#[derive(Serialize)]
pub struct ContainerSummary {
    id: String, name: String, image: String, status: String,
    ports: Vec<PortSummary>, compose_project: Option<String>,
}

#[derive(Serialize)]
pub struct PortSummary { host_port: u16, container_port: u16, protocol: String }

#[tauri::command]
pub async fn list_docker_containers(engine: Option<String>) -> Result<Vec<ContainerSummary>, String> {
    let containers = match engine.as_deref() {
        Some("windows") => wsl_porthole_core::docker::list_windows_containers().await,
        _ => wsl_porthole_core::docker::list_wsl_containers().await,
    }.map_err(|e| e.to_string())?;

    Ok(containers.into_iter().map(|c| ContainerSummary {
        id: c.id, name: c.name, image: c.image, status: c.status,
        ports: c.ports.into_iter().map(|p| PortSummary {
            host_port: p.host_port, container_port: p.container_port, protocol: p.protocol,
        }).collect(),
        compose_project: c.compose_project,
    }).collect())
}

#[derive(Serialize)]
pub struct McpServerInfo {
    container_name: String, image: String, port: u16, host_port: u16, detection_reason: String,
}

#[tauri::command]
pub async fn detect_mcp_servers() -> Result<Vec<McpServerInfo>, String> {
    let containers = wsl_porthole_core::docker::list_windows_containers().await.map_err(|e| e.to_string())?;
    let servers = wsl_porthole_core::mcp::detect_mcp_servers(&containers);
    Ok(servers.into_iter().map(|s| McpServerInfo {
        container_name: s.container_name, image: s.image, port: s.port,
        host_port: s.host_port, detection_reason: format!("{:?}", s.detection_reason),
    }).collect())
}

// ---------- Health / Conflicts ----------

#[tauri::command]
pub async fn check_health() -> Result<Vec<wsl_porthole_core::health::HealthResult>, String> {
    let cfg = config::load_rules(&config_path()).map_err(|e| e.to_string())?;
    Ok(wsl_porthole_core::health::check_all_async(&cfg.rules).await)
}

#[tauri::command]
pub fn detect_conflicts() -> Result<Vec<wsl_porthole_core::conflict::Conflict>, String> {
    let cfg = config::load_rules(&config_path()).map_err(|e| e.to_string())?;
    wsl_porthole_core::conflict::detect_conflicts(&cfg.rules).map_err(|e| e.to_string())
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

// ---------- Settings / Groups / Startup Actions ----------

#[tauri::command]
pub fn get_settings() -> Result<AppSettings, String> {
    settings::load_settings(&settings_path()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_settings(data: AppSettings) -> Result<(), String> {
    settings::save_settings(&settings_path(), &data).map_err(|e| e.to_string())
}

// ---------- .wslconfig ----------

#[tauri::command]
pub fn read_wslconfig() -> Result<String, String> {
    let home = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".into());
    let path = PathBuf::from(home).join(".wslconfig");
    if path.exists() {
        std::fs::read_to_string(&path).map_err(|e| e.to_string())
    } else {
        Ok(String::new())
    }
}

#[tauri::command]
pub fn write_wslconfig(content: String) -> Result<(), String> {
    let home = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".into());
    let path = PathBuf::from(home).join(".wslconfig");
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

// ---------- Service management ----------

#[tauri::command]
pub fn install_service() -> Result<String, String> {
    let status = std::process::Command::new("wsl-porthole-service").arg("install").status().map_err(|e| e.to_string())?;
    if status.success() { Ok("Service installed".into()) } else { Err("Failed to install service".into()) }
}

#[tauri::command]
pub fn uninstall_service() -> Result<String, String> {
    let status = std::process::Command::new("wsl-porthole-service").arg("uninstall").status().map_err(|e| e.to_string())?;
    if status.success() { Ok("Service uninstalled".into()) } else { Err("Failed to uninstall service".into()) }
}

#[tauri::command]
pub fn get_service_status() -> Result<String, String> {
    let output = std::process::Command::new(r"C:\Windows\System32\sc.exe").args(["query", "WslPortHole"]).output().map_err(|e| e.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("RUNNING") { Ok("running".into()) }
    else if stdout.contains("STOPPED") { Ok("stopped".into()) }
    else { Ok("not_installed".into()) }
}

// ---------- Updater ----------

/// Check GitHub releases for a newer version.
/// The Tauri updater plugin requires signing keys we don't have,
/// so we query the GitHub API directly from Rust (no CORS issues).
#[tauri::command]
pub async fn check_for_app_updates(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let current = app.config().version.clone().unwrap_or_default();

    // Query GitHub releases API
    let client = reqwest::Client::builder()
        .user_agent("WSL-PortHole-Updater")
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get("https://api.github.com/repos/immersedone/wsl-porthole/releases/latest")
        .send()
        .await
        .map_err(|e| format!("Failed to check for updates: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("GitHub API returned {}", resp.status()));
    }

    let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let tag = data["tag_name"].as_str().unwrap_or("").trim_start_matches('v');

    if tag.is_empty() {
        return Ok(None);
    }

    // Simple version comparison (works for semver like 0.4.1 vs 0.4.2)
    if tag != current {
        Ok(Some(tag.to_string()))
    } else {
        Ok(None)
    }
}
