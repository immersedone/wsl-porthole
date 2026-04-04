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
import UpdatesPage from "./pages/UpdatesPage.vue";
import SettingsPage from "./pages/SettingsPage.vue";
import ToastContainer from "./components/ToastContainer.vue";
import { useTheme } from "./hooks/useTheme";
import { useAuditLog } from "./hooks/useAuditLog";
import { useToast } from "./hooks/useToast";
import type { Page, StatusInfo, Rule } from "./types";

const isTauri = "__TAURI__" in window || "__TAURI_INTERNALS__" in window;

const activePage = ref<Page>("rules");
const status = ref<StatusInfo | null>(null);
const rules = ref<Rule[]>([]);
const startupError = ref<string | null>(null);
const diagOutput = ref<string | null>(null);
const theme = useTheme();
const audit = useAuditLog();
const toast = useToast();

provide("rules", rules);
provide("status", status);
provide("audit", audit);
provide("toast", toast);

async function refreshStatus() {
  if (!isTauri) return;
  try {
    const { getStatus } = await import("./hooks/useTauri");
    status.value = await getStatus();
    if (status.value) startupError.value = null;
  } catch (e) {
    startupError.value = `Status check failed: ${e}`;
  }
}

async function refreshRules() {
  if (!isTauri) return;
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
onMounted(async () => {
  refreshStatus();
  refreshRules();
  interval = setInterval(refreshStatus, 10000);

  // Run diagnostics on startup to catch issues early
  if (isTauri) {
    try {
      const { diagnose } = await import("./hooks/useTauri");
      const diag = await diagnose();
      diagOutput.value = JSON.stringify(diag, null, 2);
    } catch (e) {
      diagOutput.value = `Diagnostics failed: ${e}`;
    }
  }

  // Global keyboard shortcuts
  document.addEventListener("keydown", handleKeydown);
});
onUnmounted(() => {
  clearInterval(interval);
  document.removeEventListener("keydown", handleKeydown);
});

function handleKeydown(e: KeyboardEvent) {
  // Ctrl+N — add rule (navigate to rules page, trigger add)
  if (e.ctrlKey && e.key === "n") {
    e.preventDefault();
    activePage.value = "rules";
  }
  // Ctrl+F — focus search (navigate to rules page)
  if (e.ctrlKey && e.key === "f") {
    e.preventDefault();
    activePage.value = "rules";
    // Focus the search input after page renders
    setTimeout(() => {
      const input = document.querySelector<HTMLInputElement>('input[placeholder="Search rules..."]');
      input?.focus();
    }, 50);
  }
  // Ctrl+S — sync now
  if (e.ctrlKey && e.key === "s") {
    e.preventDefault();
    refreshStatus();
  }
}

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
  updates: UpdatesPage,
  settings: SettingsPage,
};


</script>

<template>
  <div class="flex h-screen w-screen overflow-hidden no-select">
    <SidebarNav :active-page="activePage" :status="status" @navigate="activePage = $event" />
    <div class="flex flex-col flex-1 min-w-0">
      <!-- Error banner (only shows on actual failures) -->
      <div v-if="startupError" class="px-4 py-2 text-xs border-b"
        :style="{ background: 'rgba(248,81,73,0.1)', borderColor: 'var(--status-err)', color: 'var(--status-err)' }">
        <strong>Error:</strong> {{ startupError }}
      </div>
      <main class="flex-1 overflow-y-auto p-6">
        <component :is="pageComponents[activePage]" />
      </main>
      <StatusBar :status="status" @sync="refreshStatus" />
    </div>
    <ToastContainer />
  </div>
</template>
