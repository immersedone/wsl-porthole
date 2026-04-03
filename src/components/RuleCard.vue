<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from "vue";
import { MoreVertical, ArrowRight, Globe, Lock, Copy, Edit2, Trash2, ExternalLink, Terminal, Power } from "lucide-vue-next";
import type { Rule } from "../types";

const props = defineProps<{ rule: Rule; selected: boolean }>();
const emit = defineEmits<{
  toggle: [id: string]; edit: [rule: Rule]; delete: [id: string];
  duplicate: [rule: Rule]; select: [];
}>();

const menuOpen = ref(false);
const menuEl = ref<HTMLElement | null>(null);

function onDocClick(e: MouseEvent) {
  if (menuEl.value && !menuEl.value.contains(e.target as Node)) menuOpen.value = false;
}
onMounted(() => document.addEventListener("click", onDocClick));
onUnmounted(() => document.removeEventListener("click", onDocClick));

const healthColor = computed(() =>
  props.rule.health === "ok" ? "var(--status-ok)" : props.rule.health === "warn" ? "var(--status-warn)" : props.rule.health === "error" ? "var(--status-err)" : "var(--text-secondary)"
);
const listenDisp = computed(() => props.rule.listenPort.type === "single" ? `${props.rule.listenPort.port}` : `${props.rule.listenPort.start}–${props.rule.listenPort.end}`);
const connectDisp = computed(() => props.rule.connectPort.type === "single" ? `${props.rule.connectPort.port}` : `${props.rule.connectPort.start}–${props.rule.connectPort.end}`);
const isRemapped = computed(() => listenDisp.value !== connectDisp.value);

function onKey(e: KeyboardEvent) {
  if (e.key === " ") { e.preventDefault(); emit("toggle", props.rule.id); }
  if (e.key === "Enter") emit("edit", props.rule);
  if (e.key === "Delete") emit("delete", props.rule.id);
}

function copyCmd() {
  menuOpen.value = false;
  const lp = props.rule.listenPort.type === "single" ? props.rule.listenPort.port : props.rule.listenPort.start;
  const cp = props.rule.connectPort.type === "single" ? props.rule.connectPort.port : props.rule.connectPort.start;
  navigator.clipboard.writeText(`netsh interface portproxy add v4tov4 listenport=${lp} listenaddress=${props.rule.listenAddr} connectport=${cp} connectaddress=${props.rule.connectAddr}`);
}

async function openInBrowser() {
  const port = props.rule.listenPort.type === "single" ? props.rule.listenPort.port : props.rule.listenPort.start;
  const url = `http://localhost:${port}`;
  if ("__TAURI__" in window) {
    const { open } = await import("@tauri-apps/plugin-shell");
    await open(url);
  } else {
    globalThis.window.open(url, "_blank");
  }
}
</script>

<template>
  <div class="flex items-center gap-3 px-4 py-2.5 rounded-lg transition-all cursor-pointer" tabindex="0"
    :style="{ background: selected ? 'var(--bg-tertiary)' : 'var(--bg-secondary)', border: `1px solid ${selected ? 'var(--accent-dim)' : 'var(--border)'}`, opacity: rule.enabled ? 1 : 0.5 }"
    @click="emit('select')" @keydown="onKey">
    <span class="w-2 h-2 rounded-full shrink-0" :style="{ background: healthColor }" :title="`Health: ${rule.health ?? 'unknown'}`" />
    <button @click.stop="emit('toggle', rule.id)" class="shrink-0" :title="rule.enabled ? 'Disable' : 'Enable'">
      <Power :size="14" :style="{ color: rule.enabled ? 'var(--status-ok)' : 'var(--text-secondary)' }" />
    </button>
    <span class="text-[10px] font-mono px-1.5 py-0.5 rounded shrink-0"
      :style="{ background: rule.direction === 'winToWsl' ? 'var(--accent-dim)' : 'var(--status-warn)', color: 'var(--text-primary)' }">
      {{ rule.direction === "winToWsl" ? "WIN→WSL" : "WSL→WIN" }}
    </span>
    <span class="font-medium text-sm min-w-0 truncate" :style="{ color: 'var(--text-primary)' }">{{ rule.name }}</span>
    <span class="text-xs font-mono px-1.5 py-0.5 rounded shrink-0" :style="{ background: 'var(--bg-tertiary)', color: 'var(--accent)' }">
      <template v-if="isRemapped">{{ listenDisp }}<ArrowRight :size="10" class="inline mx-0.5" />{{ connectDisp }}</template>
      <template v-else>{{ listenDisp }}</template>
    </span>
    <span v-if="rule.source !== 'manual'" class="text-[10px] px-1.5 py-0.5 rounded"
      :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-secondary)' }">{{ rule.source }}</span>
    <span :title="rule.lan ? 'LAN visible (0.0.0.0)' : 'Local only (127.0.0.1)'">
      <Globe v-if="rule.lan" :size="13" :style="{ color: 'var(--status-warn)' }" />
      <Lock v-else :size="13" :style="{ color: 'var(--text-secondary)' }" />
    </span>
    <span class="text-[10px] px-1.5 py-0.5 rounded"
      :style="{ background: 'var(--bg-tertiary)', color: rule.distro ? 'var(--accent)' : 'var(--text-secondary)' }"
      :title="rule.distro ? `Targeting: ${rule.distro}` : 'Using default distro'">{{ rule.distro || 'default' }}</span>
    <span v-if="rule.conflict" class="text-[10px] px-1.5 py-0.5 rounded"
      :style="{ background: 'var(--status-warn)', color: '#000' }" :title="rule.conflict">conflict</span>
    <div class="flex-1" />
    <div class="relative" ref="menuEl">
      <button @click.stop="menuOpen = !menuOpen" class="p-1 rounded hover:opacity-80" :style="{ color: 'var(--text-secondary)' }">
        <MoreVertical :size="14" />
      </button>
      <div v-if="menuOpen" class="absolute right-0 top-8 z-50 min-w-[160px] py-1 rounded-lg shadow-lg"
        :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }">
        <button @click="menuOpen = false; emit('edit', rule)" class="w-full flex items-center gap-2 px-3 py-1.5 text-xs hover:opacity-80" :style="{ color: 'var(--text-primary)' }"><Edit2 :size="12" /> Edit</button>
        <button @click="menuOpen = false; emit('duplicate', rule)" class="w-full flex items-center gap-2 px-3 py-1.5 text-xs hover:opacity-80" :style="{ color: 'var(--text-primary)' }"><Copy :size="12" /> Duplicate</button>
        <button @click="copyCmd" class="w-full flex items-center gap-2 px-3 py-1.5 text-xs hover:opacity-80" :style="{ color: 'var(--text-primary)' }"><Terminal :size="12" /> Copy command</button>
        <template v-if="rule.lan">
          <button @click="menuOpen = false; openInBrowser()"
            class="w-full flex items-center gap-2 px-3 py-1.5 text-xs hover:opacity-80" :style="{ color: 'var(--text-primary)' }"><ExternalLink :size="12" /> Open in browser</button>
        </template>
        <div class="my-1" :style="{ borderTop: '1px solid var(--border)' }" />
        <button @click="menuOpen = false; emit('delete', rule.id)" class="w-full flex items-center gap-2 px-3 py-1.5 text-xs hover:opacity-80" :style="{ color: 'var(--status-err)' }"><Trash2 :size="12" /> Delete</button>
      </div>
    </div>
  </div>
</template>
