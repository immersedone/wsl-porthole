//! WSL /etc/hosts and environment variable injection.
//!
//! Writes gateway aliases into WSL's `/etc/hosts` and
//! optional env vars into `.bashrc`/`.profile`.

use anyhow::Result;
use std::process::Command;

const HOSTS_MARKER: &str = "# WSL PortHole managed entries";

/// Write a hostname → IP mapping into WSL's /etc/hosts.
///
/// Entries are grouped between marker comments for easy cleanup.
/// Example: `mcp-gw  172.22.192.1`
pub fn write_hosts_entry(hostname: &str, ip: &str, distro: Option<&str>) -> Result<()> {
    let entry = format!("{ip}  {hostname}");

    // Read current hosts file
    let mut cmd = crate::sys_path::command(crate::sys_path::wsl());
    if let Some(d) = distro {
        cmd.args(["-d", d]);
    }
    cmd.args(["cat", "/etc/hosts"]);
    let output = cmd.output()?;
    let current = String::from_utf8_lossy(&output.stdout).to_string();

    // Build new hosts content
    let mut new_lines: Vec<String> = Vec::new();
    let mut in_managed = false;
    let mut managed_entries: Vec<String> = Vec::new();

    for line in current.lines() {
        if line.trim() == HOSTS_MARKER {
            if !in_managed {
                in_managed = true;
                continue;
            } else {
                // End of managed block
                in_managed = false;
                continue;
            }
        }
        if in_managed {
            managed_entries.push(line.to_string());
        } else {
            new_lines.push(line.to_string());
        }
    }

    // Update or add the entry
    managed_entries.retain(|l| {
        !l.split_whitespace()
            .nth(1)
            .map(|h| h == hostname)
            .unwrap_or(false)
    });
    managed_entries.push(entry);

    // Append managed block
    new_lines.push(String::new());
    new_lines.push(HOSTS_MARKER.to_string());
    for e in &managed_entries {
        new_lines.push(e.clone());
    }
    new_lines.push(HOSTS_MARKER.to_string());

    let new_content = new_lines.join("\n");

    // Write back via wsl
    let mut write_cmd = crate::sys_path::command(crate::sys_path::wsl());
    if let Some(d) = distro {
        write_cmd.args(["-d", d]);
    }
    write_cmd.args(["bash", "-c", &format!("echo '{}' | sudo tee /etc/hosts > /dev/null", new_content.replace('\'', "'\\''"))]);

    let status = write_cmd.status()?;
    if !status.success() {
        anyhow::bail!("Failed to write /etc/hosts");
    }

    Ok(())
}

/// Remove all WSL PortHole managed entries from /etc/hosts.
pub fn clean_hosts(distro: Option<&str>) -> Result<()> {
    let mut cmd = crate::sys_path::command(crate::sys_path::wsl());
    if let Some(d) = distro {
        cmd.args(["-d", d]);
    }
    cmd.args(["bash", "-c", &format!(
        "sed -i '/{marker}/,/{marker}/d' /etc/hosts",
        marker = HOSTS_MARKER.replace('#', "\\#")
    )]);
    cmd.status()?;
    Ok(())
}

/// Inject an environment variable into WSL's .bashrc.
///
/// Adds a line like: `export MCP_GW=172.22.192.1  # WSL PortHole`
pub fn inject_env_var(name: &str, value: &str, distro: Option<&str>) -> Result<()> {
    let marker = "# WSL PortHole";
    let export_line = format!("export {name}={value}  {marker}");

    let mut cmd = crate::sys_path::command(crate::sys_path::wsl());
    if let Some(d) = distro {
        cmd.args(["-d", d]);
    }
    cmd.args(["bash", "-c", &format!(
        // Remove old entry, then append new one
        "sed -i '/{marker}/d' ~/.bashrc && echo '{line}' >> ~/.bashrc",
        marker = format!("{name}=.*{marker}").replace('/', "\\/"),
        line = export_line.replace('\'', "'\\''")
    )]);

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("Failed to inject env var {name} into .bashrc");
    }

    Ok(())
}

/// Remove all WSL PortHole env vars from .bashrc.
pub fn clean_env_vars(distro: Option<&str>) -> Result<()> {
    let marker = "# WSL PortHole";
    let mut cmd = crate::sys_path::command(crate::sys_path::wsl());
    if let Some(d) = distro {
        cmd.args(["-d", d]);
    }
    cmd.args(["bash", "-c", &format!(
        "sed -i '/{}/d' ~/.bashrc",
        marker.replace('/', "\\/")
    )]);
    cmd.status()?;
    Ok(())
}
