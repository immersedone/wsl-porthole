<script setup lang="ts">
import { ref, computed } from "vue";
import { AlertTriangle, Save } from "lucide-vue-next";
import { useAuditLog } from "../hooks/useAuditLog";

const { log } = useAuditLog();

interface Entry { key: string; value: string; description: string; section: string }
const entries = ref<Entry[]>([
  { key: "networkingMode", value: "NAT", description: "Network mode (NAT or mirrored)", section: "wsl2" },
  { key: "memory", value: "8GB", description: "Maximum memory allocation", section: "wsl2" },
  { key: "processors", value: "4", description: "Number of logical processors", section: "wsl2" },
  { key: "swap", value: "4GB", description: "Swap file size", section: "wsl2" },
  { key: "localhostForwarding", value: "true", description: "Enable localhost forwarding", section: "wsl2" },
  { key: "dnsTunneling", value: "true", description: "Enable DNS tunneling", section: "experimental" },
  { key: "autoProxy", value: "true", description: "Auto-configure proxy settings", section: "experimental" },
]);

const warnings = computed(() => {
  const w: string[] = [];
  if (entries.value.find((e) => e.key === "networkingMode")?.value.toLowerCase() === "mirrored") {
    w.push("Mirrored mode + full-tunnel VPN may break WSL networking.");
    w.push("Mirrored mode on Windows Server 2025 silently falls back to NAT.");
  }
  return w;
});

const sections = computed(() => {
  const m: Record<string, Entry[]> = {};
  for (const e of entries.value) (m[e.section] ??= []).push(e);
  return m;
});

function save() { log("wslconfig.save", "Saved .wslconfig changes"); }
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-lg font-semibold" :style="{ color: 'var(--text-primary)' }">.wslconfig Inspector</h2>
      <button @click="save" class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg font-medium" :style="{ background: 'var(--accent)', color: 'var(--bg-primary)' }"><Save :size="12" /> Save & Restart WSL</button>
    </div>
    <div v-if="warnings.length" class="rounded-lg p-3 mb-4" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--status-warn)' }">
      <div v-for="(w, i) in warnings" :key="i" class="flex items-start gap-2 text-xs mb-1 last:mb-0" :style="{ color: 'var(--status-warn)' }">
        <AlertTriangle :size="12" class="mt-0.5 shrink-0" /><span>{{ w }}</span>
      </div>
    </div>
    <div v-for="(items, sec) in sections" :key="sec" class="mb-6">
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-2" :style="{ color: 'var(--text-secondary)' }">[{{ sec }}]</h3>
      <div class="space-y-1.5">
        <div v-for="e in items" :key="e.key" class="flex items-center gap-3 px-4 py-2 rounded-lg" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }">
          <span class="text-sm font-mono w-44" :style="{ color: 'var(--accent)' }">{{ e.key }}</span>
          <input v-model="e.value" class="w-32 px-2 py-1 text-sm rounded outline-none" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border)' }" />
          <span class="text-xs flex-1" :style="{ color: 'var(--text-secondary)' }">{{ e.description }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
