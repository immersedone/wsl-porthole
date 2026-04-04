//! WSL / host IP detection utilities.
//!
//! - `detect_wsl_ip()` / `detect_wsl_ip_for(distro)` — WSL2 guest IP via `wsl hostname -I`
//! - `detect_host_ip()` — Windows host LAN IP (first non-loopback IPv4)
//! - `detect_host_gateway()` — WSL→Windows gateway IP via `/etc/resolv.conf`

use anyhow::{anyhow, Result};

use crate::sys_path;

/// Detect the default WSL distro's IPv4 address.
pub fn detect_wsl_ip() -> Result<String> {
    detect_wsl_ip_for(None)
}

/// Detect a specific WSL distro's IPv4 address (or default if `None`).
pub fn detect_wsl_ip_for(distro: Option<&str>) -> Result<String> {
    let mut cmd = sys_path::command(sys_path::wsl());
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
    let ps_output = sys_path::command(sys_path::powershell())
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
    let output = sys_path::command(sys_path::ipconfig()).output()?;
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
    let output = sys_path::command(sys_path::wsl())
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

/// Info about an installed WSL distribution.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DistroInfo {
    pub name: String,
    pub state: String,
    pub version: u32,
    pub default: bool,
    pub ip: Option<String>,
}

/// List installed WSL distributions by parsing `wsl.exe -l -v`.
pub fn list_distros() -> Result<Vec<DistroInfo>> {
    let output = sys_path::command(sys_path::wsl())
        .args(["-l", "-v"])
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("wsl -l -v failed: {stderr}");
    }

    // wsl.exe outputs UTF-16LE on Windows
    let stdout = decode_wsl_output(&output.stdout);
    let mut distros = Vec::new();

    for line in stdout.lines().skip(1) {
        // Skip empty lines
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Lines look like: "* Ubuntu-24.04    Running    2"
        // or:               "  Debian          Stopped    2"
        let is_default = line.starts_with('*');
        let line = line.trim_start_matches('*').trim();

        // Split into parts — name, state, version
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }

        let name = parts[0].to_string();
        let state = parts[1].to_string();
        let version: u32 = parts[2].parse().unwrap_or(2);

        // Try to get IP for running distros
        let ip = if state == "Running" {
            detect_wsl_ip_for(Some(&name)).ok()
        } else {
            None
        };

        distros.push(DistroInfo {
            name,
            state,
            version,
            default: is_default,
            ip,
        });
    }

    Ok(distros)
}

/// Decode wsl.exe output which is UTF-16LE on Windows.
fn decode_wsl_output(bytes: &[u8]) -> String {
    // Try UTF-16LE first (Windows wsl.exe output)
    if bytes.len() >= 2 {
        // Check for BOM or try to decode as UTF-16LE
        let u16_iter: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        if let Ok(s) = String::from_utf16(&u16_iter) {
            // Strip BOM if present
            return s.trim_start_matches('\u{feff}').to_string();
        }
    }
    // Fallback to UTF-8
    String::from_utf8_lossy(bytes).to_string()
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
