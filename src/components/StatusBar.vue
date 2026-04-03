<script setup lang="ts">
import { ref } from "vue";
import { RefreshCw, Globe, Wifi, Clock } from "lucide-vue-next";
import type { StatusInfo } from "../types";

defineProps<{ status: StatusInfo | null }>();
const emit = defineEmits<{ sync: [] }>();

const syncing = ref(false);
const lastSync = ref<string | null>(null);

async function handleSync() {
  syncing.value = true;
  try {
    if ("__TAURI__" in window) {
      const { syncNow } = await import("../hooks/useTauri");
      await syncNow();
    }
    lastSync.value = new Date().toLocaleTimeString();
    emit("sync");
  } finally {
    setTimeout(() => (syncing.value = false), 500);
  }
}

function copy(text: string) {
  navigator.clipboard.writeText(text);
}
</script>

<template>
  <footer class="h-8 flex items-center px-4 gap-4 text-xs border-t shrink-0"
    :style="{ background: 'var(--bg-tertiary)', borderColor: 'var(--border)', color: 'var(--text-secondary)' }">
    <div class="flex items-center gap-1.5">
      <span class="w-2 h-2 rounded-full"
        :style="{ background: status?.wsl_ip ? 'var(--status-ok)' : 'var(--status-err)' }" />
      <span>Service</span>
    </div>
    <span :style="{ color: 'var(--border)' }">|</span>
    <div class="flex items-center gap-1">
      <span :style="{ color: 'var(--accent)' }">{{ status?.active_rules ?? 0 }}</span>
      <span>active</span>
    </div>
    <div class="flex items-center gap-1">
      <Globe :size="11" />
      <span :style="{ color: 'var(--accent)' }">{{ status?.lan_rules ?? 0 }}</span>
      <span>LAN</span>
    </div>
    <span :style="{ color: 'var(--border)' }">|</span>
    <div class="flex items-center gap-1">
      <Wifi :size="11" />
      <span>WSL:</span>
      <button v-if="status?.wsl_ip" @click="copy(status.wsl_ip!)" class="hover:underline cursor-pointer"
        :style="{ color: 'var(--accent)' }" title="Click to copy">{{ status.wsl_ip }}</button>
      <span v-else :style="{ color: 'var(--accent)' }">—</span>
    </div>
    <div class="flex items-center gap-1">
      <span>Host:</span>
      <button v-if="status?.host_ip" @click="copy(status.host_ip!)" class="hover:underline cursor-pointer"
        :style="{ color: 'var(--accent)' }" title="Click to copy">{{ status.host_ip }}</button>
      <span v-else :style="{ color: 'var(--accent)' }">—</span>
    </div>
    <template v-if="lastSync">
      <span :style="{ color: 'var(--border)' }">|</span>
      <div class="flex items-center gap-1">
        <Clock :size="11" />
        <span>{{ lastSync }}</span>
      </div>
    </template>
    <div class="flex-1" />
    <button @click="handleSync" class="flex items-center gap-1 px-2 py-0.5 rounded transition-colors"
      :style="{ color: 'var(--accent)', background: syncing ? 'var(--accent-dim)' : 'transparent' }" title="Re-sync all rules now">
      <RefreshCw :size="11" :class="{ 'animate-spin': syncing }" />
      <span>Sync</span>
    </button>
  </footer>
</template>
