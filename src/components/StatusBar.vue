<script setup lang="ts">
import { ref, inject, type Ref } from "vue";
import { RefreshCw, Globe, Wifi, Clock, AlertTriangle } from "lucide-vue-next";
import type { StatusInfo, Rule } from "../types";

const props = defineProps<{ status: StatusInfo | null }>();
const emit = defineEmits<{ sync: [] }>();
const rules = inject<Ref<Rule[]>>("rules", ref([]));

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

function conflictCount() {
  return rules.value.filter(r => r.conflict).length;
}
</script>

<template>
  <footer class="h-8 flex items-center px-4 gap-4 text-xs border-t shrink-0"
    :style="{ background: 'var(--bg-tertiary)', borderColor: 'var(--border)', color: 'var(--text-secondary)' }">
    <!-- Service status -->
    <div class="flex items-center gap-1.5 cursor-help"
      :title="status?.wsl_ip ? 'Service: connected — WSL IP detected' : `Disconnected: ${status?.wsl_error ?? 'WSL IP not detected'}`">
      <span class="w-2 h-2 rounded-full"
        :style="{ background: status?.wsl_ip ? 'var(--status-ok)' : 'var(--status-err)' }" />
      <span>{{ status?.wsl_ip ? 'Connected' : 'Disconnected' }}</span>
    </div>
    <span :style="{ color: 'var(--border)' }">|</span>

    <!-- Active rule count -->
    <div class="flex items-center gap-1 cursor-help"
      :title="`${status?.active_rules ?? 0} active rules out of ${status?.total_rules ?? 0} total`">
      <span :style="{ color: 'var(--accent)' }">{{ status?.active_rules ?? 0 }}</span>
      <span>active</span>
    </div>

    <!-- LAN count -->
    <div class="flex items-center gap-1 cursor-help"
      :title="`${status?.lan_rules ?? 0} rules exposed on LAN (0.0.0.0)`">
      <Globe :size="11" />
      <span :style="{ color: 'var(--accent)' }">{{ status?.lan_rules ?? 0 }}</span>
      <span>LAN</span>
    </div>

    <!-- Conflict count (if any) -->
    <div v-if="conflictCount() > 0" class="flex items-center gap-1 cursor-help"
      :title="`${conflictCount()} port conflicts detected — check rules for details`">
      <AlertTriangle :size="11" :style="{ color: 'var(--status-warn)' }" />
      <span :style="{ color: 'var(--status-warn)' }">{{ conflictCount() }}</span>
      <span :style="{ color: 'var(--status-warn)' }">conflicts</span>
    </div>

    <span :style="{ color: 'var(--border)' }">|</span>

    <!-- WSL IP -->
    <div class="flex items-center gap-1">
      <Wifi :size="11" />
      <span>WSL:</span>
      <button v-if="status?.wsl_ip" @click="copy(status.wsl_ip!)" class="hover:underline cursor-pointer"
        :style="{ color: 'var(--accent)' }" title="Click to copy WSL IP to clipboard">{{ status.wsl_ip }}</button>
      <span v-else :style="{ color: 'var(--text-secondary)' }" title="WSL IP not detected">—</span>
    </div>

    <!-- Host IP -->
    <div class="flex items-center gap-1">
      <span>Host:</span>
      <button v-if="status?.host_ip" @click="copy(status.host_ip!)" class="hover:underline cursor-pointer"
        :style="{ color: 'var(--accent)' }" title="Click to copy Host IP to clipboard">{{ status.host_ip }}</button>
      <span v-else :style="{ color: 'var(--text-secondary)' }" title="Host IP not detected">—</span>
    </div>

    <!-- Last sync timestamp -->
    <template v-if="lastSync">
      <span :style="{ color: 'var(--border)' }">|</span>
      <div class="flex items-center gap-1 cursor-help" :title="`Last sync at ${lastSync}`">
        <Clock :size="11" />
        <span>{{ lastSync }}</span>
      </div>
    </template>

    <div class="flex-1" />

    <!-- Sync button -->
    <button @click="handleSync" class="flex items-center gap-1 px-2 py-0.5 rounded transition-colors"
      :style="{ color: 'var(--accent)', background: syncing ? 'var(--accent-dim)' : 'transparent' }"
      title="Re-detect WSL IP and re-apply all enabled port forwarding rules">
      <RefreshCw :size="11" :class="{ 'animate-spin': syncing }" />
      <span>Sync</span>
    </button>
  </footer>
</template>
