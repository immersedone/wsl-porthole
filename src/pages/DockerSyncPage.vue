<script setup lang="ts">
import { ref, inject, onMounted, onUnmounted, computed, type Ref } from "vue";
import { Container, RefreshCw, Plus, AlertCircle } from "lucide-vue-next";
import type { ContainerSummary, Rule } from "../types";
import { useAuditLog } from "../hooks/useAuditLog";

const rules = inject<Ref<Rule[]>>("rules")!;
const refreshRules = inject<() => void>("refreshRules")!;
const { log } = useAuditLog();
const containers = ref<ContainerSummary[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);
const engine = ref<"wsl" | "windows">("wsl");

async function refresh() {
  loading.value = true; error.value = null;
  try {
    if ("__TAURI__" in window) { const { listDockerContainers } = await import("../hooks/useTauri"); containers.value = await listDockerContainers(engine.value); }
    else containers.value = [
      { id: "abc123", name: "postgres-dev", image: "postgres:16", status: "Up 2 hours", ports: [{ host_port: 5432, container_port: 5432, protocol: "tcp" }], compose_project: "myapp" },
      { id: "def456", name: "redis-cache", image: "redis:7-alpine", status: "Up 2 hours", ports: [{ host_port: 6379, container_port: 6379, protocol: "tcp" }], compose_project: "myapp" },
      { id: "ghi789", name: "nginx-proxy", image: "nginx:latest", status: "Up 5 minutes", ports: [{ host_port: 80, container_port: 80, protocol: "tcp" }, { host_port: 443, container_port: 443, protocol: "tcp" }], compose_project: null },
    ];
  } catch (e: any) { error.value = String(e); }
  loading.value = false;
}

onMounted(refresh);
const iv = setInterval(refresh, 30000);
onUnmounted(() => clearInterval(iv));

const grouped = computed(() => {
  const m: Record<string, ContainerSummary[]> = {};
  for (const c of containers.value) { const k = c.compose_project ?? "(standalone)"; (m[k] ??= []).push(c); }
  return m;
});

async function addPort(c: ContainerSummary, p: ContainerSummary["ports"][0]) {
  if (rules.value.some((r) => r.listenPort.type === "single" && r.listenPort.port === p.host_port)) return;
  try {
    if ("__TAURI__" in window) {
      const { addRule } = await import("../hooks/useTauri");
      await addRule({ name: `${c.name}:${p.container_port}`, direction: "WinToWsl", listenAddr: "0.0.0.0", listenPort: p.host_port, connectPort: p.container_port, connectAddr: "${WSL_IP}", lan: true });
      refreshRules();
    } else {
      rules.value.push({ id: crypto.randomUUID(), name: `${c.name}:${p.container_port}`, direction: "winToWsl", listenAddr: "0.0.0.0", listenPort: { type: "single", port: p.host_port }, connectPort: { type: "single", port: p.container_port }, connectAddr: "${WSL_IP}", distro: null, lan: true, enabled: true, source: "docker", note: `From ${c.image}`, health: "unknown" });
    }
    log("docker.add_rule", `Added rule for ${c.name}:${p.container_port}`);
  } catch (e) { log("docker.add_rule", `Failed: ${e}`, "error"); }
}
function hasRule(port: number) { return rules.value.some((r) => r.listenPort.type === "single" && r.listenPort.port === port); }
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-lg font-semibold" :style="{ color: 'var(--text-primary)' }">Docker Sync</h2>
      <div class="flex items-center gap-2">
        <select v-model="engine" @change="refresh" class="text-xs px-2 py-1 rounded" :style="{ background: 'var(--bg-secondary)', color: 'var(--text-primary)', border: '1px solid var(--border)' }">
          <option value="wsl">WSL Engine</option><option value="windows">Windows Engine</option>
        </select>
        <button @click="refresh" class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg" :style="{ color: 'var(--accent)', border: '1px solid var(--border)' }"><RefreshCw :size="12" :class="{ 'animate-spin': loading }" /> Refresh</button>
      </div>
    </div>
    <div v-if="error" class="flex items-center gap-2 p-3 rounded-lg mb-4" :style="{ background: 'var(--status-err)', color: '#fff' }"><AlertCircle :size="14" /><span class="text-xs">{{ error }}</span></div>
    <div v-for="(ctrs, project) in grouped" :key="project" class="mb-6">
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-2" :style="{ color: 'var(--text-secondary)' }">{{ project === "(standalone)" ? "Standalone" : `Project: ${project}` }}</h3>
      <div class="space-y-2">
        <div v-for="c in ctrs" :key="c.id" class="rounded-lg p-3" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }">
          <div class="flex items-center gap-3 mb-2">
            <Container :size="14" :style="{ color: 'var(--accent)' }" />
            <span class="font-medium text-sm" :style="{ color: 'var(--text-primary)' }">{{ c.name }}</span>
            <span class="text-xs font-mono" :style="{ color: 'var(--text-secondary)' }">{{ c.image }}</span>
            <span class="text-xs" :style="{ color: 'var(--status-ok)' }">{{ c.status }}</span>
          </div>
          <div v-if="c.ports.length" class="flex flex-wrap gap-2">
            <div v-for="(p, i) in c.ports" :key="i" class="flex items-center gap-1.5 text-xs px-2 py-1 rounded" :style="{ background: 'var(--bg-tertiary)' }">
              <span class="font-mono" :style="{ color: 'var(--accent)' }">{{ p.host_port }}:{{ p.container_port }}/{{ p.protocol }}</span>
              <span v-if="hasRule(p.host_port)" class="text-[10px] px-1 rounded" :style="{ background: 'var(--status-ok)', color: '#000' }">forwarded</span>
              <button v-else @click="addPort(c, p)" class="flex items-center gap-0.5 text-[10px] px-1 rounded hover:opacity-80" :style="{ background: 'var(--accent-dim)', color: 'var(--text-primary)' }"><Plus :size="10" /> forward</button>
            </div>
          </div>
          <span v-else class="text-xs" :style="{ color: 'var(--text-secondary)' }">No exposed ports</span>
        </div>
      </div>
    </div>
    <div v-if="!containers.length && !loading" class="text-center py-12" :style="{ color: 'var(--text-secondary)' }">No running containers found.</div>
  </div>
</template>
