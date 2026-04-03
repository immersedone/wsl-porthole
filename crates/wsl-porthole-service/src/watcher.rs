//! WSL IP change watcher.
//!
//! Two detection modes:
//! 1. **Event-driven** (primary): Subscribes to Hyper-V VmSwitch event log
//!    (Event ID 102) via `wevtutil`. Reacts within seconds of WSL IP change.
//! 2. **Polling** (fallback): Checks every 30s if event subscription fails.
//!
//! On IP change → wait 5s for settle → re-apply all rules + firewall → toast.

use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
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

/// Main watcher loop — tries event-driven first, falls back to polling.
pub async fn watch_loop(last_ip: &mut String) {
    // Try event-driven mode first
    tracing::info!("Attempting Hyper-V event subscription...");
    tokio::select! {
        result = event_driven_loop(last_ip) => {
            match result {
                Ok(()) => return,
                Err(e) => {
                    tracing::warn!("Event subscription failed: {e}. Falling back to polling.");
                }
            }
        }
        // Give the event subscription 5s to start. If it fails immediately, fall through.
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
            tracing::info!("Event subscription appears stable, continuing...");
            // Re-enter event loop indefinitely
            if let Err(e) = event_driven_loop(last_ip).await {
                tracing::warn!("Event subscription lost: {e}. Switching to polling.");
            }
        }
    }

    // Fallback: polling loop
    tracing::info!("Starting polling loop (every {POLL_INTERVAL_SECS}s)");
    polling_loop(last_ip).await;
}

/// Event-driven watcher using `wevtutil` subscription to Hyper-V VmSwitch events.
///
/// Subscribes to: `Microsoft-Windows-Hyper-V-VmSwitch-Operational` Event ID 102
/// which fires when the WSL virtual network adapter changes (IP reassignment).
async fn event_driven_loop(last_ip: &mut String) -> anyhow::Result<()> {
    // Use wevtutil to subscribe to the event log in real-time
    // This streams XML events to stdout as they occur
    let mut child = tokio::process::Command::new("wevtutil")
        .args([
            "subscribe",
            "Microsoft-Windows-Hyper-V-VmSwitch-Operational",
            "/format:text",
            "/query:*[System[EventID=102]]",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    tracing::info!("Hyper-V event subscription active (wevtutil pid: {})", child.id().unwrap_or(0));

    let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("No stdout from wevtutil"))?;
    let mut reader = BufReader::new(stdout).lines();

    // Read lines from the event stream — each event produces multiple lines.
    // We just need to detect that *any* event occurred and debounce.
    let mut debounce_timer: Option<tokio::time::Instant> = None;

    loop {
        tokio::select! {
            line = reader.next_line() => {
                match line {
                    Ok(Some(_)) => {
                        // Event received — start/reset debounce timer
                        debounce_timer = Some(tokio::time::Instant::now());
                    }
                    Ok(None) => {
                        // wevtutil exited
                        anyhow::bail!("wevtutil subscription ended");
                    }
                    Err(e) => {
                        anyhow::bail!("wevtutil read error: {e}");
                    }
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
                // Check if we have a pending debounced event (events settled for >3s)
                if let Some(last_event) = debounce_timer {
                    if last_event.elapsed() >= std::time::Duration::from_secs(3) {
                        debounce_timer = None;
                        handle_ip_change(last_ip).await;
                    }
                }
            }
        }
    }
}

/// Polling fallback loop — checks WSL IP every 30 seconds.
async fn polling_loop(last_ip: &mut String) {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;

        match detect_wsl_ip() {
            Ok(ip) if ip != *last_ip => {
                tracing::info!("WSL IP changed (poll): {} → {}", last_ip, ip);
                handle_ip_change(last_ip).await;
            }
            Ok(_) => {}
            Err(e) => {
                tracing::warn!("Failed to detect WSL IP: {e}");
            }
        }
    }
}

/// Handle a detected IP change: settle, confirm, reset, reapply, notify.
async fn handle_ip_change(last_ip: &mut String) {
    // Wait for IP to settle
    tokio::time::sleep(std::time::Duration::from_secs(IP_SETTLE_DELAY_SECS)).await;

    // Detect and confirm the new IP
    let confirmed_ip = match detect_wsl_ip() {
        Ok(ip) => ip,
        Err(e) => {
            tracing::error!("Failed to detect WSL IP after settle: {e}");
            return;
        }
    };

    if confirmed_ip == *last_ip {
        tracing::debug!("IP unchanged after settle, skipping reapply");
        return;
    }

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

/// Send a Windows toast notification about the IP change.
fn send_toast_notification(new_ip: &str, rule_count: usize) {
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
