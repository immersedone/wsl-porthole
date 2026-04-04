//! Export rules as a PowerShell netsh script.

use crate::rules::{expand_ports, Direction, Rule};

/// Generate a PowerShell script that applies all enabled rules.
///
/// The script auto-detects WSL IP and applies all rules with
/// proper error handling and a summary.
pub fn export_netsh_script(rules: &[Rule]) -> String {
    let mut script = String::new();

    script.push_str("# WSL PortHole — Generated netsh portproxy script\n");
    script.push_str("# Run as Administrator\n");
    script.push_str("#Requires -RunAsAdministrator\n\n");
    script.push_str("$ErrorActionPreference = \"Stop\"\n\n");

    // Auto-detect WSL IP
    script.push_str("# Detect WSL IP\n");
    script.push_str("$wslIp = (wsl hostname -I 2>$null).Trim().Split(\" \")[0]\n");
    script.push_str("if (-not ($wslIp -match '^\\d{1,3}(\\.\\d{1,3}){3}$')) {\n");
    script.push_str("    Write-Host \"[ERROR] Could not detect WSL IP. Is WSL running?\" -ForegroundColor Red\n");
    script.push_str("    exit 1\n");
    script.push_str("}\n");
    script.push_str("Write-Host \"WSL IP: $wslIp\" -ForegroundColor Green\n\n");

    // Reset existing rules
    script.push_str("# Reset existing portproxy rules\n");
    script.push_str("netsh interface portproxy reset | Out-Null\n\n");

    // Apply each rule
    let enabled: Vec<&Rule> = rules.iter().filter(|r| r.enabled && r.direction == Direction::WinToWsl).collect();

    script.push_str(&format!("# Apply {} rules\n", enabled.len()));

    for rule in &enabled {
        let listen_ports = expand_ports(&rule.listen_port);
        let connect_ports = expand_ports(&rule.connect_port);
        let connect_addr = rule.connect_addr.replace("${WSL_IP}", "$wslIp");

        script.push_str(&format!("# {}\n", rule.name));

        for (lp, cp) in listen_ports.iter().zip(connect_ports.iter()) {
            script.push_str(&format!(
                "netsh interface portproxy add v4tov4 listenport={} listenaddress={} connectport={} connectaddress={} | Out-Null\n",
                lp, rule.listen_addr, cp, connect_addr
            ));
        }

        if let Some(note) = &rule.note {
            script.push_str(&format!("# Note: {}\n", note));
        }
        script.push('\n');
    }

    // Firewall rule
    let all_ports: Vec<u16> = enabled
        .iter()
        .flat_map(|r| expand_ports(&r.listen_port))
        .collect();

    if !all_ports.is_empty() {
        let port_list: String = all_ports
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",");

        script.push_str("# Firewall rule\n");
        script.push_str("Remove-NetFirewallRule -DisplayName \"WSL PortHole\" -ErrorAction SilentlyContinue\n");
        script.push_str(&format!(
            "New-NetFirewallRule -DisplayName \"WSL PortHole\" -Direction Inbound -Action Allow -Protocol TCP -LocalPort {} | Out-Null\n\n",
            port_list
        ));
    }

    // Summary
    script.push_str("# Summary\n");
    script.push_str("$rules = netsh interface portproxy show v4tov4\n");
    script.push_str("$count = ($rules | Select-String \"0.0.0.0\").Count\n");
    script.push_str("Write-Host \"[DONE] $count rules active. WSL IP: $wslIp\" -ForegroundColor Green\n");

    script
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{Direction, PortSpec, Rule, Source};

    #[test]
    fn test_export_basic() {
        let rules = vec![
            Rule {
                id: "1".into(),
                name: "HTTP".into(),
                direction: Direction::WinToWsl,
                listen_addr: "0.0.0.0".into(),
                listen_port: PortSpec::Single { port: 80 },
                connect_port: PortSpec::Single { port: 80 },
                connect_addr: "${WSL_IP}".into(),
                distro: None,
                lan: true,
                enabled: true,
                source: Source::Manual,
                note: None,
            },
        ];
        let script = export_netsh_script(&rules);
        assert!(script.contains("listenport=80"));
        assert!(script.contains("connectaddress=$wslIp"));
        assert!(script.contains("# HTTP"));
        assert!(script.contains("#Requires -RunAsAdministrator"));
    }

    #[test]
    fn test_export_skips_disabled() {
        let rules = vec![
            Rule {
                id: "1".into(),
                name: "Disabled".into(),
                direction: Direction::WinToWsl,
                listen_addr: "0.0.0.0".into(),
                listen_port: PortSpec::Single { port: 80 },
                connect_port: PortSpec::Single { port: 80 },
                connect_addr: "${WSL_IP}".into(),
                distro: None,
                lan: true,
                enabled: false,
                source: Source::Manual,
                note: None,
            },
        ];
        let script = export_netsh_script(&rules);
        assert!(!script.contains("listenport=80"));
        assert!(script.contains("# Apply 0 rules"));
    }
}
