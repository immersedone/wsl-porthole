//! netsh portproxy CRUD wrappers.
//!
//! All functions shell out to `netsh.exe` and require admin privileges.

use crate::rules::{Direction, Rule, expand_ports, resolve_addr_simple};
use anyhow::Result;

/// A single active portproxy entry as reported by `netsh`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveProxy {
    pub listen_addr: String,
    pub listen_port: u16,
    pub connect_addr: String,
    pub connect_port: u16,
}

/// Apply a rule by creating netsh portproxy entries for all expanded ports.
///
/// Only applies WinToWsl rules that are enabled.
pub fn apply_rule(rule: &Rule, wsl_ip: &str, host_gw: &str) -> Result<()> {
    if !rule.enabled || rule.direction != Direction::WinToWsl {
        return Ok(());
    }
    let connect_addr = resolve_addr_simple(&rule.connect_addr, wsl_ip, host_gw);
    let listen_ports = expand_ports(&rule.listen_port);
    let connect_ports = expand_ports(&rule.connect_port);

    for (lp, cp) in listen_ports.iter().zip(connect_ports.iter()) {
        let status = crate::sys_path::command(crate::sys_path::netsh())
            .args([
                "interface",
                "portproxy",
                "add",
                "v4tov4",
                &format!("listenport={lp}"),
                &format!("listenaddress={}", rule.listen_addr),
                &format!("connectport={cp}"),
                &format!("connectaddress={connect_addr}"),
            ])
            .status()?;
        if !status.success() {
            anyhow::bail!("netsh add failed for port {lp}");
        }
    }
    Ok(())
}

/// Remove all portproxy entries for a rule's expanded listen ports.
pub fn remove_rule(rule: &Rule) -> Result<()> {
    let listen_ports = expand_ports(&rule.listen_port);
    for lp in &listen_ports {
        remove_listen_port(*lp, &rule.listen_addr)?;
    }
    Ok(())
}

/// Remove a single portproxy entry by listen port and address.
pub fn remove_listen_port(port: u16, addr: &str) -> Result<()> {
    crate::sys_path::command(crate::sys_path::netsh())
        .args([
            "interface",
            "portproxy",
            "delete",
            "v4tov4",
            &format!("listenport={port}"),
            &format!("listenaddress={addr}"),
        ])
        .status()?;
    Ok(())
}

/// Remove all portproxy rules.
pub fn reset_all() -> Result<()> {
    crate::sys_path::command(crate::sys_path::netsh())
        .args(["interface", "portproxy", "reset"])
        .status()?;
    Ok(())
}

/// List all active v4tov4 portproxy rules from netsh.
///
/// Parses the output of `netsh interface portproxy show v4tov4`.
pub fn list_active() -> Result<Vec<ActiveProxy>> {
    let output = crate::sys_path::command(crate::sys_path::netsh())
        .args(["interface", "portproxy", "show", "v4tov4"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("netsh show failed: {stderr}");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_netsh_show_output(&stdout)
}

/// Parse the tabular output from `netsh interface portproxy show v4tov4`.
///
/// Example output:
/// ```text
/// Listen on ipv4:             Connect to ipv4:
///
/// Address         Port        Address         Port
/// --------------- ----------  --------------- ----------
/// 0.0.0.0         80          172.22.207.71   80
/// 0.0.0.0         443         172.22.207.71   443
/// ```
fn parse_netsh_show_output(output: &str) -> Result<Vec<ActiveProxy>> {
    let mut proxies = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        // Skip headers, separators, and empty lines
        if line.is_empty()
            || line.starts_with("Listen")
            || line.starts_with("Address")
            || line.starts_with('-')
        {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let listen_port = parts[1].parse::<u16>();
            let connect_port = parts[3].parse::<u16>();

            if let (Ok(lp), Ok(cp)) = (listen_port, connect_port) {
                proxies.push(ActiveProxy {
                    listen_addr: parts[0].to_string(),
                    listen_port: lp,
                    connect_addr: parts[2].to_string(),
                    connect_port: cp,
                });
            }
        }
    }

    Ok(proxies)
}

/// Generate the netsh command string for a rule (for display/preview).
pub fn preview_command(rule: &Rule, wsl_ip: &str, host_gw: &str) -> Vec<String> {
    if rule.direction != Direction::WinToWsl {
        return vec![];
    }
    let connect_addr = resolve_addr_simple(&rule.connect_addr, wsl_ip, host_gw);
    let listen_ports = expand_ports(&rule.listen_port);
    let connect_ports = expand_ports(&rule.connect_port);

    listen_ports
        .iter()
        .zip(connect_ports.iter())
        .map(|(lp, cp)| {
            format!(
                "netsh interface portproxy add v4tov4 listenport={lp} listenaddress={} connectport={cp} connectaddress={connect_addr}",
                rule.listen_addr
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_netsh_show_output() {
        let output = r#"
Listen on ipv4:             Connect to ipv4:

Address         Port        Address         Port
--------------- ----------  --------------- ----------
0.0.0.0         80          172.22.207.71   80
0.0.0.0         443         172.22.207.71   443
0.0.0.0         8080        172.22.207.71   80
"#;
        let proxies = parse_netsh_show_output(output).unwrap();
        assert_eq!(proxies.len(), 3);

        assert_eq!(proxies[0].listen_addr, "0.0.0.0");
        assert_eq!(proxies[0].listen_port, 80);
        assert_eq!(proxies[0].connect_addr, "172.22.207.71");
        assert_eq!(proxies[0].connect_port, 80);

        assert_eq!(proxies[2].listen_port, 8080);
        assert_eq!(proxies[2].connect_port, 80);
    }

    #[test]
    fn test_parse_empty_output() {
        let output = r#"
Listen on ipv4:             Connect to ipv4:

Address         Port        Address         Port
--------------- ----------  --------------- ----------
"#;
        let proxies = parse_netsh_show_output(output).unwrap();
        assert!(proxies.is_empty());
    }

    #[test]
    fn test_preview_command() {
        let rule = Rule {
            id: "test".into(),
            name: "HTTP".into(),
            direction: Direction::WinToWsl,
            listen_addr: "0.0.0.0".into(),
            listen_port: crate::rules::PortSpec::Single(80),
            connect_port: crate::rules::PortSpec::Single(80),
            connect_addr: "${WSL_IP}".into(),
            distro: None,
            lan: true,
            enabled: true,
            source: crate::rules::Source::Manual,
            note: None,
        };
        let cmds = preview_command(&rule, "172.22.1.1", "172.22.0.1");
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("connectaddress=172.22.1.1"));
        assert!(cmds[0].contains("listenport=80"));
    }

    #[test]
    fn test_preview_range() {
        let rule = Rule {
            id: "test".into(),
            name: "Range".into(),
            direction: Direction::WinToWsl,
            listen_addr: "0.0.0.0".into(),
            listen_port: crate::rules::PortSpec::Range(1024, 1026),
            connect_port: crate::rules::PortSpec::Range(1024, 1026),
            connect_addr: "${WSL_IP}".into(),
            distro: None,
            lan: true,
            enabled: true,
            source: crate::rules::Source::Manual,
            note: None,
        };
        let cmds = preview_command(&rule, "172.22.1.1", "172.22.0.1");
        assert_eq!(cmds.len(), 3);
    }
}
