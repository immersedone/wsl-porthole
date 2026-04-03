//! Rule model, port specs, and variable resolution.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Direction {
    WinToWsl,
    WslToWin,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PortSpec {
    Single(u16),
    Range(u16, u16),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Source {
    Manual,
    Docker,
    Mcp,
    Imported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub direction: Direction,
    pub listen_addr: String,
    pub listen_port: PortSpec,
    pub connect_port: PortSpec,
    /// Template string, e.g. `"${WSL_IP}"`, `"${WSL_IP:Ubuntu-24.04}"`, `"${HOST_GW}"`
    pub connect_addr: String,
    pub distro: Option<String>,
    pub lan: bool,
    pub enabled: bool,
    pub source: Source,
    pub note: Option<String>,
}

impl Rule {
    pub fn new(name: impl Into<String>, direction: Direction, listen_port: u16, connect_port: u16) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            direction,
            listen_addr: "0.0.0.0".into(),
            listen_port: PortSpec::Single(listen_port),
            connect_port: PortSpec::Single(connect_port),
            connect_addr: "${WSL_IP}".into(),
            distro: None,
            lan: true,
            enabled: true,
            source: Source::Manual,
            note: None,
        }
    }

    /// Returns true if listen and connect ports differ (remapped rule).
    pub fn is_remapped(&self) -> bool {
        self.listen_port != self.connect_port
    }

    /// Returns true if this is a port range rule.
    pub fn is_range(&self) -> bool {
        matches!(self.listen_port, PortSpec::Range(_, _))
    }
}

/// Context for resolving address template variables.
pub struct ResolveContext<'a> {
    pub wsl_ip: &'a str,
    pub host_ip: &'a str,
    pub host_gw: &'a str,
    pub distro_name: &'a str,
    /// Per-distro IPs: (distro_name, ip)
    pub distro_ips: &'a [(String, String)],
}

/// Resolve address template variables in a connect_addr string.
///
/// Supported variables:
/// - `${WSL_IP}` — default WSL distro IP
/// - `${WSL_IP:DistroName}` — IP of a specific distro
/// - `${HOST_IP}` — Windows host LAN IP
/// - `${HOST_GW}` — WSL→Windows gateway IP
/// - `${DISTRO_NAME}` — active distro name
pub fn resolve_addr(template: &str, ctx: &ResolveContext<'_>) -> String {
    let mut result = template.to_string();

    // Handle ${WSL_IP:DistroName} patterns first (before plain ${WSL_IP})
    while let Some(start) = result.find("${WSL_IP:") {
        if let Some(end) = result[start..].find('}') {
            let full = &result[start..start + end + 1];
            let distro = &result[start + 9..start + end];
            let ip = ctx
                .distro_ips
                .iter()
                .find(|(name, _)| name == distro)
                .map(|(_, ip)| ip.as_str())
                .unwrap_or(ctx.wsl_ip);
            result = result.replace(full, ip);
        } else {
            break;
        }
    }

    result = result.replace("${WSL_IP}", ctx.wsl_ip);
    result = result.replace("${HOST_IP}", ctx.host_ip);
    result = result.replace("${HOST_GW}", ctx.host_gw);
    result = result.replace("${DISTRO_NAME}", ctx.distro_name);

    result
}

/// Simple two-variable resolve for backward compat with netsh module.
pub fn resolve_addr_simple(template: &str, wsl_ip: &str, host_gw: &str) -> String {
    template
        .replace("${WSL_IP}", wsl_ip)
        .replace("${HOST_GW}", host_gw)
}

/// Expand a PortSpec into individual port numbers.
pub fn expand_ports(spec: &PortSpec) -> Vec<u16> {
    match spec {
        PortSpec::Single(p) => vec![*p],
        PortSpec::Range(s, e) => (*s..=*e).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_single() {
        assert_eq!(expand_ports(&PortSpec::Single(8080)), vec![8080]);
    }

    #[test]
    fn test_expand_range() {
        let ports = expand_ports(&PortSpec::Range(1024, 1028));
        assert_eq!(ports, vec![1024, 1025, 1026, 1027, 1028]);
    }

    #[test]
    fn test_resolve_addr_simple() {
        assert_eq!(
            resolve_addr_simple("${WSL_IP}", "172.22.1.1", "172.22.0.1"),
            "172.22.1.1"
        );
        assert_eq!(
            resolve_addr_simple("${HOST_GW}", "172.22.1.1", "172.22.0.1"),
            "172.22.0.1"
        );
    }

    #[test]
    fn test_resolve_addr_full_context() {
        let ctx = ResolveContext {
            wsl_ip: "172.22.1.1",
            host_ip: "192.168.1.42",
            host_gw: "172.22.0.1",
            distro_name: "Ubuntu-24.04",
            distro_ips: &[("Debian".into(), "172.22.2.2".into())],
        };
        assert_eq!(resolve_addr("${WSL_IP}", &ctx), "172.22.1.1");
        assert_eq!(resolve_addr("${HOST_IP}", &ctx), "192.168.1.42");
        assert_eq!(resolve_addr("${HOST_GW}", &ctx), "172.22.0.1");
        assert_eq!(resolve_addr("${DISTRO_NAME}", &ctx), "Ubuntu-24.04");
        assert_eq!(resolve_addr("${WSL_IP:Debian}", &ctx), "172.22.2.2");
        // Unknown distro falls back to default WSL IP
        assert_eq!(resolve_addr("${WSL_IP:Unknown}", &ctx), "172.22.1.1");
    }

    #[test]
    fn test_rule_new() {
        let rule = Rule::new("HTTP", Direction::WinToWsl, 80, 80);
        assert_eq!(rule.name, "HTTP");
        assert!(!rule.id.is_empty());
        assert!(rule.enabled);
        assert!(rule.lan);
        assert!(!rule.is_remapped());
        assert!(!rule.is_range());
    }

    #[test]
    fn test_rule_remapped() {
        let mut rule = Rule::new("HTTP alt", Direction::WinToWsl, 8080, 80);
        rule.listen_port = PortSpec::Single(8080);
        rule.connect_port = PortSpec::Single(80);
        assert!(rule.is_remapped());
    }

    #[test]
    fn test_portspec_serde_roundtrip() {
        let single = PortSpec::Single(8080);
        let json = serde_json::to_string(&single).unwrap();
        assert_eq!(json, "8080");

        let range = PortSpec::Range(1024, 1048);
        let json = serde_json::to_string(&range).unwrap();
        assert_eq!(json, "[1024,1048]");

        let back: PortSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(back, PortSpec::Range(1024, 1048));
    }
}
