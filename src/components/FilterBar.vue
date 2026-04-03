<script setup lang="ts">
import { computed } from "vue";
import { Search, X } from "lucide-vue-next";
import type { Direction, Source, HealthStatus } from "../types";

export interface FilterState {
  search: string;
  direction: Direction | "all";
  source: Source | "all";
  scope: "lan" | "local" | "all";
  health: HealthStatus | "all";
  enabled: "enabled" | "disabled" | "all";
}

const props = defineProps<{ modelValue: FilterState }>();
const emit = defineEmits<{ "update:modelValue": [filters: FilterState] }>();

function update(patch: Partial<FilterState>) {
  emit("update:modelValue", { ...props.modelValue, ...patch });
}

function reset() {
  emit("update:modelValue", { search: "", direction: "all", source: "all", scope: "all", health: "all", enabled: "all" });
}

const filterCount = computed(() =>
  [props.modelValue.direction !== "all", props.modelValue.source !== "all",
   props.modelValue.scope !== "all", props.modelValue.health !== "all",
   props.modelValue.enabled !== "all"].filter(Boolean).length
);

const selectStyle = "text-xs px-2 py-1 rounded";
const selectCss = { background: "var(--bg-tertiary)", color: "var(--text-primary)", border: "1px solid var(--border)" };
</script>

<template>
  <div class="flex items-center gap-3 p-3 rounded-lg mb-4"
    :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }">
    <div class="flex items-center gap-2 flex-1">
      <Search :size="14" :style="{ color: 'var(--text-secondary)' }" />
      <input type="text" placeholder="Search rules..." :value="modelValue.search"
        @input="update({ search: ($event.target as HTMLInputElement).value })"
        class="flex-1 bg-transparent text-sm outline-none" :style="{ color: 'var(--text-primary)' }" />
    </div>
    <select :value="modelValue.direction" @change="update({ direction: ($event.target as HTMLSelectElement).value as any })"
      :class="selectStyle" :style="selectCss">
      <option value="all">All directions</option>
      <option value="winToWsl">WIN → WSL</option>
      <option value="wslToWin">WSL → WIN</option>
    </select>
    <select :value="modelValue.source" @change="update({ source: ($event.target as HTMLSelectElement).value as any })"
      :class="selectStyle" :style="selectCss">
      <option value="all">All sources</option>
      <option value="manual">Manual</option>
      <option value="docker">Docker</option>
      <option value="mcp">MCP</option>
      <option value="imported">Imported</option>
    </select>
    <select :value="modelValue.scope" @change="update({ scope: ($event.target as HTMLSelectElement).value as any })"
      :class="selectStyle" :style="selectCss">
      <option value="all">All scopes</option>
      <option value="lan">LAN</option>
      <option value="local">Local</option>
    </select>
    <select :value="modelValue.health" @change="update({ health: ($event.target as HTMLSelectElement).value as any })"
      :class="selectStyle" :style="selectCss">
      <option value="all">All health</option>
      <option value="ok">Healthy</option>
      <option value="warn">Warning</option>
      <option value="error">Error</option>
      <option value="unknown">Unknown</option>
    </select>
    <div v-if="filterCount > 0" class="flex items-center gap-1">
      <span class="text-xs px-1.5 py-0.5 rounded-full"
        :style="{ background: 'var(--accent-dim)', color: 'var(--text-primary)' }">{{ filterCount }}</span>
      <button @click="reset" class="p-0.5 rounded hover:opacity-80" :style="{ color: 'var(--text-secondary)' }" title="Clear all filters">
        <X :size="12" />
      </button>
    </div>
  </div>
</template>
