//! Port conflict detection — scans Windows TCP listeners
//! to find ports already in use before applying rules.

use crate::rules::{expand_ports, Rule};
use anyhow::Result;
use std::process::Command;

/// A detected port conflict.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Conflict {
    pub rule_id: String,
    pub rule_name: String,
    pub port: u16,
    pub owning_pid: u32,
    pub owning_process: String,
}

/// Represents an active TCP listener parsed from netstat output.
#[derive(Debug, Clone)]
struct TcpListener {
    addr: String,
    port: u16,
    pid: u32,
}

/// Scan for conflicts between rules and existing Windows TCP listeners.
///
/// Runs `netstat -anop TCP` and cross-references with rule listen ports.
pub fn detect_conflicts(rules: &[Rule]) -> Result<Vec<Conflict>> {
    let listeners = get_tcp_listeners()?;
    let mut conflicts = Vec::new();

    for rule in rules {
        if !rule.enabled {
            continue;
        }
        let ports = expand_ports(&rule.listen_port);
        for port in &ports {
            // Check if any listener is on this port with a matching address
            for listener in &listeners {
                let port_match = listener.port == *port;
                let addr_match = listener.addr == "0.0.0.0"
                    || listener.addr == rule.listen_addr
                    || rule.listen_addr == "0.0.0.0";

                if port_match && addr_match {
                    let process_name = get_process_name(listener.pid);
                    conflicts.push(Conflict {
                        rule_id: rule.id.clone(),
                        rule_name: rule.name.clone(),
                        port: *port,
                        owning_pid: listener.pid,
                        owning_process: process_name,
                    });
                }
            }
        }
    }

    Ok(conflicts)
}

/// Parse `netstat -anop TCP` output to get TCP listeners in LISTENING state.
fn get_tcp_listeners() -> Result<Vec<TcpListener>> {
    let output = Command::new("netstat")
        .args(["-anop", "TCP"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut listeners = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if !line.contains("LISTENING") {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        // Format: TCP  addr:port  0.0.0.0:0  LISTENING  pid
        if parts.len() >= 5 {
            if let Some((addr, port_str)) = parts[1].rsplit_once(':') {
                if let Ok(port) = port_str.parse::<u16>() {
                    let pid = parts[4].parse::<u32>().unwrap_or(0);
                    listeners.push(TcpListener {
                        addr: addr.to_string(),
                        port,
                        pid,
                    });
                }
            }
        }
    }

    Ok(listeners)
}

/// Get the process name for a PID via tasklist.
fn get_process_name(pid: u32) -> String {
    if pid == 0 {
        return "System".to_string();
    }

    let output = Command::new("tasklist")
        .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // CSV format: "process.exe","pid","session","#","mem"
            stdout
                .lines()
                .next()
                .and_then(|line| line.split(',').next())
                .map(|s| s.trim_matches('"').to_string())
                .unwrap_or_else(|| format!("PID {pid}"))
        }
        Err(_) => format!("PID {pid}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_netstat_line() {
        // This test validates the parsing logic with a mock line
        let line = "  TCP    0.0.0.0:5432           0.0.0.0:0              LISTENING       1234";
        let parts: Vec<&str> = line.trim().split_whitespace().collect();
        assert_eq!(parts.len(), 5);
        let (addr, port_str) = parts[1].rsplit_once(':').unwrap();
        assert_eq!(addr, "0.0.0.0");
        assert_eq!(port_str, "5432");
        assert_eq!(parts[4].parse::<u32>().unwrap(), 1234);
    }
}
