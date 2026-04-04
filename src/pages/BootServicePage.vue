<script setup lang="ts">
import { ref, onMounted } from "vue";
import { HardDrive, Play, Square, Download, Trash2, RefreshCw } from "lucide-vue-next";
import { useAuditLog } from "../hooks/useAuditLog";

const { log } = useAuditLog();
const svcStatus = ref<"running" | "stopped" | "not_installed" | "loading">("loading");

async function refresh() {
  if (!("__TAURI__" in window)) { svcStatus.value = "not_installed"; return; }
  try { const { getServiceStatus } = await import("../hooks/useTauri"); svcStatus.value = (await getServiceStatus()) as any; } catch { svcStatus.value = "not_installed"; }
}
onMounted(refresh);

async function install() {
  try {
    if ("__TAURI__" in window) { const { installService } = await import("../hooks/useTauri"); await installService(); }
    log("service.install", "Installed service");
  } catch (e) { log("service.install", `Failed: ${e}`, "error"); }
  refresh();
}
async function uninstall() {
  try {
    if ("__TAURI__" in window) { const { uninstallService } = await import("../hooks/useTauri"); await uninstallService(); }
    log("service.uninstall", "Uninstalled service");
  } catch (e) { log("service.uninstall", `Failed: ${e}`, "error"); }
  refresh();
}
async function toggleService() {
  const action = svcStatus.value === "running" ? "stop" : "start";
  try {
    if ("__TAURI__" in window) {
      const { invoke } = await import("@tauri-apps/api/core");
      // Use sc.exe via shell — service start/stop requires admin
      await invoke("plugin:shell|execute", { program: "sc", args: [action, "WslPortHole"] });
    }
    log(`service.${action}`, `${action === "start" ? "Started" : "Stopped"} service`);
  } catch (e) {
    log(`service.${action}`, `Failed: ${e}`, "error");
  }
  // Wait briefly for the state to change, then refresh
  setTimeout(refresh, 1500);
}

const statusColor = (s: string) => s === "running" ? "var(--status-ok)" : s === "stopped" ? "var(--status-warn)" : "var(--text-secondary)";
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold mb-2" :style="{ color: 'var(--text-primary)' }">Boot Service</h2>
    <p class="text-sm mb-6" :style="{ color: 'var(--text-secondary)' }">The WSL PortHole Windows Service runs at boot and auto-manages port forwarding rules when WSL's IP changes.</p>
    <div class="rounded-lg p-5 mb-6" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }">
      <div class="flex items-center gap-3 mb-4">
        <HardDrive :size="20" :style="{ color: statusColor(svcStatus) }" />
        <span class="text-base font-semibold" :style="{ color: 'var(--text-primary)' }">WslPortHole</span>
        <span class="text-xs px-2 py-0.5 rounded" :style="{ background: statusColor(svcStatus), color: '#000' }">{{ svcStatus === "loading" ? "..." : svcStatus }}</span>
        <div class="flex-1" />
        <button @click="refresh" class="p-1" :style="{ color: 'var(--text-secondary)' }"><RefreshCw :size="14" /></button>
      </div>
      <div class="grid grid-cols-2 gap-3">
        <button v-if="svcStatus === 'not_installed'" @click="install" class="flex items-center justify-center gap-2 px-4 py-2 rounded-lg text-sm font-medium col-span-2" :style="{ background: 'var(--accent)', color: 'var(--bg-primary)' }"><Download :size="14" /> Install Service</button>
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
        <li>Registers as a Windows Service with auto-start on boot</li>
        <li>Subscribes to Hyper-V VmSwitch event log (Event ID 102)</li>
        <li>When WSL's IP changes, waits 5s for it to settle</li>
        <li>Re-applies all enabled portproxy + firewall rules with the new IP</li>
        <li>Sends a Windows toast notification</li>
        <li>Falls back to 30s polling if event subscription is unavailable</li>
      </ol>
    </div>
  </div>
</template>
