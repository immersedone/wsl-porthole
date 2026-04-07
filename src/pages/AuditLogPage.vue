<script setup lang="ts">
import { ref, computed } from "vue";
import { Download, Trash2 } from "lucide-vue-next";
import { useAuditLog } from "../hooks/useAuditLog";
import type { AuditEntry } from "../types";

const { entries, exportLog, clear } = useAuditLog();
const levelFilter = ref<"all" | "info" | "warn" | "error">("all");
const searchText = ref("");

const filtered = computed(() => {
  const result = entries.value.filter((e) => {
    if (levelFilter.value !== "all" && e.level !== levelFilter.value) return false;
    if (searchText.value && !e.event.includes(searchText.value) && !e.detail.includes(searchText.value)) return false;
    return true;
  });
  // Display newest first without mutating the source array
  return result.slice().reverse();
});

function doExport() {
  const a = document.createElement("a");
  a.href = URL.createObjectURL(new Blob([exportLog()], { type: "text/plain" }));
  a.download = `wsl-porthole-audit-${new Date().toISOString().split("T")[0]}.log`;
  a.click();
}

const levelColor = (l: AuditEntry["level"]) => l === "error" ? "var(--status-err)" : l === "warn" ? "var(--status-warn)" : "var(--text-secondary)";
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-lg font-semibold" :style="{ color: 'var(--text-primary)' }">Audit Log <span class="text-sm font-normal ml-2" :style="{ color: 'var(--text-secondary)' }">{{ filtered.length }} entries</span></h2>
      <div class="flex items-center gap-2">
        <button @click="doExport" class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg" :style="{ color: 'var(--text-secondary)', border: '1px solid var(--border)' }"><Download :size="12" /> Export</button>
        <button @click="clear" class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg" :style="{ color: 'var(--status-err)', border: '1px solid var(--border)' }"><Trash2 :size="12" /> Clear</button>
      </div>
    </div>
    <div class="flex items-center gap-3 mb-4">
      <input v-model="searchText" placeholder="Search events..." class="flex-1 px-3 py-1.5 text-sm rounded-lg outline-none"
        :style="{ background: 'var(--bg-secondary)', color: 'var(--text-primary)', border: '1px solid var(--border)' }" />
      <select v-model="levelFilter" class="text-xs px-2 py-1.5 rounded"
        :style="{ background: 'var(--bg-secondary)', color: 'var(--text-primary)', border: '1px solid var(--border)' }">
        <option value="all">All levels</option><option value="info">Info</option><option value="warn">Warning</option><option value="error">Error</option>
      </select>
    </div>
    <div class="space-y-0.5">
      <div v-for="(e, i) in filtered" :key="e.id ?? i" class="flex items-center gap-3 px-3 py-1.5 rounded text-xs font-mono"
        :style="{ background: i % 2 === 0 ? 'var(--bg-secondary)' : 'transparent' }">
        <span :style="{ color: 'var(--text-secondary)' }">{{ new Date(e.timestamp).toLocaleTimeString() }}</span>
        <span class="w-10 text-center uppercase text-[10px] font-bold" :style="{ color: levelColor(e.level) }">{{ e.level }}</span>
        <span :style="{ color: 'var(--accent)' }">{{ e.event }}</span>
        <span :style="{ color: 'var(--text-secondary)' }">{{ e.detail }}</span>
      </div>
    </div>
    <div v-if="!filtered.length" class="text-center py-12" :style="{ color: 'var(--text-secondary)' }">
      {{ entries.length === 0 ? "No audit entries yet." : "No entries match the filter." }}
    </div>
  </div>
</template>
