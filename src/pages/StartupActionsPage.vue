<script setup lang="ts">
import { ref, onMounted } from "vue";
import { Zap, Plus, Trash2, GripVertical, Play, Clock } from "lucide-vue-next";
import { useAuditLog } from "../hooks/useAuditLog";
import { isTauri } from "../lib/tauri";
import { useAlive } from "../hooks/useAlive";

const alive = useAlive();
const { log } = useAuditLog();

interface StartupAction { id: string; label: string; type: string; command: string; delayMs: number; enabled: boolean; target: string }
const actions = ref<StartupAction[]>([]);
const newCmd = ref("");

async function load() {
  if (isTauri) {
    try {
      const { getSettings } = await import("../hooks/useTauri");
      const s = await getSettings();
      if (!alive.value) return;
      if (s.startupActions?.length) {
        actions.value = s.startupActions.map((a: any) => ({ ...a, type: a.actionType ?? a.type }));
        return;
      }
    } catch {}
  }
  // Defaults
  actions.value = [
    { id: "1", label: "Sync port rules", type: "builtin", command: "sync-rules", delayMs: 0, enabled: true, target: "all" },
    { id: "2", label: "Write /etc/hosts", type: "builtin", command: "write-hosts", delayMs: 1000, enabled: true, target: "all" },
    { id: "3", label: "Inject env vars", type: "builtin", command: "inject-env", delayMs: 2000, enabled: false, target: "all" },
  ];
}

async function persist() {
  if (!isTauri) return;
  try {
    const { getSettings, saveSettings } = await import("../hooks/useTauri");
    const s = await getSettings();
    s.startupActions = actions.value.map((a) => ({ ...a, actionType: a.type }));
    await saveSettings(s);
  } catch (e) { log("startup.save", `Failed: ${e}`, "error"); }
}

onMounted(load);

async function add() {
  if (!newCmd.value.trim()) return;
  actions.value.push({ id: crypto.randomUUID(), label: newCmd.value.trim().split(" ")[0], type: "custom", command: newCmd.value.trim(), delayMs: 0, enabled: true, target: "all" });
  log("startup.add", `Added: ${newCmd.value.trim()}`);
  newCmd.value = "";
  await persist();
}
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold mb-2" :style="{ color: 'var(--text-primary)' }">Startup Actions</h2>
    <p class="text-sm mb-6" :style="{ color: 'var(--text-secondary)' }">Commands that run on WSL start. Supports variable substitution and configurable delays.</p>
    <div class="space-y-1.5 mb-6">
      <div v-for="a in actions" :key="a.id" class="flex items-center gap-3 px-4 py-2.5 rounded-lg"
        :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)', opacity: a.enabled ? 1 : 0.5 }">
        <GripVertical :size="12" :style="{ color: 'var(--text-secondary)' }" class="cursor-grab" />
        <Zap :size="14" :style="{ color: a.type === 'builtin' ? 'var(--accent)' : 'var(--status-warn)' }" />
        <span class="text-sm font-medium" :style="{ color: 'var(--text-primary)' }">{{ a.label }}</span>
        <span v-if="a.type === 'custom'" class="text-xs font-mono" :style="{ color: 'var(--text-secondary)' }">{{ a.command }}</span>
        <span v-if="a.delayMs > 0" class="flex items-center gap-0.5 text-[10px]" :style="{ color: 'var(--text-secondary)' }"><Clock :size="10" /> +{{ a.delayMs }}ms</span>
        <span class="text-[10px] px-1.5 py-0.5 rounded" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-secondary)' }">{{ a.target }}</span>
        <div class="flex-1" />
        <input type="number" v-model.number="a.delayMs" @change="persist" class="w-20 text-xs px-2 py-0.5 rounded text-right" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border)' }" />
        <button @click="a.enabled = !a.enabled; persist()"><Play :size="12" :style="{ color: a.enabled ? 'var(--status-ok)' : 'var(--text-secondary)' }" /></button>
        <button v-if="a.type === 'custom'" @click="actions = actions.filter((x) => x.id !== a.id); persist()"><Trash2 :size="12" :style="{ color: 'var(--status-err)' }" /></button>
      </div>
    </div>
    <div class="flex items-center gap-2">
      <input v-model="newCmd" @keydown.enter="add" placeholder="Custom shell command..." class="flex-1 px-3 py-1.5 text-sm rounded-lg outline-none font-mono"
        :style="{ background: 'var(--bg-secondary)', color: 'var(--text-primary)', border: '1px solid var(--border)' }" />
      <button @click="add" class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg font-medium" :style="{ background: 'var(--accent)', color: 'var(--bg-primary)' }"><Plus :size="12" /> Add Action</button>
    </div>
  </div>
</template>
