//! IPC server for GUI ↔ Service communication.
//!
//! Uses a simple TCP listener on localhost:19836 (chosen to avoid common ports).
//! Protocol: newline-delimited JSON messages.
//!
//! When the full Tauri app is built, this could be upgraded to named pipes
//! on Windows for better security, but TCP localhost is simpler and
//! works cross-platform for development.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

const IPC_PORT: u16 = 19836;
const IPC_ADDR: &str = "127.0.0.1";

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IpcRequest {
    #[serde(rename = "status")]
    Status,
    #[serde(rename = "sync")]
    SyncNow,
    #[serde(rename = "get_ip")]
    GetIp,
    #[serde(rename = "get_rules")]
    GetRules,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IpcResponse {
    #[serde(rename = "status")]
    Status {
        running: bool,
        wsl_ip: Option<String>,
        host_ip: Option<String>,
        rule_count: usize,
        last_sync: Option<String>,
        uptime_secs: u64,
    },
    #[serde(rename = "sync_result")]
    SyncResult { success: bool, message: String },
    #[serde(rename = "ip")]
    Ip {
        wsl_ip: Option<String>,
        host_ip: Option<String>,
        host_gw: Option<String>,
    },
    #[serde(rename = "error")]
    Error { message: String },
}

/// Run the IPC TCP server. Listens on localhost only.
pub async fn run_ipc_server() -> Result<()> {
    let listener = TcpListener::bind((IPC_ADDR, IPC_PORT)).await?;
    tracing::info!("IPC server listening on {IPC_ADDR}:{IPC_PORT}");

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                tracing::debug!("IPC connection from {addr}");
                tokio::spawn(async move {
                    if let Err(e) = handle_ipc_client(stream).await {
                        tracing::debug!("IPC client error: {e}");
                    }
                });
            }
            Err(e) => {
                tracing::warn!("IPC accept error: {e}");
            }
        }
    }
}

async fn handle_ipc_client(stream: tokio::net::TcpStream) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break; // Connection closed
        }

        let response = match serde_json::from_str::<IpcRequest>(line.trim()) {
            Ok(request) => handle_request(request).await,
            Err(e) => IpcResponse::Error {
                message: format!("Invalid request: {e}"),
            },
        };

        let mut json = serde_json::to_string(&response)?;
        json.push('\n');
        writer.write_all(json.as_bytes()).await?;
    }

    Ok(())
}

async fn handle_request(request: IpcRequest) -> IpcResponse {
    use wsl_porthole_core::ip;

    match request {
        IpcRequest::Status => {
            let wsl_ip = ip::detect_wsl_ip().ok();
            let host_ip = ip::detect_host_ip().ok();
            let config_path = std::path::Path::new("wsl-porthole-rules.json");
            let rule_count = wsl_porthole_core::config::load_rules(config_path)
                .map(|c| c.rules.iter().filter(|r| r.enabled).count())
                .unwrap_or(0);

            IpcResponse::Status {
                running: true,
                wsl_ip,
                host_ip,
                rule_count,
                last_sync: None, // TODO: track last sync time
                uptime_secs: 0,  // TODO: track uptime
            }
        }
        IpcRequest::SyncNow => {
            let wsl_ip = match ip::detect_wsl_ip() {
                Ok(ip) => ip,
                Err(e) => {
                    return IpcResponse::SyncResult {
                        success: false,
                        message: format!("Failed to detect WSL IP: {e}"),
                    };
                }
            };

            match crate::watcher::apply_current_rules(&wsl_ip) {
                Ok(count) => IpcResponse::SyncResult {
                    success: true,
                    message: format!("Applied {count} rules (WSL IP: {wsl_ip})"),
                },
                Err(e) => IpcResponse::SyncResult {
                    success: false,
                    message: format!("Sync failed: {e}"),
                },
            }
        }
        IpcRequest::GetIp => IpcResponse::Ip {
            wsl_ip: ip::detect_wsl_ip().ok(),
            host_ip: ip::detect_host_ip().ok(),
            host_gw: ip::detect_host_gateway().ok(),
        },
        IpcRequest::GetRules => {
            // TODO: return full rule list
            IpcResponse::Error {
                message: "Not yet implemented".into(),
            }
        }
    }
}
