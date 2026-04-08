<script setup lang="ts">
import { ref, computed, inject, onMounted, type Ref } from "vue";
import { Plus, Upload, Download, FileText, Package, FolderOpen, X } from "lucide-vue-next";
import RuleCard from "../components/RuleCard.vue";
import QrCode from "../components/QrCode.vue";
import RuleEditor from "../components/RuleEditor.vue";
import FilterBar, { type FilterState } from "../components/FilterBar.vue";
import type { Rule } from "../types";
import { useAuditLog } from "../hooks/useAuditLog";
import { useToast } from "../hooks/useToast";

const rules = inject<Ref<Rule[]>>("rules")!;
const refreshRules = inject<() => void>("refreshRules")!;
const { log } = useAuditLog();
const { show: showToast } = useToast();

const filters = ref<FilterState>({ search: "", direction: "all", source: "all", scope: "all", health: "all", enabled: "all" });
const selectedId = ref<string | null>(null);
const editorRule = ref<Rule | undefined>(undefined);
const showEditor = ref(false);
const showImport = ref(false);
const showImportBundle = ref(false);
const importText = ref("");
const importBundleText = ref("");
const importBundleParsed = ref<any>(null);
const importMode = ref<"merge" | "replace">("merge");
const qrUrl = ref<string | null>(null);
const viewMode = ref<"flat" | "grouped">("flat");
const groups = ref<any[]>([]);

async function loadGroups() {
  if (!("__TAURI__" in window) && !("__TAURI_INTERNALS__" in window)) return;
  try {
    const { getSettings } = await import("../hooks/useTauri");
    const s = await getSettings();
    groups.value = s.groups ?? [];
  } catch {}
}

// Load groups on component mount for grouped view
onMounted(loadGroups);


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

interface GroupedRules { name: string; id: string | null; rules: Rule[] }
const groupedFiltered = computed((): GroupedRules[] => {
  if (viewMode.value === "flat") return [{ name: "", id: null, rules: filtered.value }];
  const result: GroupedRules[] = [];
  const assigned = new Set<string>();
  for (const g of groups.value) {
    const grpRules = filtered.value.filter(r => g.ruleIds?.includes(r.id));
    if (grpRules.length) {
      result.push({ name: g.name, id: g.id, rules: grpRules });
      grpRules.forEach(r => assigned.add(r.id));
    }
  }
  const ungrouped = filtered.value.filter(r => !assigned.has(r.id));
  if (ungrouped.length) result.push({ name: "Ungrouped", id: null, rules: ungrouped });
  return result;
});

async function handleToggle(id: string) {
  try {
    if ("__TAURI__" in window) { const { toggleRule } = await import("../hooks/useTauri"); await toggleRule(id); refreshRules(); }
    else rules.value = rules.value.map((r) => r.id === id ? { ...r, enabled: !r.enabled } : r);
    log("rule.toggle", `Toggled rule ${id}`);
    showToast("Rule toggled", "success");
  } catch (e) { log("rule.toggle", `Failed: ${e}`, "error"); showToast(`Toggle failed: ${e}`, "error"); }
}
async function handleDelete(id: string) {
  try {
    if ("__TAURI__" in window) { const { deleteRule } = await import("../hooks/useTauri"); await deleteRule(id); refreshRules(); }
    else rules.value = rules.value.filter((r) => r.id !== id);
    log("rule.delete", `Deleted rule ${id}`);
    showToast("Rule deleted", "success");
  } catch (e) { log("rule.delete", `Failed: ${e}`, "error"); showToast(`Delete failed: ${e}`, "error"); }
}
async function handleDuplicate(rule: Rule) {
  try {
    const dup = { ...rule, id: crypto.randomUUID(), name: `${rule.name} (copy)`, health: "unknown" as const };
    if ("__TAURI__" in window) {
      const { addRule } = await import("../hooks/useTauri");
      const lp = dup.listenPort.type === "single" ? dup.listenPort.port! : dup.listenPort.start!;
      const cp = dup.connectPort.type === "single" ? dup.connectPort.port! : dup.connectPort.start!;
      await addRule({ name: dup.name, direction: dup.direction === "winToWsl" ? "WinToWsl" : "WslToWin", listenAddr: dup.listenAddr, listenPort: lp, connectPort: cp, connectAddr: dup.connectAddr, lan: dup.lan, distro: dup.distro });
      refreshRules();
    } else {
      rules.value = [...rules.value, dup];
    }
    log("rule.duplicate", `Duplicated "${rule.name}"`);
    showToast(`Duplicated "${rule.name}"`, "success");
  } catch (e) { log("rule.duplicate", `Failed: ${e}`, "error"); showToast(`Duplicate failed: ${e}`, "error"); }
}
async function handleSave(partial: Partial<Rule>) {
  try {
    let ruleId = partial.id;
    if (partial.id) {
      if ("__TAURI__" in window) { const { updateRule } = await import("../hooks/useTauri"); await updateRule(partial as Rule); refreshRules(); }
      else rules.value = rules.value.map((r) => r.id === partial.id ? { ...r, ...partial } as Rule : r);
      log("rule.update", `Updated "${partial.name}"`);
      showToast(`Updated "${partial.name}"`, "success");
    } else {
      ruleId = crypto.randomUUID();
      const nr: Rule = { id: ruleId, name: partial.name ?? "Untitled", direction: partial.direction ?? "winToWsl", listenAddr: partial.listenAddr ?? "0.0.0.0", listenPort: partial.listenPort ?? { type: "single", port: 8080 }, connectPort: partial.connectPort ?? { type: "single", port: 8080 }, connectAddr: partial.connectAddr ?? "${WSL_IP}", distro: partial.distro ?? null, lan: partial.lan ?? true, enabled: partial.enabled ?? true, source: partial.source ?? "manual", note: partial.note ?? null, health: "unknown" };
      if ("__TAURI__" in window) {
        const { addRule } = await import("../hooks/useTauri");
        const lp = nr.listenPort.type === "single" ? nr.listenPort.port! : nr.listenPort.start!;
        const cp = nr.connectPort.type === "single" ? nr.connectPort.port! : nr.connectPort.start!;
        await addRule({ name: nr.name, direction: nr.direction === "winToWsl" ? "WinToWsl" : "WslToWin", listenAddr: nr.listenAddr, listenPort: lp, connectPort: cp, connectAddr: nr.connectAddr, lan: nr.lan, distro: nr.distro });
        refreshRules();
      } else {
        rules.value = [...rules.value, nr];
      }
      log("rule.add", `Added "${nr.name}"`);
      showToast(`Added "${nr.name}"`, "success");
    }
    // Persist group membership
    if (ruleId && ("__TAURI__" in window || "__TAURI_INTERNALS__" in window)) {
      const selectedGroupId = partial.group ?? null;
      const { getSettings, saveSettings } = await import("../hooks/useTauri");
      const s = await getSettings();
      const grps = s.groups ?? [];
      let changed = false;
      for (const g of grps) {
        if (!g.ruleIds) g.ruleIds = [];
        if (g.id === selectedGroupId) {
          if (!g.ruleIds.includes(ruleId)) { g.ruleIds.push(ruleId); changed = true; }
        } else {
          const idx = g.ruleIds.indexOf(ruleId);
          if (idx !== -1) { g.ruleIds.splice(idx, 1); changed = true; }
        }
      }
      if (changed) { s.groups = grps; await saveSettings(s); loadGroups(); }
    }
  } catch (e) { log("rule.save", `Failed: ${e}`, "error"); showToast(`Save failed: ${e}`, "error"); }
  showEditor.value = false; editorRule.value = undefined;
}
async function handleImport() {
  if (!importText.value.trim()) return;
  try {
    if ("__TAURI__" in window) { const { importNetshScript } = await import("../hooks/useTauri"); const imp = await importNetshScript(importText.value); refreshRules(); log("rule.import", `Imported ${imp.length} rules`); showToast(`Imported ${imp.length} rules`, "success"); }
  } catch (e) { log("rule.import", `Failed: ${e}`, "error"); showToast(`Import failed: ${e}`, "error"); }
  showImport.value = false; importText.value = "";
}
function handleExport() {
  const json = JSON.stringify({ version: 1, distro: "auto", rules: rules.value }, null, 2);
  const a = document.createElement("a"); a.href = URL.createObjectURL(new Blob([json], { type: "application/json" })); a.download = "wsl-porthole-rules.json"; a.click();
  log("rule.export", "Exported rules as JSON");
  showToast("Rules exported as JSON", "success");
}
async function handleHealthCheck(id: string) {
  try {
    if ("__TAURI__" in window || "__TAURI_INTERNALS__" in window) {
      const { checkHealth } = await import("../hooks/useTauri");
      const results = await checkHealth();
      const result = results.find((r: any) => r.ruleId === id);
      if (result) {
        const rule = rules.value.find(r => r.id === id);
        if (rule) rule.health = result.status as any;
        showToast(`Health: ${result.reachable}/${result.total} ports reachable`, result.status === "ok" ? "success" : "warn");
      }
    }
  } catch (e) { showToast(`Health check failed: ${e}`, "error"); }
}

async function handleExportPs1() {
  try {
    if (!("__TAURI__" in window)) return;
    const { exportNetshScript } = await import("../hooks/useTauri");
    const script = await exportNetshScript();
    const a = document.createElement("a"); a.href = URL.createObjectURL(new Blob([script], { type: "text/plain" })); a.download = "wsl-porthole-rules.ps1"; a.click();
    log("rule.export", "Exported rules as .ps1 script");
  } catch (e) { log("rule.export", `Failed: ${e}`, "error"); }
}

async function handleExportBundle() {
  try {
    let groups: any[] = [];
    if ("__TAURI__" in window || "__TAURI_INTERNALS__" in window) {
      const { getSettings } = await import("../hooks/useTauri");
      const s = await getSettings();
      groups = s.groups ?? [];
    }
    const bundle = {
      format: "wsl-porthole-bundle",
      version: 2,
      exported: new Date().toISOString(),
      rules: rules.value,
      groups,
    };
    const json = JSON.stringify(bundle, null, 2);
    const a = document.createElement("a");
    a.href = URL.createObjectURL(new Blob([json], { type: "application/json" }));
    a.download = `wsl-porthole-bundle-${new Date().toISOString().slice(0, 10)}.json`;
    a.click();
    log("bundle.export", `Exported ${rules.value.length} rules + ${groups.length} groups`);
    showToast(`Exported ${rules.value.length} rules and ${groups.length} groups`, "success");
  } catch (e) { log("bundle.export", `Failed: ${e}`, "error"); showToast(`Export failed: ${e}`, "error"); }
}

function parseImportBundle() {
  try {
    const data = JSON.parse(importBundleText.value);
    // Support both old format (version 1, rules only) and new bundle format
    if (data.format === "wsl-porthole-bundle" || (data.rules && Array.isArray(data.rules))) {
      importBundleParsed.value = {
        rules: data.rules ?? [],
        groups: data.groups ?? [],
        format: data.format ?? "legacy",
      };
    } else {
      importBundleParsed.value = null;
      showToast("Invalid bundle format", "error");
    }
  } catch {
    importBundleParsed.value = null;
    showToast("Invalid JSON", "error");
  }
}

async function handleImportBundle() {
  if (!importBundleParsed.value) return;
  const { rules: importedRules, groups: importedGroups } = importBundleParsed.value;
  try {
    if ("__TAURI__" in window || "__TAURI_INTERNALS__" in window) {
      const { getRules, saveRules, getSettings, saveSettings } = await import("../hooks/useTauri");

      // Handle rules
      if (importMode.value === "replace") {
        await saveRules(importedRules);
      } else {
        const existing = await getRules();
        const existingPorts = new Set(existing.map((r: Rule) => `${r.listenPort.type === "single" ? r.listenPort.port : r.listenPort.start}-${r.direction}`));
        const newRules = importedRules.filter((r: Rule) => {
          const key = `${r.listenPort.type === "single" ? r.listenPort.port : r.listenPort.start}-${r.direction}`;
          return !existingPorts.has(key);
        });
        // Give new IDs to avoid conflicts
        for (const r of newRules) r.id = crypto.randomUUID();
        await saveRules([...existing, ...newRules]);
      }
      refreshRules();

      // Handle groups
      if (importedGroups.length) {
        const s = await getSettings();
        if (importMode.value === "replace") {
          s.groups = importedGroups;
        } else {
          const existingNames = new Set((s.groups ?? []).map((g: any) => g.name));
          const newGroups = importedGroups.filter((g: any) => !existingNames.has(g.name));
          for (const g of newGroups) g.id = crypto.randomUUID();
          s.groups = [...(s.groups ?? []), ...newGroups];
        }
        await saveSettings(s);
      }

      const msg = importMode.value === "replace"
        ? `Replaced with ${importedRules.length} rules and ${importedGroups.length} groups`
        : `Merged ${importedRules.length} rules and ${importedGroups.length} groups`;
      log("bundle.import", msg);
      showToast(msg, "success");
    }
  } catch (e) { log("bundle.import", `Failed: ${e}`, "error"); showToast(`Import failed: ${e}`, "error"); }
  showImportBundle.value = false;
  importBundleText.value = "";
  importBundleParsed.value = null;
}
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-lg font-semibold" :style="{ color: 'var(--text-primary)' }">
        Port Rules <span class="text-sm font-normal ml-2" :style="{ color: 'var(--text-secondary)' }">{{ filtered.length }} of {{ rules.length }}</span>
      </h2>
      <div class="flex items-center gap-2">
        <button @click="showImportBundle = true" class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg" :style="{ color: 'var(--text-secondary)', border: '1px solid var(--border)' }" title="Import rules + groups from a bundle file"><Upload :size="12" /> Import</button>
        <button @click="handleExportBundle" class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg" :style="{ color: 'var(--accent)', border: '1px solid var(--accent-dim)' }" title="Export all rules + groups as a transferable bundle"><Package :size="12" /> Export Bundle</button>
        <button @click="handleExportPs1" class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg" :style="{ color: 'var(--text-secondary)', border: '1px solid var(--border)' }" title="Export rules as a PowerShell netsh script"><FileText :size="12" /> .ps1</button>
        <button @click="editorRule = undefined; showEditor = true" class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg font-medium" :style="{ background: 'var(--accent)', color: 'var(--bg-primary)' }" title="Create a new port forwarding rule"><Plus :size="12" /> Add Rule</button>
      </div>
    </div>

    <div class="flex items-center gap-3 mb-4">
      <div class="flex-1"><FilterBar v-model="filters" /></div>
      <select v-model="viewMode" class="text-xs px-2 py-1 rounded shrink-0 h-8"
        :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border)' }"
        title="Switch between flat list and grouped view">
        <option value="flat">Flat view</option>
        <option value="grouped">By group</option>
      </select>
    </div>

    <div class="space-y-1.5">
      <template v-for="group in groupedFiltered" :key="group.id ?? 'ungrouped'">
        <!-- Group header (only in grouped mode) -->
        <div v-if="viewMode === 'grouped' && group.name" class="flex items-center gap-2 pt-3 pb-1 px-1">
          <FolderOpen :size="14" :style="{ color: 'var(--accent)' }" />
          <span class="text-xs font-semibold uppercase tracking-wider" :style="{ color: 'var(--accent)' }">{{ group.name }}</span>
          <span class="text-[10px]" :style="{ color: 'var(--text-secondary)' }">{{ group.rules.length }} rules</span>
          <div class="flex-1 border-b" :style="{ borderColor: 'var(--border)' }" />
        </div>
        <RuleCard v-for="rule in group.rules" :key="rule.id" :rule="rule" :selected="selectedId === rule.id"
          @toggle="handleToggle" @edit="(r) => { editorRule = r; showEditor = true }" @delete="handleDelete"
          @duplicate="handleDuplicate" @select="selectedId = rule.id" @qr="(url) => qrUrl = url"
          @health-check="handleHealthCheck" />
      </template>
      <div v-if="filtered.length === 0" class="text-center py-12" :style="{ color: 'var(--text-secondary)' }">
        {{ rules.length === 0 ? "No rules yet. Add one or import from a netsh script." : "No rules match the current filters." }}
      </div>
    </div>

    <RuleEditor v-if="showEditor" :rule="editorRule" :groups="groups" @save="handleSave" @cancel="showEditor = false; editorRule = undefined" />

    <!-- Import Bundle modal -->
    <div v-if="showImportBundle" class="fixed inset-0 z-50 flex items-center justify-center" style="background: rgba(0,0,0,0.6)">
      <div class="w-[600px] rounded-xl p-6 shadow-2xl" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }">
        <h3 class="text-base font-semibold mb-2" :style="{ color: 'var(--text-primary)' }">Import Rules & Groups</h3>
        <p class="text-xs mb-3" :style="{ color: 'var(--text-secondary)' }">
          Paste a WSL PortHole bundle JSON, or a netsh portproxy script.
        </p>

        <!-- Tab selector -->
        <div class="flex gap-2 mb-3">
          <button @click="showImport = false" class="text-xs px-3 py-1 rounded-lg"
            :style="{ background: !showImport ? 'var(--accent-dim)' : 'var(--bg-tertiary)', color: 'var(--text-primary)' }">Bundle (JSON)</button>
          <button @click="showImport = true" class="text-xs px-3 py-1 rounded-lg"
            :style="{ background: showImport ? 'var(--accent-dim)' : 'var(--bg-tertiary)', color: 'var(--text-primary)' }">netsh Script</button>
        </div>

        <!-- Bundle JSON tab -->
        <template v-if="!showImport">
          <textarea v-model="importBundleText" @input="parseImportBundle" class="w-full h-40 p-3 text-xs font-mono rounded-lg outline-none resize-none"
            :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border)' }"
            placeholder='Paste bundle JSON here (exported via "Export Bundle")...' />
          <div v-if="importBundleParsed" class="mt-3 p-3 rounded-lg text-xs" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-secondary)' }">
            <div class="flex items-center gap-4 mb-2">
              <span><strong :style="{ color: 'var(--accent)' }">{{ importBundleParsed.rules.length }}</strong> rules</span>
              <span><strong :style="{ color: 'var(--accent)' }">{{ importBundleParsed.groups.length }}</strong> groups</span>
            </div>
            <div class="flex items-center gap-3">
              <label class="flex items-center gap-1.5 cursor-pointer">
                <input type="radio" v-model="importMode" value="merge" class="accent-[var(--accent)]" />
                <span>Merge (add new, skip duplicates)</span>
              </label>
              <label class="flex items-center gap-1.5 cursor-pointer">
                <input type="radio" v-model="importMode" value="replace" class="accent-[var(--accent)]" />
                <span :style="{ color: 'var(--status-warn)' }">Replace all existing</span>
              </label>
            </div>
          </div>
          <div class="flex justify-end gap-2 mt-4">
            <button @click="showImportBundle = false; importBundleText = ''; importBundleParsed = null" class="px-4 py-1.5 text-sm rounded-lg" :style="{ color: 'var(--text-secondary)', border: '1px solid var(--border)' }">Cancel</button>
            <button @click="handleImportBundle" :disabled="!importBundleParsed" class="px-4 py-1.5 text-sm rounded-lg font-medium"
              :style="{ background: importBundleParsed ? 'var(--accent)' : 'var(--accent-dim)', color: 'var(--bg-primary)' }">Import Bundle</button>
          </div>
        </template>

        <!-- netsh script tab -->
        <template v-else>
          <textarea v-model="importText" class="w-full h-40 p-3 text-xs font-mono rounded-lg outline-none resize-none"
            :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border)' }"
            placeholder="netsh interface portproxy add v4tov4 listenport=80 ..." />
          <div class="flex justify-end gap-2 mt-4">
            <button @click="showImportBundle = false; importText = ''" class="px-4 py-1.5 text-sm rounded-lg" :style="{ color: 'var(--text-secondary)', border: '1px solid var(--border)' }">Cancel</button>
            <button @click="handleImport; showImportBundle = false" class="px-4 py-1.5 text-sm rounded-lg font-medium" :style="{ background: 'var(--accent)', color: 'var(--bg-primary)' }">Import Script</button>
          </div>
        </template>
      </div>
    </div>

    <!-- QR Code modal -->
    <div v-if="qrUrl" class="fixed inset-0 z-50 flex items-center justify-center" style="background: rgba(0,0,0,0.6)" @click.self="qrUrl = null">
      <div class="rounded-xl p-6 shadow-2xl text-center" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-sm font-semibold" :style="{ color: 'var(--text-primary)' }">Scan to access</h3>
          <button @click="qrUrl = null" :style="{ color: 'var(--text-secondary)' }"><X :size="16" /></button>
        </div>
        <QrCode :url="qrUrl" :size="200" />
        <p class="mt-3 text-xs font-mono" :style="{ color: 'var(--accent)' }">{{ qrUrl }}</p>
      </div>
    </div>
  </div>
</template>
