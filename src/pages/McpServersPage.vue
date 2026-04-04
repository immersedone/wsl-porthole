<script setup lang="ts">
import { ref, inject, onMounted, type Ref } from "vue";
import { Radio, RefreshCw, Plus, AlertCircle, CheckCircle, Container } from "lucide-vue-next";
import type { McpServerInfo, Rule } from "../types";
import { useAuditLog } from "../hooks/useAuditLog";

const rules = inject<Ref<Rule[]>>("rules")!;
const refreshRules = inject<() => void>("refreshRules")!;
const { log } = useAuditLog();
const servers = ref<McpServerInfo[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);
const lastScan = ref<string | null>(null);

async function refresh() {
  loading.value = true; error.value = null;
  try {
    if (!("__TAURI__" in window)) { loading.value = false; return; }
    const { detectMcpServers } = await import("../hooks/useTauri");
    servers.value = await detectMcpServers();
    lastScan.value = new Date().toLocaleTimeString();
    log("mcp.scan", `Found ${servers.value.length} MCP servers`);
  } catch (e: any) {
    error.value = String(e);
    log("mcp.scan", `Scan failed: ${e}`, "error");
  }
  loading.value = false;
}
onMounted(refresh);

async function addRoute(s: McpServerInfo) {
  try {
    if (!("__TAURI__" in window)) return;
    const { addRule } = await import("../hooks/useTauri");
    await addRule({ name: `MCP: ${s.container_name}`, direction: "WslToWin", listenAddr: "0.0.0.0", listenPort: s.host_port, connectPort: s.port, connectAddr: "${HOST_GW}", lan: false });
    refreshRules();
    log("mcp.add_route", `Added WSL→WIN route for ${s.container_name} on port ${s.host_port}`);
  } catch (e) { log("mcp.add_route", `Failed: ${e}`, "error"); }
}

function hasRoute(port: number) { return rules.value.some((r) => r.source === "mcp" && r.listenPort.type === "single" && r.listenPort.port === port); }
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-2">
      <h2 class="text-lg font-semibold" :style="{ color: 'var(--text-primary)' }">MCP Servers</h2>
      <div class="flex items-center gap-2">
        <span v-if="lastScan" class="text-[10px]" :style="{ color: 'var(--text-secondary)' }">Last scan: {{ lastScan }}</span>
        <button @click="refresh" class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg"
          :style="{ color: 'var(--accent)', border: '1px solid var(--border)' }"
          title="Scan for MCP servers running in Docker on the Windows engine">
          <RefreshCw :size="12" :class="{ 'animate-spin': loading }" /> Scan
        </button>
      </div>
    </div>
    <p class="text-sm mb-4" :style="{ color: 'var(--text-secondary)' }">
      Detects MCP (Model Context Protocol) servers running in Docker on the Windows engine.
      Creates WSL→WIN firewall rules so WSL processes can reach them via the Hyper-V gateway.
    </p>
    <div v-if="error" class="flex items-center gap-2 p-3 rounded-lg mb-4" :style="{ background: 'var(--status-err)', color: '#fff' }">
      <AlertCircle :size="14" /><span class="text-xs">{{ error }}</span>
    </div>
    <div class="space-y-2">
      <div v-for="(s, i) in servers" :key="i" class="flex items-center gap-3 p-3 rounded-lg"
        :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }">
        <Radio :size="14" :style="{ color: 'var(--accent)' }" />
        <span class="font-medium text-sm" :style="{ color: 'var(--text-primary)' }">{{ s.container_name }}</span>
        <span class="text-xs font-mono truncate max-w-[240px]" :style="{ color: 'var(--text-secondary)' }" :title="s.image">{{ s.image }}</span>
        <span class="text-xs font-mono px-1.5 py-0.5 rounded" :style="{ background: 'var(--bg-tertiary)', color: 'var(--accent)' }"
          :title="`Container port: ${s.port}, Host port: ${s.host_port}`">:{{ s.host_port }}</span>
        <span class="text-[10px] px-1.5 py-0.5 rounded cursor-help" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-secondary)' }"
          :title="s.detection_reason">{{ s.detection_reason.split(' ')[0] }}</span>
        <div class="flex-1" />
        <span v-if="hasRoute(s.host_port)" class="flex items-center gap-1 text-xs px-2 py-0.5 rounded" :style="{ background: 'var(--status-ok)', color: '#000' }">
          <CheckCircle :size="10" /> routed
        </span>
        <button v-else @click="addRoute(s)" class="flex items-center gap-1 text-xs px-2 py-1 rounded"
          :style="{ background: 'var(--accent-dim)', color: 'var(--text-primary)' }"
          :title="`Create a WSL→WIN firewall rule so WSL can reach ${s.container_name} on port ${s.host_port}`">
          <Plus :size="10" /> Allow in WSL
        </button>
      </div>
    </div>
    <div v-if="!servers.length && !loading" class="text-center py-12">
      <Container :size="32" class="mx-auto mb-3" :style="{ color: 'var(--text-secondary)', opacity: 0.5 }" />
      <p :style="{ color: 'var(--text-secondary)' }">No MCP servers detected.</p>
      <p class="text-xs mt-1" :style="{ color: 'var(--text-secondary)' }">Ensure MCP server containers are running on the Windows Docker engine.</p>
    </div>
  </div>
</template>
