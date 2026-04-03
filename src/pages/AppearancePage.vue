<script setup lang="ts">
import { computed } from "vue";
import { Check } from "lucide-vue-next";
import { useTheme } from "../hooks/useTheme";

const { currentTheme, setTheme, themes } = useTheme();

const categories = ["dark", "light", "accessibility", "auto"] as const;
function themesByCategory(cat: string) { return themes.filter((t) => t.category === cat); }
function displayName(name: string) { return name.split("-").map((w) => w.charAt(0).toUpperCase() + w.slice(1)).join(" "); }

const currentTokens = computed(() => themes.find((t) => t.name === currentTheme.value)?.tokens ?? {});
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold mb-2" :style="{ color: 'var(--text-primary)' }">Appearance</h2>
    <p class="text-sm mb-6" :style="{ color: 'var(--text-secondary)' }">Choose a theme. All themes use 11 CSS token variables.</p>
    <div v-for="cat in categories" :key="cat" class="mb-6">
      <template v-if="themesByCategory(cat).length">
        <h3 class="text-xs font-semibold uppercase tracking-wider mb-3" :style="{ color: 'var(--text-secondary)' }">{{ cat === "auto" ? "System" : cat.charAt(0).toUpperCase() + cat.slice(1) }}</h3>
        <div class="grid grid-cols-3 gap-3">
          <button v-for="t in themesByCategory(cat)" :key="t.name" @click="setTheme(t.name)" class="relative rounded-lg p-3 text-left transition-all"
            :style="{ background: t.tokens['--bg-primary'], border: `2px solid ${currentTheme === t.name ? t.tokens['--accent'] : t.tokens['--border']}` }">
            <div v-if="currentTheme === t.name" class="absolute top-2 right-2"><Check :size="14" :style="{ color: t.tokens['--accent'] }" /></div>
            <span class="text-sm font-medium" :style="{ color: t.tokens['--text-primary'] }">{{ displayName(t.name) }}</span>
            <div class="flex gap-1 mt-2">
              <span v-for="(c, i) in [t.tokens['--accent'], t.tokens['--status-ok'], t.tokens['--status-warn'], t.tokens['--status-err'], t.tokens['--text-secondary']]" :key="i" class="w-4 h-4 rounded-full" :style="{ background: c }" />
            </div>
          </button>
        </div>
      </template>
    </div>
    <div class="rounded-lg p-4 mt-6" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }">
      <h3 class="text-sm font-semibold mb-3" :style="{ color: 'var(--text-primary)' }">Current Theme Tokens</h3>
      <div class="grid grid-cols-2 gap-2">
        <div v-for="(value, key) in currentTokens" :key="key" class="flex items-center gap-2 text-xs">
          <span class="w-4 h-4 rounded border" :style="{ background: value, borderColor: 'var(--border)' }" />
          <span class="font-mono" :style="{ color: 'var(--text-secondary)' }">{{ key }}</span>
          <span class="font-mono" :style="{ color: 'var(--accent)' }">{{ value }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
