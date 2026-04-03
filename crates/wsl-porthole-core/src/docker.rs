//! Docker container discovery via the bollard API.
//!
//! Supports both:
//! - **WSL engine**: Unix socket at `/var/run/docker.sock`
//! - **Windows engine**: Named pipe at `npipe:////./pipe/docker_engine`

use anyhow::Result;
use bollard::container::ListContainersOptions;
use bollard::Docker;
use std::collections::HashMap;

/// A discovered container with its exposed ports.
#[derive(Debug, Clone)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    /// Ports exposed to the host: (host_port, container_port, protocol)
    pub ports: Vec<PortMapping>,
    /// docker-compose project name, if any
    pub compose_project: Option<String>,
    /// Labels from the container
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: String,
}

/// Connect to the Docker engine running inside WSL (unix socket).
///
/// On Windows, connects via the named pipe since WSL containers are
/// accessible through Docker Desktop's shared daemon.
pub fn connect_wsl_engine() -> Result<Docker> {
    #[cfg(unix)]
    {
        let docker = Docker::connect_with_unix_defaults()?;
        Ok(docker)
    }
    #[cfg(windows)]
    {
        let docker = Docker::connect_with_named_pipe_defaults()?;
        Ok(docker)
    }
}

/// Connect to the Docker engine running on Windows (named pipe).
///
/// On non-Windows platforms, falls back to HTTP connection to the default
/// Docker endpoint (useful for development/testing on Linux).
pub fn connect_windows_engine() -> Result<Docker> {
    #[cfg(windows)]
    {
        let docker = Docker::connect_with_named_pipe_defaults()?;
        Ok(docker)
    }
    #[cfg(not(windows))]
    {
        // On non-Windows, fall back to default connection for dev/testing
        let docker = Docker::connect_with_socket_defaults()?;
        Ok(docker)
    }
}

/// List running containers from a Docker engine connection.
pub async fn list_containers(docker: &Docker) -> Result<Vec<ContainerInfo>> {
    let options = ListContainersOptions::<String> {
        all: false, // only running
        ..Default::default()
    };

    let containers = docker.list_containers(Some(options)).await?;
    let mut result = Vec::new();

    for c in containers {
        let id = c.id.unwrap_or_default();
        let name = c
            .names
            .as_ref()
            .and_then(|n| n.first())
            .map(|n| n.trim_start_matches('/').to_string())
            .unwrap_or_else(|| id[..12].to_string());

        let image = c.image.unwrap_or_default();
        let status = c.status.unwrap_or_default();
        let labels = c.labels.unwrap_or_default();

        let compose_project = labels.get("com.docker.compose.project").cloned();

        let ports = c
            .ports
            .unwrap_or_default()
            .iter()
            .filter_map(|p| {
                let host_port = p.public_port? as u16;
                let container_port = p.private_port as u16;
                let protocol = p.typ.map(|t| format!("{t:?}").to_lowercase())
                    .unwrap_or_else(|| "tcp".into());
                Some(PortMapping {
                    host_port,
                    container_port,
                    protocol,
                })
            })
            .collect();

        result.push(ContainerInfo {
            id,
            name,
            image,
            status,
            ports,
            compose_project,
            labels,
        });
    }

    Ok(result)
}

/// List containers from the WSL Docker engine.
pub async fn list_wsl_containers() -> Result<Vec<ContainerInfo>> {
    let docker = connect_wsl_engine()?;
    list_containers(&docker).await
}

/// List containers from the Windows Docker engine.
pub async fn list_windows_containers() -> Result<Vec<ContainerInfo>> {
    let docker = connect_windows_engine()?;
    list_containers(&docker).await
}

/// Get port mappings for a specific container by ID.
pub async fn container_ports(docker: &Docker, container_id: &str) -> Result<Vec<PortMapping>> {
    let inspect = docker.inspect_container(container_id, None).await?;
    let mut ports = Vec::new();

    if let Some(network_settings) = inspect.network_settings {
        if let Some(port_map) = network_settings.ports {
            for (container_port_proto, bindings) in port_map {
                let container_port: u16 = container_port_proto
                    .split('/')
                    .next()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(0);

                let protocol = container_port_proto
                    .split('/')
                    .nth(1)
                    .unwrap_or("tcp")
                    .to_string();

                if let Some(bindings) = bindings {
                    for binding in bindings {
                        if let Some(hp) = binding.host_port.and_then(|p| p.parse::<u16>().ok()) {
                            ports.push(PortMapping {
                                host_port: hp,
                                container_port,
                                protocol: protocol.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(ports)
}
