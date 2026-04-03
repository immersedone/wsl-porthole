//! WSL IP change watcher.
//!
//! 1. Subscribe to Hyper-V VmSwitch event log (Event ID 102) — TODO
//! 2. On event → wait 5s for IP to settle
//! 3. `wsl hostname -I` → compare to last known IP
//! 4. If changed → re-apply all rules + update firewall
//! 5. Toast notification to user

use std::path::Path;
use wsl_porthole_core::config;
use wsl_porthole_core::firewall;
use wsl_porthole_core::ip::detect_wsl_ip;
use wsl_porthole_core::netsh;
use wsl_porthole_core::rules::Direction;

const CONFIG_PATH: &str = "wsl-porthole-rules.json";
const POLL_INTERVAL_SECS: u64 = 30;
const IP_SETTLE_DELAY_SECS: u64 = 5;

/// Apply all current rules from config for the given WSL IP. Returns the rule count.
pub fn apply_current_rules(wsl_ip: &str) -> anyhow::Result<usize> {
    let config_path = Path::new(CONFIG_PATH);
    let cfg = config::load_rules(config_path)?;

    let host_gw = wsl_porthole_core::ip::detect_host_gateway().unwrap_or_default();
    let mut applied = 0;

    for rule in &cfg.rules {
        if !rule.enabled {
            continue;
        }
        if let Err(e) = netsh::apply_rule(rule, wsl_ip, &host_gw) {
            tracing::error!("Failed to apply rule '{}': {e}", rule.name);
            continue;
        }
        // Sync firewall rules atomically
        match rule.direction {
            Direction::WinToWsl => {
                if let Err(e) = firewall::add_inbound_rule(rule) {
                    tracing::error!("Failed to add firewall rule '{}': {e}", rule.name);
                }
            }
            Direction::WslToWin => {
                if let Err(e) = firewall::add_wsl_interface_rule(rule) {
                    tracing::error!("Failed to add WSL interface rule '{}': {e}", rule.name);
                }
            }
        }
        applied += 1;
    }

    tracing::info!("Applied {applied}/{} rules with WSL IP {wsl_ip}", cfg.rules.len());
    Ok(applied)
}

/// Main watcher loop — polls for WSL IP changes and re-applies rules.
///
/// TODO: Replace polling with Hyper-V VmSwitch Event ID 102 subscription
/// for event-driven detection. The polling loop serves as a reliable fallback.
pub async fn watch_loop(last_ip: &mut String) {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;

        match detect_wsl_ip() {
            Ok(ip) if ip != *last_ip => {
                tracing::info!("WSL IP changed: {} → {}", last_ip, ip);

                // Wait for IP to settle after WSL restart
                tokio::time::sleep(std::time::Duration::from_secs(IP_SETTLE_DELAY_SECS)).await;

                // Re-detect after settle delay to confirm
                let confirmed_ip = match detect_wsl_ip() {
                    Ok(ip) => ip,
                    Err(e) => {
                        tracing::error!("Failed to confirm WSL IP after settle: {e}");
                        continue;
                    }
                };

                // Reset and reapply
                if let Err(e) = netsh::reset_all() {
                    tracing::error!("Failed to reset portproxy rules: {e}");
                }

                match apply_current_rules(&confirmed_ip) {
                    Ok(count) => {
                        tracing::info!("Reapplied {count} rules for new IP: {confirmed_ip}");
                        send_toast_notification(&confirmed_ip, count);
                    }
                    Err(e) => {
                        tracing::error!("Failed to reapply rules: {e}");
                    }
                }

                *last_ip = confirmed_ip;
            }
            Ok(_) => {} // IP unchanged
            Err(e) => {
                tracing::warn!("Failed to detect WSL IP: {e}");
            }
        }
    }
}

/// Send a Windows toast notification about the IP change.
fn send_toast_notification(new_ip: &str, rule_count: usize) {
    // Use PowerShell to send toast (works without COM registration)
    let message = format!("WSL PortHole: {rule_count} rules updated (new IP: {new_ip})");
    let _ = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] > $null; \
                 $xml = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText01); \
                 $xml.GetElementsByTagName('text')[0].AppendChild($xml.CreateTextNode('{message}')) > $null; \
                 $toast = [Windows.UI.Notifications.ToastNotification]::new($xml); \
                 [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('WSL PortHole').Show($toast)"
            ),
        ])
        .status();
}
