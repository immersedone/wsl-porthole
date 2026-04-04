<script setup lang="ts">
import { ref, onMounted } from "vue";
import { Shield, RefreshCw } from "lucide-vue-next";

const fwRules = ref<string[]>([]);
const loading = ref(false);

async function refresh() {
  loading.value = true;
  try {
    if (!("__TAURI__" in window)) { loading.value = false; return; }
    const { getFirewallRules } = await import("../hooks/useTauri");
    fwRules.value = await getFirewallRules();
  } catch (e) { console.error(e); }
  loading.value = false;
}
onMounted(refresh);
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-lg font-semibold" :style="{ color: 'var(--text-primary)' }">Firewall Rules</h2>
      <button @click="refresh" class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg" :style="{ color: 'var(--accent)', border: '1px solid var(--border)' }"><RefreshCw :size="12" :class="{ 'animate-spin': loading }" /> Refresh</button>
    </div>
    <p class="text-sm mb-4" :style="{ color: 'var(--text-secondary)' }">Windows Defender Firewall rules managed by WSL PortHole.</p>
    <div class="space-y-1.5">
      <div v-for="(name, i) in fwRules" :key="i" class="flex items-center gap-3 px-4 py-2 rounded-lg" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }">
        <Shield :size="14" :style="{ color: name.includes('WSL→WIN') ? 'var(--status-warn)' : 'var(--accent)' }" />
        <span class="text-sm font-mono" :style="{ color: 'var(--text-primary)' }">{{ name }}</span>
        <span class="text-[10px] px-1.5 py-0.5 rounded" :style="{ background: 'var(--bg-tertiary)', color: name.includes('WSL→WIN') ? 'var(--status-warn)' : 'var(--accent)' }">{{ name.includes("WSL→WIN") ? "WSL interface" : "Inbound TCP" }}</span>
      </div>
    </div>
    <div v-if="!fwRules.length && !loading" class="text-center py-12" :style="{ color: 'var(--text-secondary)' }">No WSL PortHole firewall rules found.</div>
  </div>
</template>
