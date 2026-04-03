<script setup lang="ts">
import { computed, inject, type Ref } from "vue";
import { Globe, Copy, ExternalLink, QrCode } from "lucide-vue-next";
import type { Rule, StatusInfo } from "../types";

const rules = inject<Ref<Rule[]>>("rules")!;
const status = inject<Ref<StatusInfo | null>>("status")!;
const lanRules = computed(() => rules.value.filter((r) => r.enabled && r.lan));
const hostIp = computed(() => status.value?.host_ip ?? "?.?.?.?");

function copyUrl(port: number) { navigator.clipboard.writeText(`http://${hostIp.value}:${port}`); }
function openUrl(port: number) { globalThis.window.open(`http://${hostIp.value}:${port}`, "_blank"); }
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold mb-2" :style="{ color: 'var(--text-primary)' }">LAN Access</h2>
    <p class="text-sm mb-6" :style="{ color: 'var(--text-secondary)' }">
      Rules bound to <code class="font-mono text-xs px-1 rounded" :style="{ background: 'var(--bg-tertiary)' }">0.0.0.0</code> are accessible from your network.
      Host IP: <span :style="{ color: 'var(--accent)' }">{{ hostIp }}</span>
    </p>
    <div v-if="lanRules.length" class="space-y-2">
      <div v-for="rule in lanRules" :key="rule.id" class="flex items-center gap-3 p-3 rounded-lg" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }">
        <Globe :size="14" :style="{ color: 'var(--status-warn)' }" />
        <span class="font-medium text-sm" :style="{ color: 'var(--text-primary)' }">{{ rule.name }}</span>
        <span class="text-xs font-mono" :style="{ color: 'var(--accent)' }">http://{{ hostIp }}:{{ rule.listenPort.type === "single" ? rule.listenPort.port : rule.listenPort.start }}</span>
        <div class="flex-1" />
        <button @click="copyUrl(rule.listenPort.type === 'single' ? rule.listenPort.port! : rule.listenPort.start!)" class="p-1 rounded hover:opacity-80" :style="{ color: 'var(--text-secondary)' }" title="Copy URL"><Copy :size="13" /></button>
        <button @click="openUrl(rule.listenPort.type === 'single' ? rule.listenPort.port! : rule.listenPort.start!)" class="p-1 rounded hover:opacity-80" :style="{ color: 'var(--text-secondary)' }" title="Open"><ExternalLink :size="13" /></button>
        <button class="p-1 rounded hover:opacity-80" :style="{ color: 'var(--text-secondary)' }" title="QR Code"><QrCode :size="13" /></button>
      </div>
    </div>
    <div v-else class="text-center py-12" :style="{ color: 'var(--text-secondary)' }">No LAN-exposed rules.</div>
  </div>
</template>
