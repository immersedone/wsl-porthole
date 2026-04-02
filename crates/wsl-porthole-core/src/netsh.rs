//! netsh portproxy CRUD wrappers.

use crate::rules::{Direction, Rule, expand_ports, resolve_addr};
use anyhow::Result;
use std::process::Command;

pub fn apply_rule(rule: &Rule, wsl_ip: &str, host_gw: &str) -> Result<()> {
    if !rule.enabled || rule.direction != Direction::WinToWsl { return Ok(()); }
    let connect_addr  = resolve_addr(&rule.connect_addr, wsl_ip, host_gw);
    let listen_ports  = expand_ports(&rule.listen_port);
    let connect_ports = expand_ports(&rule.connect_port);

    for (lp, cp) in listen_ports.iter().zip(connect_ports.iter()) {
        let ok = Command::new("netsh").args([
            "interface", "portproxy", "add", "v4tov4",
            &format!("listenport={lp}"),
            &format!("listenaddress={}", rule.listen_addr),
            &format!("connectport={cp}"),
            &format!("connectaddress={connect_addr}"),
        ]).status()?.success();
        if !ok { anyhow::bail!("netsh failed for port {lp}"); }
    }
    Ok(())
}

pub fn remove_listen_port(port: u16, addr: &str) -> Result<()> {
    Command::new("netsh").args([
        "interface", "portproxy", "delete", "v4tov4",
        &format!("listenport={port}"),
        &format!("listenaddress={addr}"),
    ]).status()?;
    Ok(())
}

pub fn reset_all() -> Result<()> {
    Command::new("netsh").args(["interface", "portproxy", "reset"]).status()?;
    Ok(())
}
