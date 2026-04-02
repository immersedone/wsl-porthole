//! WSL IP detection — runs `wsl hostname -I`, returns first IPv4.

use anyhow::{anyhow, Result};
use std::process::Command;

pub fn detect_wsl_ip() -> Result<String> { detect_wsl_ip_for(None) }

pub fn detect_wsl_ip_for(distro: Option<&str>) -> Result<String> {
    let mut cmd = Command::new("wsl");
    if let Some(d) = distro { cmd.args(["-d", d]); }
    cmd.args(["hostname", "-I"]);
    let output = cmd.output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let ip = stdout.split_whitespace()
        .find(|s| is_ipv4(s))
        .ok_or_else(|| anyhow!("No IPv4 found: {stdout}"))?
        .to_string();
    Ok(ip)
}

fn is_ipv4(s: &str) -> bool {
    s.split('.').count() == 4 && s.split('.').all(|p| p.parse::<u8>().is_ok())
}
