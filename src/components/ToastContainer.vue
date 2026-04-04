<script setup lang="ts">
import { CheckCircle, AlertCircle, Info, AlertTriangle, X } from "lucide-vue-next";
import { useToast } from "../hooks/useToast";

const { toasts, dismiss } = useToast();

function iconFor(type: string) {
  switch (type) {
    case "success": return CheckCircle;
    case "error": return AlertCircle;
    case "warn": return AlertTriangle;
    default: return Info;
  }
}

function colorFor(type: string) {
  switch (type) {
    case "success": return "var(--status-ok)";
    case "error": return "var(--status-err)";
    case "warn": return "var(--status-warn)";
    default: return "var(--accent)";
  }
}
</script>

<template>
  <div class="fixed top-4 right-4 z-[100] flex flex-col gap-2 pointer-events-none">
    <div v-for="toast in toasts" :key="toast.id"
      class="flex items-center gap-2 px-4 py-2.5 rounded-lg shadow-lg pointer-events-auto min-w-[260px] max-w-[400px]"
      :class="toast.exiting ? 'toast-exit' : 'toast-enter'"
      :style="{ background: 'var(--bg-secondary)', border: `1px solid ${colorFor(toast.type)}`, color: 'var(--text-primary)' }">
      <component :is="iconFor(toast.type)" :size="16" :style="{ color: colorFor(toast.type), flexShrink: 0 }" />
      <span class="text-sm flex-1">{{ toast.message }}</span>
      <button @click="dismiss(toast.id)" class="shrink-0 p-0.5 rounded hover:opacity-80"
        :style="{ color: 'var(--text-secondary)' }">
        <X :size="12" />
      </button>
    </div>
  </div>
</template>
