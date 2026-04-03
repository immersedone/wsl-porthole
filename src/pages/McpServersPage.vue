<script setup lang="ts">
import { ref, inject, onMounted, type Ref } from "vue";
import { Radio, RefreshCw, Plus, AlertCircle } from "lucide-vue-next";
import type { McpServerInfo, Rule } from "../types";
import { useAuditLog } from "../hooks/useAuditLog";

const rules = inject<Ref<Rule[]>>("rules")!;
const { log } = useAuditLog();
const servers = ref<McpServerInfo[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);

async function refresh() {
  loading.value = true; error.value = null;
  try {
    if ("__TAURI__" in window) { const { detectMcpServers } = await import("../hooks/useTauri"); servers.value = await detectMcpServers(); }
    else servers.value = [
      { container_name: "mcp-server-github", image: "ghcr.io/modelcontextprotocol/mcp-server-github", port: 3000, host_port: 3000, detection_reason: "ImageName" },
      { container_name: "mcp-server-filesystem", image: "ghcr.io/modelcontextprotocol/mcp-server-filesystem", port: 3001, host_port: 3001, detection_reason: "ImageName" },
    ];
  } catch (e: any) { error.value = String(e); }
  loading.value = false;
}
onMounted(refresh);

function addRoute(s: McpServerInfo) {
  rules.value.push({ id: crypto.randomUUID(), name: `MCP: ${s.container_name}`, direction: "wslToWin", listenAddr: "0.0.0.0", listenPort: { type: "single", port: s.host_port }, connectPort: { type: "single", port: s.port }, connectAddr: "${HOST_GW}", distro: null, lan: false, enabled: true, source: "mcp", note: `MCP from ${s.image}` });
  log("mcp.add_route", `Added WSL→WIN route for ${s.container_name}`);
}
function hasRoute(port: number) { return rules.value.some((r) => r.source === "mcp" && r.listenPort.type === "single" && r.listenPort.port === port); }
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-lg font-semibold" :style="{ color: 'var(--text-primary)' }">MCP Servers</h2>
      <button @click="refresh" class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg" :style="{ color: 'var(--accent)', border: '1px solid var(--border)' }"><RefreshCw :size="12" :class="{ 'animate-spin': loading }" /> Scan</button>
    </div>
    <p class="text-sm mb-4" :style="{ color: 'var(--text-secondary)' }">Detects MCP servers running in Docker on the Windows engine. Creates WSL→WIN firewall rules.</p>
    <div v-if="error" class="flex items-center gap-2 p-3 rounded-lg mb-4" :style="{ background: 'var(--status-err)', color: '#fff' }"><AlertCircle :size="14" /><span class="text-xs">{{ error }}</span></div>
    <div class="space-y-2">
      <div v-for="(s, i) in servers" :key="i" class="flex items-center gap-3 p-3 rounded-lg" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }">
        <Radio :size="14" :style="{ color: 'var(--accent)' }" />
        <span class="font-medium text-sm" :style="{ color: 'var(--text-primary)' }">{{ s.container_name }}</span>
        <span class="text-xs font-mono" :style="{ color: 'var(--text-secondary)' }">{{ s.image }}</span>
        <span class="text-xs font-mono px-1.5 py-0.5 rounded" :style="{ background: 'var(--bg-tertiary)', color: 'var(--accent)' }">:{{ s.host_port }}</span>
        <span class="text-[10px] px-1.5 py-0.5 rounded" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-secondary)' }">{{ s.detection_reason }}</span>
        <div class="flex-1" />
        <span v-if="hasRoute(s.host_port)" class="text-xs px-2 py-0.5 rounded" :style="{ background: 'var(--status-ok)', color: '#000' }">routed</span>
        <button v-else @click="addRoute(s)" class="flex items-center gap-1 text-xs px-2 py-1 rounded" :style="{ background: 'var(--accent-dim)', color: 'var(--text-primary)' }"><Plus :size="10" /> Allow in WSL</button>
      </div>
    </div>
    <div v-if="!servers.length && !loading" class="text-center py-12" :style="{ color: 'var(--text-secondary)' }">No MCP servers detected.</div>
  </div>
</template>
