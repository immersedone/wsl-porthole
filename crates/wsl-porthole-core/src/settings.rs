//! App settings, groups, and startup actions persistence.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default)]
    pub groups: Vec<RuleGroup>,
    #[serde(default)]
    pub startup_actions: Vec<StartupAction>,
    #[serde(default)]
    pub preferences: Preferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleGroup {
    pub id: String,
    pub name: String,
    pub rule_ids: Vec<String>,
    pub enabled: bool,
    #[serde(default)]
    pub startup_behavior: String, // "none" | "enable" | "disable"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupAction {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub action_type: String, // "builtin" | "custom"
    pub command: String,
    pub delay_ms: u64,
    pub enabled: bool,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    #[serde(default)]
    pub start_minimized: bool,
    #[serde(default = "default_true")]
    pub minimize_to_tray: bool,
    #[serde(default = "default_60")]
    pub health_check_interval: u64,
    #[serde(default = "default_5")]
    pub ip_settle_delay: u64,
    #[serde(default = "default_30")]
    pub polling_interval: u64,
    #[serde(default = "default_listen_addr")]
    pub default_listen_addr: String,
    #[serde(default = "default_true")]
    pub toast_on_ip_change: bool,
    #[serde(default = "default_true")]
    pub toast_on_conflict: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_true() -> bool { true }
fn default_60() -> u64 { 60 }
fn default_5() -> u64 { 5 }
fn default_30() -> u64 { 30 }
fn default_listen_addr() -> String { "0.0.0.0".into() }
fn default_theme() -> String { "mission-control".into() }

impl Default for Preferences {
    fn default() -> Self {
        Self {
            start_minimized: false,
            minimize_to_tray: true,
            health_check_interval: 60,
            ip_settle_delay: 5,
            polling_interval: 30,
            default_listen_addr: "0.0.0.0".into(),
            toast_on_ip_change: true,
            toast_on_conflict: true,
            theme: "mission-control".into(),
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            groups: Vec::new(),
            startup_actions: vec![
                StartupAction {
                    id: "builtin-sync".into(),
                    label: "Sync port rules".into(),
                    action_type: "builtin".into(),
                    command: "sync-rules".into(),
                    delay_ms: 0,
                    enabled: true,
                    target: "all".into(),
                },
                StartupAction {
                    id: "builtin-hosts".into(),
                    label: "Write /etc/hosts".into(),
                    action_type: "builtin".into(),
                    command: "write-hosts".into(),
                    delay_ms: 1000,
                    enabled: true,
                    target: "all".into(),
                },
                StartupAction {
                    id: "builtin-env".into(),
                    label: "Inject env vars".into(),
                    action_type: "builtin".into(),
                    command: "inject-env".into(),
                    delay_ms: 2000,
                    enabled: false,
                    target: "all".into(),
                },
            ],
            preferences: Preferences::default(),
        }
    }
}

pub fn load_settings(path: &Path) -> Result<AppSettings> {
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    let contents = std::fs::read_to_string(path)?;
    let settings: AppSettings = serde_json::from_str(&contents)?;
    Ok(settings)
}

pub fn save_settings(path: &Path, settings: &AppSettings) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(settings)?;
    std::fs::write(path, json)?;
    Ok(())
}
