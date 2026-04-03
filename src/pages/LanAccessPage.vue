<script setup lang="ts">
import { ref, computed, inject, type Ref } from "vue";
import { Globe, Copy, ExternalLink, QrCode as QrIcon, X } from "lucide-vue-next";
import QrCode from "../components/QrCode.vue";
import type { Rule, StatusInfo } from "../types";

const rules = inject<Ref<Rule[]>>("rules")!;
const status = inject<Ref<StatusInfo | null>>("status")!;
const lanRules = computed(() => rules.value.filter((r) => r.enabled && r.lan));
const hostIp = computed(() => status.value?.host_ip ?? "?.?.?.?");
const qrUrl = ref<string | null>(null);

function getPort(rule: Rule) { return rule.listenPort.type === "single" ? rule.listenPort.port! : rule.listenPort.start!; }
function getUrl(rule: Rule) { return `http://${hostIp.value}:${getPort(rule)}`; }
function copyUrl(rule: Rule) { navigator.clipboard.writeText(getUrl(rule)); }
async function openUrl(rule: Rule) {
  const url = getUrl(rule);
  if ("__TAURI__" in window) {
    const { open } = await import("@tauri-apps/plugin-shell");
    await open(url);
  } else {
    globalThis.window.open(url, "_blank");
  }
}
function showQr(rule: Rule) { qrUrl.value = getUrl(rule); }
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
        <span class="text-xs font-mono" :style="{ color: 'var(--accent)' }">{{ getUrl(rule) }}</span>
        <div class="flex-1" />
        <button @click="copyUrl(rule)" class="p-1 rounded hover:opacity-80" :style="{ color: 'var(--text-secondary)' }" title="Copy URL"><Copy :size="13" /></button>
        <button @click="openUrl(rule)" class="p-1 rounded hover:opacity-80" :style="{ color: 'var(--text-secondary)' }" title="Open"><ExternalLink :size="13" /></button>
        <button @click="showQr(rule)" class="p-1 rounded hover:opacity-80" :style="{ color: 'var(--text-secondary)' }" title="QR Code"><QrIcon :size="13" /></button>
      </div>
    </div>
    <div v-else class="text-center py-12" :style="{ color: 'var(--text-secondary)' }">No LAN-exposed rules.</div>

    <!-- QR modal -->
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
