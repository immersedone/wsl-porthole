import { invoke } from "@tauri-apps/api/core";
import type {
  Rule,
  StatusInfo,
  ActiveProxy,
  ContainerSummary,
  McpServerInfo,
} from "../types";

// Rule CRUD
export const getRules = () => invoke<Rule[]>("get_rules");
export const saveRules = (rules: Rule[]) =>
  invoke<void>("save_rules", { rules });
export const addRule = (params: {
  name: string;
  direction: string;
  listenAddr: string;
  listenPort: number;
  connectPort: number;
  connectAddr: string;
  lan: boolean;
}) => invoke<Rule>("add_rule", params);
export const updateRule = (rule: Rule) => invoke<void>("update_rule", { rule });
export const deleteRule = (id: string) => invoke<void>("delete_rule", { id });
export const toggleRule = (id: string) => invoke<boolean>("toggle_rule", { id });

// Netsh operations
export const applyRules = () => invoke<string>("apply_rules");
export const removeAppliedRule = (id: string) =>
  invoke<void>("remove_applied_rule", { id });
export const listActiveProxies = () =>
  invoke<ActiveProxy[]>("list_active_proxies");
export const previewNetshCommand = (id: string) =>
  invoke<string[]>("preview_netsh_command", { id });

// Status
export const getStatus = () => invoke<StatusInfo>("get_status");
export const detectIps = () =>
  invoke<{ wsl_ip: string | null; host_ip: string | null; host_gw: string | null }>(
    "detect_ips"
  );
export const syncNow = () => invoke<string>("sync_now");

// Import
export const importNetshScript = (script: string) =>
  invoke<Rule[]>("import_netsh_script", { script });

// Docker
export const listDockerContainers = () =>
  invoke<ContainerSummary[]>("list_docker_containers");
export const detectMcpServers = () =>
  invoke<McpServerInfo[]>("detect_mcp_servers");

// Firewall
export const getFirewallRules = () => invoke<string[]>("get_firewall_rules");

// Service
export const installService = () => invoke<string>("install_service");
export const uninstallService = () => invoke<string>("uninstall_service");
export const getServiceStatus = () => invoke<string>("get_service_status");
