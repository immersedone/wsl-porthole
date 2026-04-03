//! WSL / host IP detection utilities.
//!
//! - `detect_wsl_ip()` / `detect_wsl_ip_for(distro)` — WSL2 guest IP via `wsl hostname -I`
//! - `detect_host_ip()` — Windows host LAN IP (first non-loopback IPv4)
//! - `detect_host_gateway()` — WSL→Windows gateway IP via `/etc/resolv.conf`

use anyhow::{anyhow, Result};
use std::process::Command;

/// Detect the default WSL distro's IPv4 address.
pub fn detect_wsl_ip() -> Result<String> {
    detect_wsl_ip_for(None)
}

/// Detect a specific WSL distro's IPv4 address (or default if `None`).
pub fn detect_wsl_ip_for(distro: Option<&str>) -> Result<String> {
    let mut cmd = Command::new("wsl");
    if let Some(d) = distro {
        cmd.args(["-d", d]);
    }
    cmd.args(["hostname", "-I"]);
    let output = cmd.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("wsl hostname -I failed: {stderr}");
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let ip = stdout
        .split_whitespace()
        .find(|s| is_ipv4(s))
        .ok_or_else(|| anyhow!("No IPv4 found in wsl output: {stdout}"))?
        .to_string();
    Ok(ip)
}

/// Detect the Windows host's LAN IPv4 address.
///
/// Runs `powershell` to query the first non-loopback, "Up" IPv4 address.
/// Falls back to parsing `ipconfig` output if PowerShell is unavailable.
pub fn detect_host_ip() -> Result<String> {
    // Try PowerShell first — most reliable
    let ps_output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "(Get-NetIPAddress -AddressFamily IPv4 | Where-Object { $_.InterfaceAlias -notlike '*Loopback*' -and $_.InterfaceAlias -notlike '*WSL*' -and $_.PrefixOrigin -ne 'WellKnown' } | Select-Object -First 1).IPAddress",
        ])
        .output();

    if let Ok(output) = ps_output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let ip = stdout.trim();
        if is_ipv4(ip) {
            return Ok(ip.to_string());
        }
    }

    // Fallback: parse ipconfig
    let output = Command::new("ipconfig").output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("IPv4 Address") || line.starts_with("IPv4") {
            if let Some(ip) = line.split(':').nth(1) {
                let ip = ip.trim();
                if is_ipv4(ip) && !ip.starts_with("127.") && !ip.starts_with("172.") {
                    return Ok(ip.to_string());
                }
            }
        }
    }

    Err(anyhow!("Could not detect host LAN IP"))
}

/// Detect the WSL→Windows gateway IP (the Hyper-V virtual adapter address).
///
/// Reads from WSL's `/etc/resolv.conf` nameserver entry, which is
/// typically the Windows host gateway address.
pub fn detect_host_gateway() -> Result<String> {
    let output = Command::new("wsl")
        .args(["cat", "/etc/resolv.conf"])
        .output()?;
    if !output.status.success() {
        anyhow::bail!("Failed to read /etc/resolv.conf from WSL");
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("nameserver") {
            let ip = rest.trim();
            if is_ipv4(ip) {
                return Ok(ip.to_string());
            }
        }
    }
    Err(anyhow!("No nameserver IPv4 found in /etc/resolv.conf"))
}

fn is_ipv4(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    parts.len() == 4 && parts.iter().all(|p| p.parse::<u8>().is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ipv4_valid() {
        assert!(is_ipv4("192.168.1.1"));
        assert!(is_ipv4("172.22.207.71"));
        assert!(is_ipv4("0.0.0.0"));
        assert!(is_ipv4("255.255.255.255"));
    }

    #[test]
    fn test_is_ipv4_invalid() {
        assert!(!is_ipv4("not-an-ip"));
        assert!(!is_ipv4("192.168.1"));
        assert!(!is_ipv4("192.168.1.256"));
        assert!(!is_ipv4("fe80::1"));
        assert!(!is_ipv4(""));
    }
}
