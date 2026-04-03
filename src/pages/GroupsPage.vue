<script setup lang="ts">
import { ref, inject, onMounted, type Ref } from "vue";
import { FolderOpen, Plus, Power, Trash2 } from "lucide-vue-next";
import type { Rule } from "../types";
import { useAuditLog } from "../hooks/useAuditLog";
import { isTauri } from "../lib/tauri";

const rules = inject<Ref<Rule[]>>("rules")!;
const { log } = useAuditLog();

interface RuleGroup { id: string; name: string; ruleIds: string[]; enabled: boolean; startupBehavior: string }

const groups = ref<RuleGroup[]>([]);
const newName = ref("");

async function loadGroups() {
  if (isTauri) {
    try {
      const { getSettings } = await import("../hooks/useTauri");
      const s = await getSettings();
      groups.value = s.groups ?? [];
    } catch { /* use defaults */ }
  }
  if (!groups.value.length) {
    groups.value = [
      { id: "1", name: "Web Stack", ruleIds: ["1", "2", "3"], enabled: true, startupBehavior: "enable" },
      { id: "2", name: "Dev Tools", ruleIds: ["6", "7"], enabled: false, startupBehavior: "none" },
    ];
  }
}

async function persist() {
  if (!isTauri) return;
  try {
    const { getSettings, saveSettings } = await import("../hooks/useTauri");
    const s = await getSettings();
    s.groups = groups.value;
    await saveSettings(s);
  } catch (e) { log("groups.save", `Failed: ${e}`, "error"); }
}

onMounted(loadGroups);

async function addGroup() {
  if (!newName.value.trim()) return;
  groups.value.push({ id: crypto.randomUUID(), name: newName.value.trim(), ruleIds: [], enabled: true, startupBehavior: "none" });
  log("group.add", `Created group "${newName.value.trim()}"`);
  newName.value = "";
  await persist();
}

async function toggleGroup(g: RuleGroup) {
  g.enabled = !g.enabled;
  rules.value = rules.value.map((r) => g.ruleIds.includes(r.id) ? { ...r, enabled: g.enabled } : r);
  log("group.toggle", `${g.enabled ? "Enabled" : "Disabled"} group "${g.name}"`);
  await persist();
}

async function deleteGroup(id: string) {
  const g = groups.value.find((x) => x.id === id);
  groups.value = groups.value.filter((x) => x.id !== id);
  if (g) log("group.delete", `Deleted group "${g.name}"`);
  await persist();
}
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold mb-2" :style="{ color: 'var(--text-primary)' }">Groups / Profiles</h2>
    <p class="text-sm mb-4" :style="{ color: 'var(--text-secondary)' }">Group rules together for one-click enable/disable.</p>
    <div class="flex items-center gap-2 mb-6">
      <input v-model="newName" @keydown.enter="addGroup" placeholder="New group name..." class="flex-1 px-3 py-1.5 text-sm rounded-lg outline-none"
        :style="{ background: 'var(--bg-secondary)', color: 'var(--text-primary)', border: '1px solid var(--border)' }" />
      <button @click="addGroup" class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg font-medium" :style="{ background: 'var(--accent)', color: 'var(--bg-primary)' }"><Plus :size="12" /> Add Group</button>
    </div>
    <div class="space-y-2">
      <div v-for="g in groups" :key="g.id" class="rounded-lg p-4" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }">
        <div class="flex items-center gap-3 mb-2">
          <FolderOpen :size="16" :style="{ color: 'var(--accent)' }" />
          <span class="font-medium text-sm" :style="{ color: 'var(--text-primary)' }">{{ g.name }}</span>
          <span class="text-xs" :style="{ color: 'var(--text-secondary)' }">{{ rules.filter((r) => g.ruleIds.includes(r.id)).length }} rules</span>
          <div class="flex-1" />
          <select v-model="g.startupBehavior" @change="persist" class="text-[10px] px-2 py-0.5 rounded" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-secondary)', border: '1px solid var(--border)' }">
            <option value="none">No startup action</option><option value="enable">Enable on startup</option><option value="disable">Disable on startup</option>
          </select>
          <button @click="toggleGroup(g)"><Power :size="14" :style="{ color: g.enabled ? 'var(--status-ok)' : 'var(--text-secondary)' }" /></button>
          <button @click="deleteGroup(g.id)"><Trash2 :size="14" :style="{ color: 'var(--status-err)' }" /></button>
        </div>
        <div v-if="rules.filter((r) => g.ruleIds.includes(r.id)).length" class="flex flex-wrap gap-1.5">
          <span v-for="r in rules.filter((r) => g.ruleIds.includes(r.id))" :key="r.id" class="text-xs px-2 py-0.5 rounded"
            :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-secondary)' }">{{ r.name }}</span>
        </div>
        <p v-else class="text-xs" :style="{ color: 'var(--text-secondary)' }">No rules assigned.</p>
      </div>
    </div>
  </div>
</template>
