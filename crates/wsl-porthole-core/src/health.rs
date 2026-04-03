//! TCP health checks for port rules.
//!
//! Probes each rule's listen port with a TCP connect attempt
//! to determine reachability (green/amber/red status).

use crate::rules::{expand_ports, Rule};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(3);

/// Health status for a single rule.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum HealthStatus {
    /// All ports reachable.
    Ok,
    /// Some ports reachable, some not (range rules).
    Warn,
    /// No ports reachable.
    Error,
    /// Not checked yet or rule is disabled.
    Unknown,
}

/// Result of a health check on a single rule.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResult {
    pub rule_id: String,
    pub status: HealthStatus,
    pub reachable: usize,
    pub total: usize,
    /// Ports that failed the check.
    pub failed_ports: Vec<u16>,
}

/// Check health of a single rule by TCP-connecting to its listen ports.
pub fn check_rule(rule: &Rule) -> HealthResult {
    if !rule.enabled {
        return HealthResult {
            rule_id: rule.id.clone(),
            status: HealthStatus::Unknown,
            reachable: 0,
            total: 0,
            failed_ports: vec![],
        };
    }

    let ports = expand_ports(&rule.listen_port);
    let total = ports.len();
    let mut reachable = 0;
    let mut failed_ports = Vec::new();

    for port in &ports {
        let addr: SocketAddr = format!("{}:{}", rule.listen_addr, port)
            .parse()
            .unwrap_or_else(|_| SocketAddr::from(([127, 0, 0, 1], *port)));

        match TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT) {
            Ok(_) => reachable += 1,
            Err(_) => failed_ports.push(*port),
        }
    }

    let status = if reachable == total {
        HealthStatus::Ok
    } else if reachable > 0 {
        HealthStatus::Warn
    } else {
        HealthStatus::Error
    };

    HealthResult {
        rule_id: rule.id.clone(),
        status,
        reachable,
        total,
        failed_ports,
    }
}

/// Check health of all rules. Returns results in the same order.
pub fn check_all(rules: &[Rule]) -> Vec<HealthResult> {
    rules.iter().map(check_rule).collect()
}

/// Check health of all rules concurrently using tokio.
pub async fn check_all_async(rules: &[Rule]) -> Vec<HealthResult> {
    let mut handles = Vec::new();

    for rule in rules {
        let rule = rule.clone();
        handles.push(tokio::task::spawn_blocking(move || check_rule(&rule)));
    }

    let mut results = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(result) => results.push(result),
            Err(_) => results.push(HealthResult {
                rule_id: String::new(),
                status: HealthStatus::Unknown,
                reachable: 0,
                total: 0,
                failed_ports: vec![],
            }),
        }
    }

    results
}
