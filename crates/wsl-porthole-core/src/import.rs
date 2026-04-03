//! Parse existing netsh portproxy scripts into Rule structs.
//!
//! Handles lines like:
//! ```text
//! netsh interface portproxy add v4tov4 listenport=80 listenaddress=0.0.0.0 connectport=80 connectaddress=172.22.207.71
//! ```

use crate::rules::{Direction, PortSpec, Rule, Source};
use uuid::Uuid;

/// Parse a netsh portproxy script (PowerShell or batch) and extract rules.
///
/// Scans each line for `netsh interface portproxy add v4tov4` commands
/// and extracts the port/address parameters. Hardcoded connect addresses
/// are replaced with `${WSL_IP}`.
pub fn parse_netsh_script(text: &str) -> Vec<Rule> {
    let mut rules = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") || line.starts_with("REM") {
            continue;
        }

        // Normalize backtick line continuations (PowerShell)
        // We work on the already-joined line, but also handle inline params
        if !line.contains("portproxy") || !line.contains("add") {
            continue;
        }

        let listen_port = extract_param(line, "listenport");
        let listen_addr = extract_param(line, "listenaddress");
        let connect_port = extract_param(line, "connectport");
        let connect_addr = extract_param(line, "connectaddress");

        if let (Some(lp), Some(cp)) = (listen_port, connect_port) {
            if let (Ok(lp), Ok(cp)) = (lp.parse::<u16>(), cp.parse::<u16>()) {
                let la = listen_addr.unwrap_or("0.0.0.0");
                // Replace any hardcoded private IP with the variable
                let ca = match connect_addr {
                    Some(addr) if is_private_ip(addr) => "${WSL_IP}".to_string(),
                    Some(addr) => addr.to_string(),
                    None => "${WSL_IP}".to_string(),
                };

                let name = if lp != cp {
                    format!("Imported {lp}→{cp}")
                } else {
                    format!("Imported {lp}")
                };

                rules.push(Rule {
                    id: Uuid::new_v4().to_string(),
                    name,
                    direction: Direction::WinToWsl,
                    listen_addr: la.to_string(),
                    listen_port: PortSpec::Single(lp),
                    connect_port: PortSpec::Single(cp),
                    connect_addr: ca,
                    distro: None,
                    lan: la == "0.0.0.0",
                    enabled: true,
                    source: Source::Imported,
                    note: None,
                });
            }
        }
    }

    rules
}

/// Extract a named parameter value from a netsh command line.
/// Handles both `param=value` and `param = value` formats.
fn extract_param<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let lower = line.to_lowercase();
    let name_lower = name.to_lowercase();

    // Find the parameter name (case-insensitive)
    let idx = lower.find(&name_lower)?;
    let rest = &line[idx + name.len()..];

    // Skip optional whitespace and '='
    let rest = rest.trim_start();
    let rest = rest.strip_prefix('=')?;
    let rest = rest.trim_start();

    // Take until next whitespace or end
    let end = rest.find(|c: char| c.is_whitespace()).unwrap_or(rest.len());
    let value = &rest[..end];

    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn is_private_ip(s: &str) -> bool {
    s.starts_with("10.")
        || s.starts_with("172.")
        || s.starts_with("192.168.")
        || s == "127.0.0.1"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_line() {
        let script = "netsh interface portproxy add v4tov4 listenport=80 listenaddress=0.0.0.0 connectport=80 connectaddress=172.22.207.71";
        let rules = parse_netsh_script(script);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].listen_port, PortSpec::Single(80));
        assert_eq!(rules[0].connect_port, PortSpec::Single(80));
        assert_eq!(rules[0].connect_addr, "${WSL_IP}");
        assert_eq!(rules[0].listen_addr, "0.0.0.0");
        assert!(rules[0].lan);
        assert_eq!(rules[0].source, Source::Imported);
    }

    #[test]
    fn test_parse_remapped() {
        let script = "netsh interface portproxy add v4tov4 listenport=8080 listenaddress=0.0.0.0 connectport=80 connectaddress=172.22.207.71";
        let rules = parse_netsh_script(script);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "Imported 8080→80");
        assert_eq!(rules[0].listen_port, PortSpec::Single(8080));
        assert_eq!(rules[0].connect_port, PortSpec::Single(80));
    }

    #[test]
    fn test_parse_multiple_lines() {
        let script = r#"
# Reset first
netsh interface portproxy reset
# Add rules
netsh interface portproxy add v4tov4 listenport=80 listenaddress=0.0.0.0 connectport=80 connectaddress=172.22.207.71
netsh interface portproxy add v4tov4 listenport=443 listenaddress=0.0.0.0 connectport=443 connectaddress=172.22.207.71
netsh interface portproxy add v4tov4 listenport=22 listenaddress=0.0.0.0 connectport=22 connectaddress=172.22.207.71
"#;
        let rules = parse_netsh_script(script);
        assert_eq!(rules.len(), 3);
    }

    #[test]
    fn test_skip_comments_and_empty() {
        let script = r#"
# comment
REM another comment

netsh interface portproxy add v4tov4 listenport=80 listenaddress=0.0.0.0 connectport=80 connectaddress=172.22.1.1
"#;
        let rules = parse_netsh_script(script);
        assert_eq!(rules.len(), 1);
    }

    #[test]
    fn test_extract_param() {
        let line = "netsh interface portproxy add v4tov4 listenport=80 listenaddress=0.0.0.0 connectport=80 connectaddress=172.22.1.1";
        assert_eq!(extract_param(line, "listenport"), Some("80"));
        assert_eq!(extract_param(line, "connectaddress"), Some("172.22.1.1"));
        assert_eq!(extract_param(line, "nonexistent"), None);
    }
}
