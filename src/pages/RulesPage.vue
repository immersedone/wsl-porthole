<script setup lang="ts">
import { ref, computed, inject, type Ref } from "vue";
import { Plus, Upload, Download } from "lucide-vue-next";
import RuleCard from "../components/RuleCard.vue";
import RuleEditor from "../components/RuleEditor.vue";
import FilterBar, { type FilterState } from "../components/FilterBar.vue";
import type { Rule, StatusInfo } from "../types";
import { useAuditLog } from "../hooks/useAuditLog";

const rules = inject<Ref<Rule[]>>("rules")!;
const refreshRules = inject<() => void>("refreshRules")!;
const { log } = useAuditLog();

const filters = ref<FilterState>({ search: "", direction: "all", source: "all", scope: "all", health: "all", enabled: "all" });
const selectedId = ref<string | null>(null);
const editorRule = ref<Rule | undefined>(undefined);
const showEditor = ref(false);
const showImport = ref(false);
const importText = ref("");

const filtered = computed(() => rules.value.filter((r) => {
  const f = filters.value;
  if (f.search && !r.name.toLowerCase().includes(f.search.toLowerCase())) return false;
  if (f.direction !== "all" && r.direction !== f.direction) return false;
  if (f.source !== "all" && r.source !== f.source) return false;
  if (f.scope === "lan" && !r.lan) return false;
  if (f.scope === "local" && r.lan) return false;
  if (f.health !== "all" && r.health !== f.health) return false;
  if (f.enabled === "enabled" && !r.enabled) return false;
  if (f.enabled === "disabled" && r.enabled) return false;
  return true;
}));

async function handleToggle(id: string) {
  if ("__TAURI__" in window) { const { toggleRule } = await import("../hooks/useTauri"); await toggleRule(id); refreshRules(); }
  else rules.value = rules.value.map((r) => r.id === id ? { ...r, enabled: !r.enabled } : r);
  log("rule.toggle", `Toggled rule ${id}`);
}
async function handleDelete(id: string) {
  if ("__TAURI__" in window) { const { deleteRule } = await import("../hooks/useTauri"); await deleteRule(id); refreshRules(); }
  else rules.value = rules.value.filter((r) => r.id !== id);
  log("rule.delete", `Deleted rule ${id}`);
}
function handleDuplicate(rule: Rule) {
  rules.value = [...rules.value, { ...rule, id: crypto.randomUUID(), name: `${rule.name} (copy)` }];
  log("rule.duplicate", `Duplicated "${rule.name}"`);
}
async function handleSave(partial: Partial<Rule>) {
  if (partial.id) {
    if ("__TAURI__" in window) { const { updateRule } = await import("../hooks/useTauri"); await updateRule(partial as Rule); refreshRules(); }
    else rules.value = rules.value.map((r) => r.id === partial.id ? { ...r, ...partial } as Rule : r);
    log("rule.update", `Updated "${partial.name}"`);
  } else {
    const nr: Rule = { id: crypto.randomUUID(), name: partial.name ?? "Untitled", direction: partial.direction ?? "winToWsl", listenAddr: partial.listenAddr ?? "0.0.0.0", listenPort: partial.listenPort ?? { type: "single", port: 8080 }, connectPort: partial.connectPort ?? { type: "single", port: 8080 }, connectAddr: partial.connectAddr ?? "${WSL_IP}", distro: partial.distro ?? null, lan: partial.lan ?? true, enabled: partial.enabled ?? true, source: partial.source ?? "manual", note: partial.note ?? null };
    if ("__TAURI__" in window) { const { addRule } = await import("../hooks/useTauri"); await addRule({ name: nr.name, direction: nr.direction === "winToWsl" ? "WinToWsl" : "WslToWin", listenAddr: nr.listenAddr, listenPort: nr.listenPort.type === "single" ? nr.listenPort.port! : nr.listenPort.start!, connectPort: nr.connectPort.type === "single" ? nr.connectPort.port! : nr.connectPort.start!, connectAddr: nr.connectAddr, lan: nr.lan }); refreshRules(); }
    else rules.value = [...rules.value, nr];
    log("rule.add", `Added "${nr.name}"`);
  }
  showEditor.value = false; editorRule.value = undefined;
}
async function handleImport() {
  if (!importText.value.trim()) return;
  if ("__TAURI__" in window) { const { importNetshScript } = await import("../hooks/useTauri"); const imp = await importNetshScript(importText.value); refreshRules(); log("rule.import", `Imported ${imp.length} rules`); }
  showImport.value = false; importText.value = "";
}
function handleExport() {
  const json = JSON.stringify({ version: 1, distro: "auto", rules: rules.value }, null, 2);
  const a = document.createElement("a"); a.href = URL.createObjectURL(new Blob([json], { type: "application/json" })); a.download = "wsl-porthole-rules.json"; a.click();
  log("rule.export", "Exported rules as JSON");
}
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-lg font-semibold" :style="{ color: 'var(--text-primary)' }">
        Port Rules <span class="text-sm font-normal ml-2" :style="{ color: 'var(--text-secondary)' }">{{ filtered.length }} of {{ rules.length }}</span>
      </h2>
      <div class="flex items-center gap-2">
        <button @click="showImport = true" class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg" :style="{ color: 'var(--text-secondary)', border: '1px solid var(--border)' }"><Upload :size="12" /> Import</button>
        <button @click="handleExport" class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg" :style="{ color: 'var(--text-secondary)', border: '1px solid var(--border)' }"><Download :size="12" /> Export</button>
        <button @click="editorRule = undefined; showEditor = true" class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg font-medium" :style="{ background: 'var(--accent)', color: 'var(--bg-primary)' }"><Plus :size="12" /> Add Rule</button>
      </div>
    </div>

    <FilterBar v-model="filters" />

    <div class="space-y-1.5">
      <RuleCard v-for="rule in filtered" :key="rule.id" :rule="rule" :selected="selectedId === rule.id"
        @toggle="handleToggle" @edit="(r) => { editorRule = r; showEditor = true }" @delete="handleDelete"
        @duplicate="handleDuplicate" @select="selectedId = rule.id" />
      <div v-if="filtered.length === 0" class="text-center py-12" :style="{ color: 'var(--text-secondary)' }">
        {{ rules.length === 0 ? "No rules yet. Add one or import from a netsh script." : "No rules match the current filters." }}
      </div>
    </div>

    <RuleEditor v-if="showEditor" :rule="editorRule" @save="handleSave" @cancel="showEditor = false; editorRule = undefined" />

    <!-- Import modal -->
    <div v-if="showImport" class="fixed inset-0 z-50 flex items-center justify-center" style="background: rgba(0,0,0,0.6)">
      <div class="w-[560px] rounded-xl p-6 shadow-2xl" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }">
        <h3 class="text-base font-semibold mb-3" :style="{ color: 'var(--text-primary)' }">Import netsh Script</h3>
        <p class="text-xs mb-3" :style="{ color: 'var(--text-secondary)' }">Paste a netsh portproxy script. Hardcoded IPs will be replaced with ${'${WSL_IP}'}.</p>
        <textarea v-model="importText" class="w-full h-48 p-3 text-xs font-mono rounded-lg outline-none resize-none"
          :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border)' }"
          placeholder="netsh interface portproxy add v4tov4 listenport=80 ..." />
        <div class="flex justify-end gap-2 mt-4">
          <button @click="showImport = false; importText = ''" class="px-4 py-1.5 text-sm rounded-lg" :style="{ color: 'var(--text-secondary)', border: '1px solid var(--border)' }">Cancel</button>
          <button @click="handleImport" class="px-4 py-1.5 text-sm rounded-lg font-medium" :style="{ background: 'var(--accent)', color: 'var(--bg-primary)' }">Import</button>
        </div>
      </div>
    </div>
  </div>
</template>
