<script setup lang="ts">
import { ref, onMounted, onUnmounted, provide } from "vue";
import SidebarNav from "./components/SidebarNav.vue";
import StatusBar from "./components/StatusBar.vue";
import RulesPage from "./pages/RulesPage.vue";
import GroupsPage from "./pages/GroupsPage.vue";
import DockerSyncPage from "./pages/DockerSyncPage.vue";
import McpServersPage from "./pages/McpServersPage.vue";
import LanAccessPage from "./pages/LanAccessPage.vue";
import FirewallPage from "./pages/FirewallPage.vue";
import DistrosPage from "./pages/DistrosPage.vue";
import StartupActionsPage from "./pages/StartupActionsPage.vue";
import BootServicePage from "./pages/BootServicePage.vue";
import WslConfigPage from "./pages/WslConfigPage.vue";
import AuditLogPage from "./pages/AuditLogPage.vue";
import AppearancePage from "./pages/AppearancePage.vue";
import SettingsPage from "./pages/SettingsPage.vue";
import { useTheme } from "./hooks/useTheme";
import { useAuditLog } from "./hooks/useAuditLog";
import type { Page, StatusInfo, Rule } from "./types";

const isTauri = "__TAURI__" in window;

const activePage = ref<Page>("rules");
const status = ref<StatusInfo | null>(null);
const rules = ref<Rule[]>([]);
const theme = useTheme();
const audit = useAuditLog();

provide("rules", rules);
provide("status", status);
provide("audit", audit);

async function refreshStatus() {
  if (!isTauri) return;
  try {
    const { getStatus } = await import("./hooks/useTauri");
    status.value = await getStatus();
  } catch (e) {
    console.error("Failed to get status:", e);
  }
}

async function refreshRules() {
  if (!isTauri) {
    rules.value = getDemoRules();
    return;
  }
  try {
    const { getRules } = await import("./hooks/useTauri");
    rules.value = await getRules();
  } catch (e) {
    console.error("Failed to get rules:", e);
  }
}

provide("refreshRules", refreshRules);
provide("refreshStatus", refreshStatus);

let interval: ReturnType<typeof setInterval>;
onMounted(() => {
  refreshStatus();
  refreshRules();
  interval = setInterval(refreshStatus, 10000);
});
onUnmounted(() => clearInterval(interval));

const pageComponents: Record<Page, any> = {
  rules: RulesPage,
  groups: GroupsPage,
  docker: DockerSyncPage,
  mcp: McpServersPage,
  lan: LanAccessPage,
  firewall: FirewallPage,
  distros: DistrosPage,
  startup: StartupActionsPage,
  service: BootServicePage,
  wslconfig: WslConfigPage,
  audit: AuditLogPage,
  appearance: AppearancePage,
  settings: SettingsPage,
};

function getDemoRules(): Rule[] {
  return [
    { id: "1", name: "HTTP", direction: "winToWsl", listenAddr: "0.0.0.0", listenPort: { type: "single", port: 80 }, connectPort: { type: "single", port: 80 }, connectAddr: "${WSL_IP}", distro: null, lan: true, enabled: true, source: "manual", note: null, health: "ok" },
    { id: "2", name: "HTTP alt", direction: "winToWsl", listenAddr: "0.0.0.0", listenPort: { type: "single", port: 8080 }, connectPort: { type: "single", port: 80 }, connectAddr: "${WSL_IP}", distro: null, lan: true, enabled: true, source: "manual", note: "remapped: listen 8080 → connect 80", health: "ok" },
    { id: "3", name: "HTTPS", direction: "winToWsl", listenAddr: "0.0.0.0", listenPort: { type: "single", port: 443 }, connectPort: { type: "single", port: 443 }, connectAddr: "${WSL_IP}", distro: null, lan: true, enabled: true, source: "manual", note: null, health: "ok" },
    { id: "4", name: "SSH", direction: "winToWsl", listenAddr: "0.0.0.0", listenPort: { type: "single", port: 22 }, connectPort: { type: "single", port: 22 }, connectAddr: "${WSL_IP}", distro: null, lan: true, enabled: true, source: "manual", note: null, health: "warn" },
    { id: "5", name: "App range", direction: "winToWsl", listenAddr: "0.0.0.0", listenPort: { type: "range", start: 1024, end: 1048 }, connectPort: { type: "range", start: 1024, end: 1048 }, connectAddr: "${WSL_IP}", distro: null, lan: true, enabled: true, source: "imported", note: "collapsed from 25 individual rules", health: "unknown" },
    { id: "6", name: "Meilisearch", direction: "winToWsl", listenAddr: "0.0.0.0", listenPort: { type: "single", port: 7700 }, connectPort: { type: "single", port: 7700 }, connectAddr: "${WSL_IP}", distro: null, lan: true, enabled: false, source: "manual", note: null, health: "error" },
    { id: "7", name: "Vite dev", direction: "winToWsl", listenAddr: "0.0.0.0", listenPort: { type: "single", port: 5173 }, connectPort: { type: "single", port: 5173 }, connectAddr: "${WSL_IP}", distro: null, lan: true, enabled: true, source: "docker", note: null, health: "ok" },
  ];
}
</script>

<template>
  <div class="flex h-screen w-screen overflow-hidden no-select">
    <SidebarNav :active-page="activePage" :status="status" @navigate="activePage = $event" />
    <div class="flex flex-col flex-1 min-w-0">
      <main class="flex-1 overflow-y-auto p-6">
        <component :is="pageComponents[activePage]" />
      </main>
      <StatusBar :status="status" @sync="refreshStatus" />
    </div>
  </div>
</template>
