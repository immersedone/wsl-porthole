<script setup lang="ts">
import { ref, computed } from "vue";
import { Check, Download, Upload } from "lucide-vue-next";
import { useTheme } from "../hooks/useTheme";
import type { ThemeTokens } from "../types";

const { currentTheme, setTheme, themes } = useTheme();

const categories = ["dark", "light", "accessibility", "auto"] as const;
function themesByCategory(cat: string) { return themes.filter((t) => t.category === cat); }
function displayName(name: string) { return name.split("-").map((w) => w.charAt(0).toUpperCase() + w.slice(1)).join(" "); }

const currentTokens = computed(() => themes.find((t) => t.name === currentTheme.value)?.tokens ?? {});

// Theme editor state
const showEditor = ref(false);
const editTokens = ref<Record<string, string>>({});
const editName = ref("My Custom Theme");

function openEditor() {
  // Clone current tokens for editing
  editTokens.value = { ...currentTokens.value };
  editName.value = `${displayName(currentTheme.value)} (custom)`;
  showEditor.value = true;
}

function applyEditTokens() {
  const root = document.documentElement;
  for (const [key, value] of Object.entries(editTokens.value)) {
    root.style.setProperty(key, value);
  }
}

function exportTheme() {
  const theme = { name: editName.value, category: "dark", tokens: editTokens.value };
  const blob = new Blob([JSON.stringify(theme, null, 2)], { type: "application/json" });
  const a = document.createElement("a"); a.href = URL.createObjectURL(blob);
  a.download = `${editName.value.toLowerCase().replace(/\s+/g, "-")}.wph-theme.json`; a.click();
}

function importTheme() {
  const input = document.createElement("input");
  input.type = "file"; input.accept = ".json,.wph-theme.json";
  input.onchange = async () => {
    const file = input.files?.[0];
    if (!file) return;
    try {
      const text = await file.text();
      const theme = JSON.parse(text);
      if (theme.tokens) {
        editTokens.value = theme.tokens;
        editName.value = theme.name ?? "Imported Theme";
        showEditor.value = true;
        applyEditTokens();
      }
    } catch {}
  };
  input.click();
}
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-2">
      <h2 class="text-lg font-semibold" :style="{ color: 'var(--text-primary)' }">Appearance</h2>
      <div class="flex items-center gap-2">
        <button @click="importTheme" class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg" :style="{ color: 'var(--text-secondary)', border: '1px solid var(--border)' }"><Upload :size="12" /> Import</button>
        <button @click="openEditor" class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg font-medium" :style="{ background: 'var(--accent)', color: 'var(--bg-primary)' }">Custom Theme</button>
      </div>
    </div>
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

    <!-- Token reference -->
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

    <!-- Theme editor modal -->
    <div v-if="showEditor" class="fixed inset-0 z-50 flex items-center justify-center" style="background: rgba(0,0,0,0.6)" @click.self="showEditor = false">
      <div class="w-[520px] max-h-[80vh] overflow-y-auto rounded-xl p-6 shadow-2xl" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }">
        <h3 class="text-base font-semibold mb-4" :style="{ color: 'var(--text-primary)' }">Custom Theme Editor</h3>
        <div class="mb-4">
          <label class="block text-xs mb-1 font-medium" :style="{ color: 'var(--text-secondary)' }">Theme name</label>
          <input v-model="editName" class="w-full px-3 py-1.5 text-sm rounded-lg outline-none" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border)' }" />
        </div>
        <div class="space-y-2 mb-6">
          <div v-for="(value, key) in editTokens" :key="key" class="flex items-center gap-3">
            <input type="color" :value="value" @input="editTokens[key] = ($event.target as HTMLInputElement).value; applyEditTokens()" class="w-8 h-8 rounded cursor-pointer border-0 p-0" />
            <span class="text-xs font-mono w-36" :style="{ color: 'var(--text-secondary)' }">{{ key }}</span>
            <input :value="value" @input="editTokens[key] = ($event.target as HTMLInputElement).value; applyEditTokens()"
              class="flex-1 px-2 py-1 text-xs font-mono rounded outline-none"
              :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border)' }" />
          </div>
        </div>
        <div class="flex justify-end gap-2">
          <button @click="showEditor = false" class="px-4 py-1.5 text-sm rounded-lg" :style="{ color: 'var(--text-secondary)', border: '1px solid var(--border)' }">Close</button>
          <button @click="exportTheme" class="flex items-center gap-1.5 px-4 py-1.5 text-sm rounded-lg font-medium" :style="{ background: 'var(--accent)', color: 'var(--bg-primary)' }"><Download :size="12" /> Export .wph-theme.json</button>
        </div>
      </div>
    </div>
  </div>
</template>
