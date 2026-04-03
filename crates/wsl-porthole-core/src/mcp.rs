//! MCP (Model Context Protocol) server detection from Docker containers.
//!
//! Scans running Docker containers for known MCP server patterns:
//! - Container image names containing "mcp"
//! - Container labels indicating MCP capability
//! - Known MCP server ports

use crate::docker::ContainerInfo;
use std::collections::HashMap;

/// Known MCP server image patterns.
const MCP_IMAGE_PATTERNS: &[&str] = &[
    "mcp-server",
    "mcp_server",
    "modelcontextprotocol",
    "/mcp",
];

/// Known default MCP server ports.
const MCP_DEFAULT_PORTS: &[u16] = &[3000, 8080, 8443];

/// An identified MCP server running in a Docker container.
#[derive(Debug, Clone)]
pub struct McpServer {
    pub container_id: String,
    pub container_name: String,
    pub image: String,
    pub port: u16,
    pub host_port: u16,
    pub protocol: String,
    pub detection_reason: DetectionReason,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectionReason {
    ImageName,
    Label,
    KnownPort,
}

/// Detect MCP servers from a list of running containers.
pub fn detect_mcp_servers(containers: &[ContainerInfo]) -> Vec<McpServer> {
    let mut servers = Vec::new();

    for container in containers {
        let image_lower = container.image.to_lowercase();

        // Check image name
        let image_match = MCP_IMAGE_PATTERNS
            .iter()
            .any(|pat| image_lower.contains(pat));

        // Check labels
        let label_match = container
            .labels
            .keys()
            .any(|k| k.to_lowercase().contains("mcp"));

        if image_match || label_match {
            let reason = if image_match {
                DetectionReason::ImageName
            } else {
                DetectionReason::Label
            };

            for port in &container.ports {
                servers.push(McpServer {
                    container_id: container.id.clone(),
                    container_name: container.name.clone(),
                    image: container.image.clone(),
                    port: port.container_port,
                    host_port: port.host_port,
                    protocol: port.protocol.clone(),
                    detection_reason: reason.clone(),
                    labels: container.labels.clone(),
                });
            }

            // If no ports exposed, still record it with port 0
            if container.ports.is_empty() {
                servers.push(McpServer {
                    container_id: container.id.clone(),
                    container_name: container.name.clone(),
                    image: container.image.clone(),
                    port: 0,
                    host_port: 0,
                    protocol: "tcp".into(),
                    detection_reason: reason,
                    labels: container.labels.clone(),
                });
            }
        }
    }

    servers
}

/// Suggest which container ports might be MCP servers based on known ports.
pub fn suggest_mcp_ports(containers: &[ContainerInfo]) -> Vec<McpServer> {
    let mut suggestions = Vec::new();

    for container in containers {
        for port in &container.ports {
            if MCP_DEFAULT_PORTS.contains(&port.container_port) {
                suggestions.push(McpServer {
                    container_id: container.id.clone(),
                    container_name: container.name.clone(),
                    image: container.image.clone(),
                    port: port.container_port,
                    host_port: port.host_port,
                    protocol: port.protocol.clone(),
                    detection_reason: DetectionReason::KnownPort,
                    labels: container.labels.clone(),
                });
            }
        }
    }

    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::docker::PortMapping;

    fn make_container(name: &str, image: &str, ports: Vec<PortMapping>, labels: HashMap<String, String>) -> ContainerInfo {
        ContainerInfo {
            id: format!("{name}-id"),
            name: name.into(),
            image: image.into(),
            status: "running".into(),
            ports,
            compose_project: None,
            labels,
        }
    }

    #[test]
    fn test_detect_by_image_name() {
        let containers = vec![make_container(
            "my-mcp",
            "ghcr.io/org/mcp-server-github:latest",
            vec![PortMapping {
                host_port: 3000,
                container_port: 3000,
                protocol: "tcp".into(),
            }],
            HashMap::new(),
        )];
        let servers = detect_mcp_servers(&containers);
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].detection_reason, DetectionReason::ImageName);
    }

    #[test]
    fn test_detect_by_label() {
        let mut labels = HashMap::new();
        labels.insert("com.mcp.server".into(), "true".into());
        let containers = vec![make_container(
            "custom-server",
            "myapp:latest",
            vec![PortMapping {
                host_port: 8080,
                container_port: 8080,
                protocol: "tcp".into(),
            }],
            labels,
        )];
        let servers = detect_mcp_servers(&containers);
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].detection_reason, DetectionReason::Label);
    }

    #[test]
    fn test_no_match() {
        let containers = vec![make_container(
            "postgres",
            "postgres:16",
            vec![PortMapping {
                host_port: 5432,
                container_port: 5432,
                protocol: "tcp".into(),
            }],
            HashMap::new(),
        )];
        let servers = detect_mcp_servers(&containers);
        assert!(servers.is_empty());
    }

    #[test]
    fn test_suggest_known_ports() {
        let containers = vec![make_container(
            "some-app",
            "myapp:latest",
            vec![
                PortMapping {
                    host_port: 3000,
                    container_port: 3000,
                    protocol: "tcp".into(),
                },
                PortMapping {
                    host_port: 5432,
                    container_port: 5432,
                    protocol: "tcp".into(),
                },
            ],
            HashMap::new(),
        )];
        let suggestions = suggest_mcp_ports(&containers);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].port, 3000);
    }
}
