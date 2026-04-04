<script setup lang="ts">
import { ref, inject, onMounted, type Ref } from "vue";
import { FolderOpen, Plus, Power, Trash2, Edit2, Check, X, ChevronDown, ChevronUp } from "lucide-vue-next";
import type { Rule } from "../types";
import { useAuditLog } from "../hooks/useAuditLog";
import { isTauri } from "../lib/tauri";

const rules = inject<Ref<Rule[]>>("rules")!;
const refreshRules = inject<() => void>("refreshRules")!;
const { log } = useAuditLog();

interface RuleGroup { id: string; name: string; ruleIds: string[]; enabled: boolean; startupBehavior: string }

const groups = ref<RuleGroup[]>([]);
const newName = ref("");
const editingId = ref<string | null>(null);
const editName = ref("");
const expandedId = ref<string | null>(null);

async function loadGroups() {
  if (isTauri) {
    try {
      const { getSettings } = await import("../hooks/useTauri");
      const s = await getSettings();
      groups.value = s.groups ?? [];
    } catch { /* use defaults */ }
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
  const g: RuleGroup = { id: crypto.randomUUID(), name: newName.value.trim(), ruleIds: [], enabled: true, startupBehavior: "none" };
  groups.value.push(g);
  log("group.add", `Created group "${g.name}"`);
  newName.value = "";
  expandedId.value = g.id;
  await persist();
}

async function toggleGroup(g: RuleGroup) {
  g.enabled = !g.enabled;
  if (isTauri) {
    try {
      const { toggleRule } = await import("../hooks/useTauri");
      for (const rid of g.ruleIds) {
        const rule = rules.value.find(r => r.id === rid);
        if (rule && rule.enabled !== g.enabled) await toggleRule(rid);
      }
      refreshRules();
    } catch {}
  } else {
    rules.value = rules.value.map((r) => g.ruleIds.includes(r.id) ? { ...r, enabled: g.enabled } : r);
  }
  log("group.toggle", `${g.enabled ? "Enabled" : "Disabled"} group "${g.name}"`);
  await persist();
}

async function deleteGroup(id: string) {
  const g = groups.value.find((x) => x.id === id);
  groups.value = groups.value.filter((x) => x.id !== id);
  if (g) log("group.delete", `Deleted group "${g.name}"`);
  await persist();
}

function startEdit(g: RuleGroup) {
  editingId.value = g.id;
  editName.value = g.name;
}

async function saveEdit(g: RuleGroup) {
  if (editName.value.trim()) {
    g.name = editName.value.trim();
    log("group.rename", `Renamed group to "${g.name}"`);
    await persist();
  }
  editingId.value = null;
}

function toggleExpand(id: string) {
  expandedId.value = expandedId.value === id ? null : id;
}

function isRuleInGroup(g: RuleGroup, ruleId: string) {
  return g.ruleIds.includes(ruleId);
}

async function toggleRuleInGroup(g: RuleGroup, ruleId: string) {
  if (g.ruleIds.includes(ruleId)) {
    g.ruleIds = g.ruleIds.filter(id => id !== ruleId);
  } else {
    g.ruleIds.push(ruleId);
  }
  await persist();
}

function groupRules(g: RuleGroup) {
  return rules.value.filter(r => g.ruleIds.includes(r.id));
}

function portDisplay(r: Rule) {
  return r.listenPort.type === "single" ? `${r.listenPort.port}` : `${r.listenPort.start}–${r.listenPort.end}`;
}
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold mb-2" :style="{ color: 'var(--text-primary)' }">Groups / Profiles</h2>
    <p class="text-sm mb-4" :style="{ color: 'var(--text-secondary)' }">Group rules together for one-click enable/disable. Click a group to expand and manage its rules.</p>
    <div class="flex items-center gap-2 mb-6">
      <input v-model="newName" @keydown.enter="addGroup" placeholder="New group name..." class="flex-1 px-3 py-1.5 text-sm rounded-lg outline-none"
        :style="{ background: 'var(--bg-secondary)', color: 'var(--text-primary)', border: '1px solid var(--border)' }" />
      <button @click="addGroup" class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg font-medium"
        :style="{ background: 'var(--accent)', color: 'var(--bg-primary)' }"
        title="Create a new rule group"><Plus :size="12" /> Add Group</button>
    </div>
    <div class="space-y-2">
      <div v-for="g in groups" :key="g.id" class="rounded-lg overflow-hidden" :style="{ background: 'var(--bg-secondary)', border: `1px solid ${expandedId === g.id ? 'var(--accent-dim)' : 'var(--border)'}` }">
        <!-- Group header -->
        <div class="flex items-center gap-3 p-4 cursor-pointer" @click="toggleExpand(g.id)">
          <FolderOpen :size="16" :style="{ color: 'var(--accent)' }" />
          <template v-if="editingId === g.id">
            <input v-model="editName" @keydown.enter="saveEdit(g)" @keydown.escape="editingId = null"
              @click.stop class="w-40 px-2 py-0.5 text-sm rounded outline-none"
              :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--accent)' }" autofocus />
            <button @click.stop="saveEdit(g)" :style="{ color: 'var(--status-ok)' }" title="Save name"><Check :size="14" /></button>
            <button @click.stop="editingId = null" :style="{ color: 'var(--text-secondary)' }" title="Cancel"><X :size="14" /></button>
          </template>
          <template v-else>
            <span class="font-medium text-sm" :style="{ color: 'var(--text-primary)' }">{{ g.name }}</span>
            <button @click.stop="startEdit(g)" class="p-0.5 rounded hover:opacity-80"
              :style="{ color: 'var(--text-secondary)' }" title="Rename group"><Edit2 :size="12" /></button>
          </template>
          <span class="text-xs" :style="{ color: 'var(--text-secondary)' }">{{ groupRules(g).length }} rules</span>
          <div class="flex-1" />
          <select v-model="g.startupBehavior" @change="persist" @click.stop class="text-[10px] px-2 py-0.5 rounded"
            :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-secondary)', border: '1px solid var(--border)' }"
            title="What happens to this group's rules on startup">
            <option value="none">No startup action</option>
            <option value="enable">Enable on startup</option>
            <option value="disable">Disable on startup</option>
          </select>
          <button @click.stop="toggleGroup(g)" :title="g.enabled ? 'Disable all rules in group' : 'Enable all rules in group'">
            <Power :size="14" :style="{ color: g.enabled ? 'var(--status-ok)' : 'var(--text-secondary)' }" />
          </button>
          <button @click.stop="deleteGroup(g.id)" title="Delete this group"><Trash2 :size="14" :style="{ color: 'var(--status-err)' }" /></button>
          <component :is="expandedId === g.id ? ChevronUp : ChevronDown" :size="14" :style="{ color: 'var(--text-secondary)' }" />
        </div>

        <!-- Assigned rule tags (collapsed view) -->
        <div v-if="expandedId !== g.id && groupRules(g).length" class="flex flex-wrap gap-1.5 px-4 pb-3">
          <span v-for="r in groupRules(g)" :key="r.id" class="text-xs px-2 py-0.5 rounded"
            :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-secondary)' }">{{ r.name }}</span>
        </div>
        <p v-if="expandedId !== g.id && !groupRules(g).length" class="text-xs px-4 pb-3" :style="{ color: 'var(--text-secondary)' }">No rules assigned.</p>

        <!-- Expanded: rule assignment checklist -->
        <div v-if="expandedId === g.id" class="border-t px-4 py-3" :style="{ borderColor: 'var(--border)', background: 'var(--bg-tertiary)' }">
          <p class="text-xs mb-2 font-medium" :style="{ color: 'var(--text-secondary)' }">Toggle rules to add or remove from this group:</p>
          <div v-if="rules.length === 0" class="text-xs py-2" :style="{ color: 'var(--text-secondary)' }">No rules available. Create rules first on the Port Rules page.</div>
          <div class="space-y-1 max-h-64 overflow-y-auto">
            <label v-for="r in rules" :key="r.id"
              class="flex items-center gap-3 px-3 py-1.5 rounded cursor-pointer transition-colors"
              :style="{ background: isRuleInGroup(g, r.id) ? 'var(--bg-secondary)' : 'transparent' }">
              <input type="checkbox" :checked="isRuleInGroup(g, r.id)" @change="toggleRuleInGroup(g, r.id)"
                class="accent-[var(--accent)] shrink-0" />
              <span class="text-sm" :style="{ color: 'var(--text-primary)' }">{{ r.name }}</span>
              <span class="text-[10px] font-mono px-1.5 py-0.5 rounded"
                :style="{ background: 'var(--bg-primary)', color: 'var(--accent)' }">{{ portDisplay(r) }}</span>
              <span class="text-[10px] px-1.5 py-0.5 rounded"
                :style="{ background: r.direction === 'winToWsl' ? 'var(--accent-dim)' : 'var(--status-warn)', color: 'var(--text-primary)' }">
                {{ r.direction === "winToWsl" ? "WIN→WSL" : "WSL→WIN" }}
              </span>
              <span v-if="r.distro" class="text-[10px]" :style="{ color: 'var(--text-secondary)' }">{{ r.distro }}</span>
            </label>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
