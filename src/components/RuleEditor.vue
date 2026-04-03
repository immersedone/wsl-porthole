<script setup lang="ts">
import { ref, computed } from "vue";
import { X } from "lucide-vue-next";
import type { Rule, Direction, PortSpec } from "../types";

const props = defineProps<{ rule?: Rule }>();
const emit = defineEmits<{ save: [data: Partial<Rule>]; cancel: [] }>();

const name = ref(props.rule?.name ?? "");
const direction = ref<Direction>(props.rule?.direction ?? "winToWsl");
const listenPortStr = ref(
  props.rule?.listenPort.type === "single" ? String(props.rule.listenPort.port)
  : props.rule?.listenPort.start != null ? `${props.rule.listenPort.start}-${props.rule.listenPort.end}` : ""
);
const connectPortStr = ref(
  props.rule?.connectPort.type === "single" ? String(props.rule.connectPort.port)
  : props.rule?.connectPort.start != null ? `${props.rule.connectPort.start}-${props.rule.connectPort.end}` : ""
);
const connectAddr = ref(props.rule?.connectAddr ?? "${WSL_IP}");
const lan = ref(props.rule?.lan ?? true);
const distro = ref(props.rule?.distro ?? "");
const note = ref(props.rule?.note ?? "");

function parsePort(str: string): PortSpec {
  if (str.includes("-")) { const [s, e] = str.split("-").map(Number); return { type: "range", start: s, end: e }; }
  return { type: "single", port: Number(str) };
}

function save() {
  if (!name.value.trim() || !listenPortStr.value || !connectPortStr.value) return;
  emit("save", {
    ...(props.rule ? { id: props.rule.id } : {}),
    name: name.value.trim(), direction: direction.value,
    listenAddr: lan.value ? "0.0.0.0" : "127.0.0.1",
    listenPort: parsePort(listenPortStr.value), connectPort: parsePort(connectPortStr.value),
    connectAddr: connectAddr.value, lan: lan.value,
    distro: distro.value || null, note: note.value || null,
    enabled: props.rule?.enabled ?? true, source: props.rule?.source ?? "manual",
  });
}

const inputStyle = { background: "var(--bg-tertiary)", color: "var(--text-primary)", border: "1px solid var(--border)" };
</script>

<template>
  <div class="fixed inset-0 z-50 flex items-center justify-center" style="background: rgba(0,0,0,0.6)">
    <div class="w-[480px] rounded-xl p-6 shadow-2xl" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }">
      <div class="flex items-center justify-between mb-5">
        <h2 class="text-base font-semibold" :style="{ color: 'var(--text-primary)' }">{{ rule ? "Edit Rule" : "New Rule" }}</h2>
        <button @click="emit('cancel')" :style="{ color: 'var(--text-secondary)' }"><X :size="18" /></button>
      </div>
      <div class="space-y-4">
        <div>
          <label class="block text-xs mb-1 font-medium" :style="{ color: 'var(--text-secondary)' }">Name</label>
          <input v-model="name" placeholder="e.g. Django API" class="w-full px-3 py-1.5 text-sm rounded-lg outline-none" :style="inputStyle" />
        </div>
        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="block text-xs mb-1 font-medium" :style="{ color: 'var(--text-secondary)' }">Direction</label>
            <select v-model="direction" class="w-full px-3 py-1.5 text-sm rounded-lg outline-none" :style="inputStyle">
              <option value="winToWsl">WIN → WSL</option>
              <option value="wslToWin">WSL → WIN</option>
            </select>
          </div>
          <div>
            <label class="block text-xs mb-1 font-medium" :style="{ color: 'var(--text-secondary)' }">Connect address</label>
            <input v-model="connectAddr" placeholder="${WSL_IP}" class="w-full px-3 py-1.5 text-sm rounded-lg outline-none" :style="inputStyle" />
          </div>
        </div>
        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="block text-xs mb-1 font-medium" :style="{ color: 'var(--text-secondary)' }">Listen port(s)</label>
            <input v-model="listenPortStr" placeholder="80 or 1024-1048" class="w-full px-3 py-1.5 text-sm rounded-lg outline-none" :style="inputStyle" />
          </div>
          <div>
            <label class="block text-xs mb-1 font-medium" :style="{ color: 'var(--text-secondary)' }">Connect port(s)</label>
            <input v-model="connectPortStr" placeholder="80 or 1024-1048" class="w-full px-3 py-1.5 text-sm rounded-lg outline-none" :style="inputStyle" />
          </div>
        </div>
        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="block text-xs mb-1 font-medium" :style="{ color: 'var(--text-secondary)' }">Distro (optional)</label>
            <input v-model="distro" placeholder="auto" class="w-full px-3 py-1.5 text-sm rounded-lg outline-none" :style="inputStyle" />
          </div>
          <div>
            <label class="block text-xs mb-1 font-medium" :style="{ color: 'var(--text-secondary)' }">LAN visible</label>
            <label class="flex items-center gap-2 mt-1 cursor-pointer">
              <input type="checkbox" v-model="lan" class="accent-[var(--accent)]" />
              <span class="text-xs" :style="{ color: 'var(--text-secondary)' }">{{ lan ? "0.0.0.0 (LAN)" : "127.0.0.1 (local only)" }}</span>
            </label>
          </div>
        </div>
        <div>
          <label class="block text-xs mb-1 font-medium" :style="{ color: 'var(--text-secondary)' }">Note (optional)</label>
          <input v-model="note" placeholder="Optional description" class="w-full px-3 py-1.5 text-sm rounded-lg outline-none" :style="inputStyle" />
        </div>
        <div class="text-[11px] font-mono p-2 rounded" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-secondary)' }">
          netsh interface portproxy add v4tov4 listenport={{ listenPortStr || "?" }} listenaddress={{ lan ? "0.0.0.0" : "127.0.0.1" }} connectport={{ connectPortStr || "?" }} connectaddress={{ connectAddr }}
        </div>
      </div>
      <div class="flex justify-end gap-2 mt-6">
        <button @click="emit('cancel')" class="px-4 py-1.5 text-sm rounded-lg" :style="{ color: 'var(--text-secondary)', border: '1px solid var(--border)' }">Cancel</button>
        <button @click="save" class="px-4 py-1.5 text-sm rounded-lg font-medium" :style="{ background: 'var(--accent)', color: 'var(--bg-primary)' }">{{ rule ? "Save" : "Add Rule" }}</button>
      </div>
    </div>
  </div>
</template>
