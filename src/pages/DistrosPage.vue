<script setup lang="ts">
import { ref, inject, onMounted, type Ref } from "vue";
import { Server, RefreshCw, Edit2, Check, X, Play, Pause, Terminal, Monitor } from "lucide-vue-next";
import type { Rule, StatusInfo } from "../types";
import { isTauri } from "../lib/tauri";
import { useAuditLog } from "../hooks/useAuditLog";
import { useToast } from "../hooks/useToast";
import { useAlive } from "../hooks/useAlive";

const alive = useAlive();
const rules = inject<Ref<Rule[]>>("rules")!;
const status = inject<Ref<StatusInfo | null>>("status")!;
const { log } = useAuditLog();
const { show: showToast } = useToast();
const loading = ref(false);
const loadError = ref<string | null>(null);

interface DistroInfo {
  name: string;
  alias: string;
  state: string;
  version: number;
  default: boolean;
  ip: string | null;
}
const distros = ref<DistroInfo[]>([]);
const editingAlias = ref<string | null>(null);
const aliasInput = ref("");

const CACHE_KEY = "wsl-porthole-distros-cache";

function loadAliases(): Record<string, string> {
  try { return JSON.parse(localStorage.getItem("wsl-porthole-distro-aliases") ?? "{}"); } catch { return {}; }
}
function saveAliases(aliases: Record<string, string>) {
  localStorage.setItem("wsl-porthole-distro-aliases", JSON.stringify(aliases));
}

function loadCache(): DistroInfo[] {
  try {
    const raw = localStorage.getItem(CACHE_KEY);
    if (!raw) return [];
    return JSON.parse(raw);
  } catch { return []; }
}

function saveCache(data: DistroInfo[]) {
  localStorage.setItem(CACHE_KEY, JSON.stringify(data));
}

function applyAliases(raw: { name: string; state: string; version: number; default: boolean; ip: string | null }[]): DistroInfo[] {
  const aliases = loadAliases();
  return raw.map((d) => ({
    name: d.name,
    alias: aliases[d.name] ?? "",
    state: d.state,
    version: d.version,
    default: d.default,
    ip: d.ip,
  }));
}

async function refresh() {
  loading.value = true;
  loadError.value = null;
  try {
    if (!isTauri) { loading.value = false; return; }
    const { listDistros } = await import("../hooks/useTauri");
    const raw = await listDistros();
    if (!alive.value) return;
    distros.value = applyAliases(raw);
    saveCache(distros.value);
    log("distros.refresh", `Loaded ${distros.value.length} distros`);
  } catch (e) {
    if (!alive.value) return;
    console.error("Failed to list distros:", e);
    loadError.value = String(e);
    if (!distros.value.length) distros.value = [];
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
  log("distro.alias", `Set alias for ${d.name}: "${d.alias || '(none)'}"`);
  showToast(`Alias ${d.alias ? 'set' : 'cleared'} for ${d.name}`, "success");
}

function cancelEditAlias() { editingAlias.value = null; }

function ruleCount(d: DistroInfo) {
  return rules.value.filter((r) => r.distro === d.name || (r.distro === null && d.default)).length;
}

function stateColor(state: string) {
  if (state === "Running") return "var(--status-ok)";
  return "var(--text-secondary)";
}

onMounted(() => {
  // Show cached data immediately, then refresh in background
  const cached = loadCache();
  if (cached.length) {
    const aliases = loadAliases();
    distros.value = cached.map((d) => ({ ...d, alias: aliases[d.name] ?? d.alias }));
  }
  refresh();
});
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-2">
      <h2 class="text-lg font-semibold" :style="{ color: 'var(--text-primary)' }">WSL Distros</h2>
      <button @click="refresh" class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg"
        :style="{ color: 'var(--accent)', border: '1px solid var(--border)' }" title="Refresh distro list">
        <RefreshCw :size="12" :class="{ 'animate-spin': loading }" /> Refresh
      </button>
    </div>
    <p class="text-sm mb-4" :style="{ color: 'var(--text-secondary)' }">
      Installed WSL distributions. Click the edit icon to set a friendly alias.
      Rules target the default distro unless a specific distro is set.
    </p>
    <div class="space-y-3">
      <div v-for="d in distros" :key="d.name" class="rounded-lg overflow-hidden"
        :style="{ background: 'var(--bg-secondary)', border: `1px solid ${d.state === 'Running' ? 'var(--accent-dim)' : 'var(--border)'}` }">
        <!-- Header row -->
        <div class="flex items-center gap-3 p-4">
          <Server :size="18" :style="{ color: stateColor(d.state) }" />
          <!-- Name + alias editing -->
          <template v-if="editingAlias === d.name">
            <input v-model="aliasInput" @keydown.enter="saveAlias(d)" @keydown.escape="cancelEditAlias"
              class="w-40 px-2 py-0.5 text-sm rounded outline-none"
              :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--accent)' }"
              placeholder="Alias (e.g. Main Dev)" autofocus />
            <button @click="saveAlias(d)" :style="{ color: 'var(--status-ok)' }" title="Save alias"><Check :size="14" /></button>
            <button @click="cancelEditAlias" :style="{ color: 'var(--text-secondary)' }" title="Cancel"><X :size="14" /></button>
          </template>
          <template v-else>
            <div class="flex items-center gap-2">
              <span class="font-semibold text-sm" :style="{ color: 'var(--text-primary)' }">{{ d.name }}</span>
              <span v-if="d.alias" class="text-xs px-1.5 py-0.5 rounded" :style="{ background: 'var(--accent-dim)', color: 'var(--text-primary)' }">{{ d.alias }}</span>
              <button @click="startEditAlias(d)" class="p-0.5 rounded hover:opacity-80"
                :style="{ color: 'var(--text-secondary)' }" title="Set a friendly alias for this distro"><Edit2 :size="12" /></button>
            </div>
          </template>
          <span v-if="d.default" class="text-[10px] px-1.5 py-0.5 rounded font-medium"
            :style="{ background: 'var(--status-ok)', color: '#000' }">default</span>
          <span class="text-xs font-medium px-1.5 py-0.5 rounded"
            :style="{ background: d.state === 'Running' ? 'rgba(63,185,80,0.15)' : 'rgba(139,148,158,0.15)', color: stateColor(d.state) }">
            {{ d.state }}
          </span>
          <span class="text-xs font-mono px-1.5 py-0.5 rounded" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-secondary)' }"
            title="WSL version">WSL{{ d.version }}</span>
          <span v-if="d.ip" class="text-xs font-mono" :style="{ color: 'var(--accent)' }"
            :title="`IP address: ${d.ip}`">{{ d.ip }}</span>
          <div class="flex-1" />
          <span class="text-xs px-2 py-0.5 rounded" :style="{ background: 'var(--bg-tertiary)', color: 'var(--accent)' }"
            :title="`${ruleCount(d)} port rules target this distro`">{{ ruleCount(d) }} rules</span>
        </div>
      </div>
    </div>
    <div v-if="loadError" class="rounded-lg p-4 mb-4" :style="{ background: 'rgba(248,81,73,0.1)', border: '1px solid var(--status-err)' }">
      <p class="text-sm font-medium mb-1" :style="{ color: 'var(--status-err)' }">Failed to detect WSL distributions</p>
      <p class="text-xs font-mono" :style="{ color: 'var(--text-secondary)' }">{{ loadError }}</p>
    </div>
    <div v-else-if="!distros.length && !loading" class="text-center py-12" :style="{ color: 'var(--text-secondary)' }">
      No WSL distributions found. Install one with <code>wsl --install</code>.
    </div>
  </div>
</template>
