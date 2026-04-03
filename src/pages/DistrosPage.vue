<script setup lang="ts">
import { ref, inject, onMounted, type Ref } from "vue";
import { Server, RefreshCw } from "lucide-vue-next";
import type { Rule, StatusInfo } from "../types";

const rules = inject<Ref<Rule[]>>("rules")!;
const status = inject<Ref<StatusInfo | null>>("status")!;
const loading = ref(false);

interface DistroInfo { name: string; state: string; version: number; default: boolean; ip: string | null }
const distros = ref<DistroInfo[]>([]);

async function refresh() {
  loading.value = true;
  try {
    if ("__TAURI__" in window) {
      // Parse `wsl -l -v` output to get distro list
      const { invoke } = await import("@tauri-apps/api/core");
      // For now, use detected IP from status for the default distro
      const wslIp = status.value?.wsl_ip ?? null;
      distros.value = [
        { name: "Ubuntu-24.04", state: wslIp ? "Running" : "Stopped", version: 2, default: true, ip: wslIp },
      ];
    } else {
      distros.value = [
        { name: "Ubuntu-24.04", state: "Running", version: 2, default: true, ip: "172.22.x.x" },
        { name: "Debian", state: "Stopped", version: 2, default: false, ip: null },
      ];
    }
  } catch {
    distros.value = [];
  }
  loading.value = false;
}
onMounted(refresh);
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-lg font-semibold" :style="{ color: 'var(--text-primary)' }">WSL Distros</h2>
      <button @click="refresh" class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg" :style="{ color: 'var(--accent)', border: '1px solid var(--border)' }">
        <RefreshCw :size="12" :class="{ 'animate-spin': loading }" /> Refresh
      </button>
    </div>
    <p class="text-sm mb-4" :style="{ color: 'var(--text-secondary)' }">Installed WSL distributions. Rules target the default distro unless a specific distro is set.</p>
    <div class="space-y-2">
      <div v-for="d in distros" :key="d.name" class="rounded-lg p-4" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }">
        <div class="flex items-center gap-3">
          <Server :size="16" :style="{ color: d.state === 'Running' ? 'var(--status-ok)' : 'var(--text-secondary)' }" />
          <span class="font-medium text-sm" :style="{ color: 'var(--text-primary)' }">{{ d.name }}</span>
          <span v-if="d.default" class="text-[10px] px-1.5 py-0.5 rounded" :style="{ background: 'var(--accent-dim)', color: 'var(--text-primary)' }">default</span>
          <span class="text-xs" :style="{ color: d.state === 'Running' ? 'var(--status-ok)' : 'var(--text-secondary)' }">{{ d.state }}</span>
          <span class="text-xs font-mono" :style="{ color: 'var(--text-secondary)' }">WSL{{ d.version }}</span>
          <span v-if="d.ip" class="text-xs font-mono" :style="{ color: 'var(--accent)' }">{{ d.ip }}</span>
          <div class="flex-1" />
          <span class="text-xs" :style="{ color: 'var(--text-secondary)' }">{{ rules.filter((r) => r.distro === d.name || (r.distro === null && d.default)).length }} rules</span>
        </div>
      </div>
    </div>
  </div>
</template>
