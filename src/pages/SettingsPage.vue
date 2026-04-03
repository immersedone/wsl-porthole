<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { Info } from "lucide-vue-next";
import { isTauri } from "../lib/tauri";

declare const __APP_VERSION__: string;
const appVersion = ref("...");

const startMinimized = ref(false);
const minimizeToTray = ref(true);
const healthCheckInterval = ref(60);
const ipSettleDelay = ref(5);
const pollingInterval = ref(30);
const defaultListenAddr = ref("0.0.0.0");
const toastOnIpChange = ref(true);
const toastOnConflict = ref(true);

async function loadSettings() {
  if (isTauri) {
    try {
      const { getVersion } = await import("@tauri-apps/api/app");
      appVersion.value = await getVersion();
    } catch { appVersion.value = __APP_VERSION__; }
    try {
      const { getSettings } = await import("../hooks/useTauri");
      const s = await getSettings();
      const p = s.preferences ?? {};
      startMinimized.value = p.startMinimized ?? false;
      minimizeToTray.value = p.minimizeToTray ?? true;
      healthCheckInterval.value = p.healthCheckInterval ?? 60;
      ipSettleDelay.value = p.ipSettleDelay ?? 5;
      pollingInterval.value = p.pollingInterval ?? 30;
      defaultListenAddr.value = p.defaultListenAddr ?? "0.0.0.0";
      toastOnIpChange.value = p.toastOnIpChange ?? true;
      toastOnConflict.value = p.toastOnConflict ?? true;
    } catch {}
  } else {
    appVersion.value = __APP_VERSION__;
  }
}

async function persist() {
  if (!isTauri) return;
  try {
    const { getSettings, saveSettings } = await import("../hooks/useTauri");
    const s = await getSettings();
    s.preferences = {
      startMinimized: startMinimized.value,
      minimizeToTray: minimizeToTray.value,
      healthCheckInterval: healthCheckInterval.value,
      ipSettleDelay: ipSettleDelay.value,
      pollingInterval: pollingInterval.value,
      defaultListenAddr: defaultListenAddr.value,
      toastOnIpChange: toastOnIpChange.value,
      toastOnConflict: toastOnConflict.value,
      theme: "",
    };
    await saveSettings(s);
  } catch {}
}

onMounted(loadSettings);

// Auto-save when any setting changes
watch([startMinimized, minimizeToTray, healthCheckInterval, ipSettleDelay, pollingInterval, defaultListenAddr, toastOnIpChange, toastOnConflict], persist);
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold mb-4" :style="{ color: 'var(--text-primary)' }">Settings</h2>
    <div class="space-y-4">
      <div class="rounded-lg overflow-hidden" :style="{ border: '1px solid var(--border)' }">
        <div class="px-4 py-2 text-xs font-semibold uppercase tracking-wider" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-secondary)' }">General</div>
        <div :style="{ background: 'var(--bg-secondary)' }">
          <div class="flex items-center justify-between px-4 py-3" :style="{ borderBottom: '1px solid var(--border)' }">
            <div><div class="text-sm" :style="{ color: 'var(--text-primary)' }">Start minimized to tray</div><div class="text-xs" :style="{ color: 'var(--text-secondary)' }">Launch in the system tray</div></div>
            <input type="checkbox" v-model="startMinimized" class="accent-[var(--accent)]" />
          </div>
          <div class="flex items-center justify-between px-4 py-3" :style="{ borderBottom: '1px solid var(--border)' }">
            <div><div class="text-sm" :style="{ color: 'var(--text-primary)' }">Minimize to tray on close</div><div class="text-xs" :style="{ color: 'var(--text-secondary)' }">Keep running in background</div></div>
            <input type="checkbox" v-model="minimizeToTray" class="accent-[var(--accent)]" />
          </div>
          <div class="flex items-center justify-between px-4 py-3">
            <div><div class="text-sm" :style="{ color: 'var(--text-primary)' }">Health check interval</div><div class="text-xs" :style="{ color: 'var(--text-secondary)' }">Seconds between TCP health checks</div></div>
            <input type="number" v-model.number="healthCheckInterval" class="w-20 text-sm px-2 py-1 rounded text-right" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border)' }" />
          </div>
        </div>
      </div>
      <div class="rounded-lg overflow-hidden" :style="{ border: '1px solid var(--border)' }">
        <div class="px-4 py-2 text-xs font-semibold uppercase tracking-wider" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-secondary)' }">Networking</div>
        <div :style="{ background: 'var(--bg-secondary)' }">
          <div class="flex items-center justify-between px-4 py-3" :style="{ borderBottom: '1px solid var(--border)' }">
            <div><div class="text-sm" :style="{ color: 'var(--text-primary)' }">IP settle delay</div><div class="text-xs" :style="{ color: 'var(--text-secondary)' }">Seconds to wait after WSL IP change</div></div>
            <input type="number" v-model.number="ipSettleDelay" class="w-20 text-sm px-2 py-1 rounded text-right" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border)' }" />
          </div>
          <div class="flex items-center justify-between px-4 py-3" :style="{ borderBottom: '1px solid var(--border)' }">
            <div><div class="text-sm" :style="{ color: 'var(--text-primary)' }">Polling interval</div><div class="text-xs" :style="{ color: 'var(--text-secondary)' }">Fallback polling in seconds</div></div>
            <input type="number" v-model.number="pollingInterval" class="w-20 text-sm px-2 py-1 rounded text-right" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border)' }" />
          </div>
          <div class="flex items-center justify-between px-4 py-3">
            <div><div class="text-sm" :style="{ color: 'var(--text-primary)' }">Default listen address</div><div class="text-xs" :style="{ color: 'var(--text-secondary)' }">Default bind address for new rules</div></div>
            <select v-model="defaultListenAddr" class="text-sm px-2 py-1 rounded" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border)' }">
              <option value="0.0.0.0">0.0.0.0 (LAN)</option><option value="127.0.0.1">127.0.0.1 (local)</option>
            </select>
          </div>
        </div>
      </div>
      <div class="rounded-lg overflow-hidden" :style="{ border: '1px solid var(--border)' }">
        <div class="px-4 py-2 text-xs font-semibold uppercase tracking-wider" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-secondary)' }">Notifications</div>
        <div :style="{ background: 'var(--bg-secondary)' }">
          <div class="flex items-center justify-between px-4 py-3" :style="{ borderBottom: '1px solid var(--border)' }">
            <div><div class="text-sm" :style="{ color: 'var(--text-primary)' }">Toast on IP change</div><div class="text-xs" :style="{ color: 'var(--text-secondary)' }">Show notification when WSL IP changes</div></div>
            <input type="checkbox" v-model="toastOnIpChange" class="accent-[var(--accent)]" />
          </div>
          <div class="flex items-center justify-between px-4 py-3">
            <div><div class="text-sm" :style="{ color: 'var(--text-primary)' }">Toast on conflict</div><div class="text-xs" :style="{ color: 'var(--text-secondary)' }">Notify when a port conflict is detected</div></div>
            <input type="checkbox" v-model="toastOnConflict" class="accent-[var(--accent)]" />
          </div>
        </div>
      </div>
      <div class="rounded-lg p-4" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }">
        <div class="flex items-center gap-2 mb-2"><Info :size="14" :style="{ color: 'var(--accent)' }" /><h3 class="text-sm font-semibold" :style="{ color: 'var(--text-primary)' }">About</h3></div>
        <div class="text-xs space-y-1" :style="{ color: 'var(--text-secondary)' }">
          <p><strong>WSL PortHole</strong> v{{ appVersion }}</p>
          <p>Manages port forwarding and firewall rules across Windows, WSL2, and Docker.</p>
          <p>Built with Tauri v2 + Vue 3 + Rust</p>
        </div>
      </div>
    </div>
  </div>
</template>
