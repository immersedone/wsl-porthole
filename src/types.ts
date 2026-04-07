// Core type definitions matching the Rust models

export type Direction = "winToWsl" | "wslToWin";
export type Source = "manual" | "docker" | "mcp" | "imported";
export type HealthStatus = "ok" | "warn" | "error" | "unknown";

export interface PortSpec {
  type: "single" | "range";
  port?: number;
  start?: number;
  end?: number;
}

export interface Rule {
  id: string;
  name: string;
  direction: Direction;
  listenAddr: string;
  listenPort: PortSpec;
  connectPort: PortSpec;
  connectAddr: string;
  distro: string | null;
  lan: boolean;
  enabled: boolean;
  source: Source;
  note: string | null;
  // Frontend-only
  health?: HealthStatus;
  conflict?: string | null;
  group?: string | null;
}

export interface StatusInfo {
  wsl_ip: string | null;
  host_ip: string | null;
  host_gw: string | null;
  active_rules: number;
  lan_rules: number;
  total_rules: number;
  wsl_error: string | null;
  host_error: string | null;
  config_dir: string | null;
}

export interface ActiveProxy {
  listen_addr: string;
  listen_port: number;
  connect_addr: string;
  connect_port: number;
}

export interface ContainerSummary {
  id: string;
  name: string;
  image: string;
  status: string;
  ports: PortSummary[];
  compose_project: string | null;
}

export interface PortSummary {
  host_port: number;
  container_port: number;
  protocol: string;
}

export interface McpServerInfo {
  container_name: string;
  image: string;
  port: number;
  host_port: number;
  detection_reason: string;
}

export interface AuditEntry {
  id?: number;
  timestamp: string;
  event: string;
  detail: string;
  level: "info" | "warn" | "error";
}

export interface ThemeTokens {
  "--bg-primary": string;
  "--bg-secondary": string;
  "--bg-tertiary": string;
  "--accent": string;
  "--accent-dim": string;
  "--text-primary": string;
  "--text-secondary": string;
  "--status-ok": string;
  "--status-warn": string;
  "--status-err": string;
  "--border": string;
  [key: string]: string;
}

export interface Theme {
  name: string;
  category: "light" | "dark" | "auto" | "accessibility";
  tokens: ThemeTokens;
}

export type Page =
  | "rules"
  | "groups"
  | "docker"
  | "mcp"
  | "lan"
  | "firewall"
  | "distros"
  | "startup"
  | "service"
  | "wslconfig"
  | "audit"
  | "appearance"
  | "updates"
  | "settings";
