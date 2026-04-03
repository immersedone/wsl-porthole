<script setup lang="ts">
import { ref, inject, onMounted, type Ref } from "vue";
import { Server, RefreshCw, Edit2, Check, X } from "lucide-vue-next";
import type { Rule, StatusInfo } from "../types";
import { isTauri } from "../lib/tauri";

const rules = inject<Ref<Rule[]>>("rules")!;
const status = inject<Ref<StatusInfo | null>>("status")!;
const loading = ref(false);

interface DistroInfo { name: string; alias: string; state: string; version: number; default: boolean; ip: string | null }
const distros = ref<DistroInfo[]>([]);
const editingAlias = ref<string | null>(null);
const aliasInput = ref("");

// Load aliases from localStorage
function loadAliases(): Record<string, string> {
  try { return JSON.parse(localStorage.getItem("wsl-porthole-distro-aliases") ?? "{}"); } catch { return {}; }
}
function saveAliases(aliases: Record<string, string>) {
  localStorage.setItem("wsl-porthole-distro-aliases", JSON.stringify(aliases));
}

async function refresh() {
  loading.value = true;
  const aliases = loadAliases();
  try {
    if (isTauri) {
      // Parse `wsl -l -v` output
      const output = await new Promise<string>((resolve) => {
        // Use WSL IP from status as indicator of running state
        const wslIp = status.value?.wsl_ip ?? null;
        resolve(wslIp ? `Ubuntu-24.04|Running|2|${wslIp}` : "Ubuntu-24.04|Stopped|2|");
      });
      // For now, build from status — future: parse wsl.exe -l -v
      const wslIp = status.value?.wsl_ip ?? null;
      distros.value = [
        { name: "Ubuntu-24.04", alias: aliases["Ubuntu-24.04"] ?? "", state: wslIp ? "Running" : "Stopped", version: 2, default: true, ip: wslIp },
      ];
    } else {
      distros.value = [
        { name: "Ubuntu-24.04", alias: aliases["Ubuntu-24.04"] ?? "Main Dev", state: "Running", version: 2, default: true, ip: "172.22.x.x" },
        { name: "Debian", alias: aliases["Debian"] ?? "", state: "Stopped", version: 2, default: false, ip: null },
      ];
    }
  } catch {
    distros.value = [];
  }
  loading.value = false;
}

function startEditAlias(d: DistroInfo) {
  editingAlias.value = d.name;
  aliasInput.value = d.alias;
}

function saveAlias(d: DistroInfo) {
  const aliases = loadAliases();
  d.alias = aliasInput.value.trim();
  if (d.alias) { aliases[d.name] = d.alias; } else { delete aliases[d.name]; }
  saveAliases(aliases);
  editingAlias.value = null;
}

function cancelEditAlias() { editingAlias.value = null; }

function displayName(d: DistroInfo) { return d.alias || d.name; }

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
    <p class="text-sm mb-4" :style="{ color: 'var(--text-secondary)' }">Installed WSL distributions. Click the edit icon to set a friendly alias. Rules target the default distro unless a specific distro is set.</p>
    <div class="space-y-2">
      <div v-for="d in distros" :key="d.name" class="rounded-lg p-4" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }">
        <div class="flex items-center gap-3">
          <Server :size="16" :style="{ color: d.state === 'Running' ? 'var(--status-ok)' : 'var(--text-secondary)' }" />
          <!-- Name + alias -->
          <template v-if="editingAlias === d.name">
            <input v-model="aliasInput" @keydown.enter="saveAlias(d)" @keydown.escape="cancelEditAlias"
              class="w-40 px-2 py-0.5 text-sm rounded outline-none"
              :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--accent)' }"
              placeholder="Alias (e.g. Main Dev)" autofocus />
            <button @click="saveAlias(d)" :style="{ color: 'var(--status-ok)' }"><Check :size="14" /></button>
            <button @click="cancelEditAlias" :style="{ color: 'var(--text-secondary)' }"><X :size="14" /></button>
          </template>
          <template v-else>
            <span class="font-medium text-sm" :style="{ color: 'var(--text-primary)' }">{{ d.name }}</span>
            <span v-if="d.alias" class="text-xs px-1.5 py-0.5 rounded" :style="{ background: 'var(--accent-dim)', color: 'var(--text-primary)' }">{{ d.alias }}</span>
            <button @click="startEditAlias(d)" class="p-0.5 rounded hover:opacity-80" :style="{ color: 'var(--text-secondary)' }" title="Set alias"><Edit2 :size="12" /></button>
          </template>
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
