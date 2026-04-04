//! Windows Defender Firewall rule management via PowerShell.
//!
//! Creates/removes inbound firewall rules to complement netsh portproxy rules.
//! Uses `New-NetFirewallRule` / `Remove-NetFirewallRule` PowerShell cmdlets.

use crate::rules::{expand_ports, Rule};
use anyhow::Result;

const RULE_PREFIX: &str = "WSL PortHole";

/// Create an inbound Windows Firewall rule allowing TCP traffic for a rule's ports.
///
/// The firewall rule is named `"WSL PortHole: {rule_name}"` for easy identification.
pub fn add_inbound_rule(rule: &Rule) -> Result<()> {
    let display_name = format!("{RULE_PREFIX}: {}", rule.name);
    let ports = expand_ports(&rule.listen_port);
    let port_list = ports
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(",");

    // Remove existing rule with same name first (idempotent)
    let _ = remove_firewall_rule_by_name(&display_name);

    let status = crate::sys_path::command(crate::sys_path::powershell())
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "New-NetFirewallRule -DisplayName '{display_name}' -Direction Inbound -Action Allow -Protocol TCP -LocalPort {port_list} -Profile Any | Out-Null"
            ),
        ])
        .status()?;

    if !status.success() {
        anyhow::bail!("Failed to create firewall rule: {display_name}");
    }
    Ok(())
}

/// Remove the firewall rule associated with a porthole rule.
pub fn remove_rule(rule: &Rule) -> Result<()> {
    let display_name = format!("{RULE_PREFIX}: {}", rule.name);
    remove_firewall_rule_by_name(&display_name)
}

/// Create a firewall rule on the vEthernet (WSL) interface for WSL→Windows routing.
///
/// This allows traffic from WSL to reach Windows services through the Hyper-V gateway.
pub fn add_wsl_interface_rule(rule: &Rule) -> Result<()> {
    let display_name = format!("{RULE_PREFIX} WSL→WIN: {}", rule.name);
    let ports = expand_ports(&rule.connect_port);
    let port_list = ports
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(",");

    let _ = remove_firewall_rule_by_name(&display_name);

    let status = crate::sys_path::command(crate::sys_path::powershell())
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "New-NetFirewallRule -DisplayName '{display_name}' -Direction Inbound -Action Allow -Protocol TCP -LocalPort {port_list} -InterfaceAlias 'vEthernet (WSL)' -Profile Any | Out-Null"
            ),
        ])
        .status()?;

    if !status.success() {
        anyhow::bail!("Failed to create WSL interface firewall rule: {display_name}");
    }
    Ok(())
}

/// Remove all WSL PortHole firewall rules (used during reset/cleanup).
pub fn remove_all_rules() -> Result<()> {
    let status = crate::sys_path::command(crate::sys_path::powershell())
        .args([
            "-NoProfile",
            "-Command",
            &format!("Remove-NetFirewallRule -DisplayName '{RULE_PREFIX}*' -ErrorAction SilentlyContinue"),
        ])
        .status()?;

    if !status.success() {
        tracing::warn!("Some firewall rules may not have been removed");
    }
    Ok(())
}

/// List display names of all WSL PortHole firewall rules.
pub fn list_rules() -> Result<Vec<String>> {
    let output = crate::sys_path::command(crate::sys_path::powershell())
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "(Get-NetFirewallRule -DisplayName '{RULE_PREFIX}*' -ErrorAction SilentlyContinue).DisplayName"
            ),
        ])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let names: Vec<String> = stdout
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    Ok(names)
}

fn remove_firewall_rule_by_name(display_name: &str) -> Result<()> {
    let status = crate::sys_path::command(crate::sys_path::powershell())
        .args([
            "-NoProfile",
            "-Command",
            &format!("Remove-NetFirewallRule -DisplayName '{display_name}' -ErrorAction SilentlyContinue"),
        ])
        .status()?;

    if !status.success() {
        anyhow::bail!("Failed to remove firewall rule: {display_name}");
    }
    Ok(())
}
