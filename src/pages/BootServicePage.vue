<script setup lang="ts">
import { ref, onMounted } from "vue";
import { HardDrive, Play, Square, Download, Trash2, RefreshCw } from "lucide-vue-next";
import { useAuditLog } from "../hooks/useAuditLog";
import { useToast } from "../hooks/useToast";
import { isTauri } from "../lib/tauri";

const { log } = useAuditLog();
const { show: showToast } = useToast();
const svcStatus = ref<"running" | "stopped" | "not_installed" | "loading">("loading");
const installing = ref(false);

const SVC_CACHE_KEY = "wsl-porthole-svc-status-cache";

async function refresh() {
  if (!isTauri) { svcStatus.value = "not_installed"; return; }
  try {
    const { getServiceStatus } = await import("../hooks/useTauri");
    svcStatus.value = (await getServiceStatus()) as any;
    localStorage.setItem(SVC_CACHE_KEY, svcStatus.value);
  } catch { svcStatus.value = "not_installed"; }
}

onMounted(() => {
  // Show cached status immediately, then refresh in background
  const cached = localStorage.getItem(SVC_CACHE_KEY) as typeof svcStatus.value | null;
  if (cached && ["running", "stopped", "not_installed"].includes(cached)) {
    svcStatus.value = cached;
  }
  refresh();
});

async function install() {
  installing.value = true;
  try {
    if (isTauri) {
      const { installService } = await import("../hooks/useTauri");
      const result = await installService();
      showToast(result, "success");
    }
    log("service.install", "Installed service");
  } catch (e) {
    log("service.install", `Failed: ${e}`, "error");
    showToast(`Install failed: ${e}`, "error");
  }
  installing.value = false;
  refresh();
}
async function uninstall() {
  try {
    if (isTauri) {
      const { uninstallService } = await import("../hooks/useTauri");
      const result = await uninstallService();
      showToast(result, "success");
    }
    log("service.uninstall", "Uninstalled service");
  } catch (e) {
    log("service.uninstall", `Failed: ${e}`, "error");
    showToast(`Uninstall failed: ${e}`, "error");
  }
  refresh();
}
async function toggleService() {
  const action = svcStatus.value === "running" ? "stop" : "start";
  try {
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("plugin:shell|execute", { program: "sc", args: [action, "WslPortHole"] });
    }
    log(`service.${action}`, `${action === "start" ? "Started" : "Stopped"} service`);
    showToast(`Service ${action === "start" ? "started" : "stopped"}`, "success");
  } catch (e) {
    log(`service.${action}`, `Failed: ${e}`, "error");
    showToast(`Failed to ${action} service: ${e}`, "error");
  }
  setTimeout(refresh, 1500);
}

const statusColor = (s: string) => s === "running" ? "var(--status-ok)" : s === "stopped" ? "var(--status-warn)" : "var(--text-secondary)";
const statusLabel = (s: string) => {
  switch (s) {
    case "running": return "Running";
    case "stopped": return "Stopped";
    case "not_installed": return "Not Installed";
    case "loading": return "...";
    default: return s;
  }
};
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold mb-2" :style="{ color: 'var(--text-primary)' }">Boot Service</h2>
    <p class="text-sm mb-6" :style="{ color: 'var(--text-secondary)' }">The WSL PortHole Windows Service runs at boot and auto-manages port forwarding rules when WSL's IP changes.</p>
    <div class="rounded-lg p-5 mb-6" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }">
      <div class="flex items-center gap-3 mb-4">
        <HardDrive :size="20" :style="{ color: statusColor(svcStatus) }" />
        <span class="text-base font-semibold" :style="{ color: 'var(--text-primary)' }">WslPortHole</span>
        <span class="text-xs px-2 py-0.5 rounded" :style="{ background: statusColor(svcStatus), color: '#000' }">{{ statusLabel(svcStatus) }}</span>
        <div class="flex-1" />
        <button @click="refresh" class="p-1" :style="{ color: 'var(--text-secondary)' }" title="Refresh service status"><RefreshCw :size="14" /></button>
      </div>
      <div class="grid grid-cols-2 gap-3">
        <button v-if="svcStatus === 'not_installed'" @click="install" :disabled="installing"
          class="flex items-center justify-center gap-2 px-4 py-2 rounded-lg text-sm font-medium col-span-2"
          :style="{ background: installing ? 'var(--accent-dim)' : 'var(--accent)', color: 'var(--bg-primary)' }">
          <RefreshCw v-if="installing" :size="14" class="animate-spin" />
          <Download v-else :size="14" />
          {{ installing ? "Downloading & installing..." : "Install Service" }}
        </button>
        <template v-else>
          <button @click="toggleService" class="flex items-center justify-center gap-2 px-4 py-2 rounded-lg text-sm" :style="{ border: '1px solid var(--border)', color: 'var(--text-primary)' }">
            <template v-if="svcStatus === 'running'"><Square :size="14" /> Stop</template>
            <template v-else><Play :size="14" /> Start</template>
          </button>
          <button @click="uninstall" class="flex items-center justify-center gap-2 px-4 py-2 rounded-lg text-sm" :style="{ border: '1px solid var(--status-err)', color: 'var(--status-err)' }"><Trash2 :size="14" /> Uninstall</button>
        </template>
      </div>
    </div>
    <div class="rounded-lg p-4" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }">
      <h3 class="text-sm font-semibold mb-2" :style="{ color: 'var(--text-primary)' }">How it works</h3>
      <ol class="text-xs space-y-1.5 list-decimal list-inside" :style="{ color: 'var(--text-secondary)' }">
        <li>Downloads and registers as a Windows Service with auto-start on boot</li>
        <li>Subscribes to Hyper-V VmSwitch event log (Event ID 102)</li>
        <li>When WSL's IP changes, waits 5s for it to settle</li>
        <li>Re-applies all enabled portproxy + firewall rules with the new IP</li>
        <li>Sends a Windows toast notification</li>
        <li>Falls back to 30s polling if event subscription is unavailable</li>
      </ol>
    </div>
    <p class="text-xs mt-4" :style="{ color: 'var(--text-secondary)' }">
      Note: Installing the service requires administrator privileges. If the install fails, try running the app as Administrator.
    </p>
  </div>
</template>
