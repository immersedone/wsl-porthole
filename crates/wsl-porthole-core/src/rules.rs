//! Rule model and variable resolution.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Direction { WinToWsl, WslToWin }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PortSpec { Single(u16), Range(u16, u16) }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Source { Manual, Docker, Mcp, Imported }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id:           String,
    pub name:         String,
    pub direction:    Direction,
    pub listen_addr:  String,
    pub listen_port:  PortSpec,
    pub connect_port: PortSpec,
    pub connect_addr: String,   // e.g. "${WSL_IP}" or "${HOST_GW}"
    pub distro:       Option<String>,
    pub lan:          bool,
    pub enabled:      bool,
    pub source:       Source,
    pub note:         Option<String>,
}

pub fn resolve_addr(template: &str, wsl_ip: &str, host_gw: &str) -> String {
    template.replace("${WSL_IP}", wsl_ip).replace("${HOST_GW}", host_gw)
}

pub fn expand_ports(spec: &PortSpec) -> Vec<u16> {
    match spec {
        PortSpec::Single(p)   => vec![*p],
        PortSpec::Range(s, e) => (*s..=*e).collect(),
    }
}
