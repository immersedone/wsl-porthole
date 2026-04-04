<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { AlertTriangle, Save, RefreshCw, RotateCcw } from "lucide-vue-next";
import { useAuditLog } from "../hooks/useAuditLog";
import { isTauri } from "../lib/tauri";

const { log } = useAuditLog();

interface Entry { key: string; value: string; description: string; section: string }
const entries = ref<Entry[]>([]);
const loading = ref(false);
const rawContent = ref("");

const defaultEntries: Entry[] = [
  { key: "networkingMode", value: "NAT", description: "Network mode (NAT or mirrored)", section: "wsl2" },
  { key: "memory", value: "8GB", description: "Maximum memory allocation", section: "wsl2" },
  { key: "processors", value: "4", description: "Number of logical processors", section: "wsl2" },
  { key: "swap", value: "4GB", description: "Swap file size", section: "wsl2" },
  { key: "localhostForwarding", value: "true", description: "Enable localhost forwarding", section: "wsl2" },
  { key: "dnsTunneling", value: "true", description: "Enable DNS tunneling", section: "experimental" },
  { key: "autoProxy", value: "true", description: "Auto-configure proxy settings", section: "experimental" },
];

const descriptions: Record<string, string> = {
  networkingMode: "Network mode (NAT or mirrored)", memory: "Maximum memory allocation",
  processors: "Number of logical processors", swap: "Swap file size",
  localhostForwarding: "Enable localhost forwarding", dnsTunneling: "Enable DNS tunneling",
  autoProxy: "Auto-configure proxy settings",
};

function parseWslconfig(content: string): Entry[] {
  const result: Entry[] = [];
  let section = "wsl2";
  for (const line of content.split("\n")) {
    const trimmed = line.trim();
    if (trimmed.startsWith("[")) { section = trimmed.replace(/[[\]]/g, ""); continue; }
    if (!trimmed || trimmed.startsWith("#")) continue;
    const [key, ...rest] = trimmed.split("=");
    if (key && rest.length) {
      const k = key.trim();
      result.push({ key: k, value: rest.join("=").trim(), description: descriptions[k] ?? "", section });
    }
  }
  return result.length ? result : defaultEntries;
}

function toWslconfig(): string {
  const sections: Record<string, Entry[]> = {};
  for (const e of entries.value) (sections[e.section] ??= []).push(e);
  let out = "";
  for (const [sec, items] of Object.entries(sections)) {
    out += `[${sec}]\n`;
    for (const e of items) out += `${e.key}=${e.value}\n`;
    out += "\n";
  }
  return out;
}

async function load() {
  loading.value = true;
  if (!isTauri) { entries.value = defaultEntries; loading.value = false; return; }
  try {
    const { readWslconfig } = await import("../hooks/useTauri");
    rawContent.value = await readWslconfig();
    entries.value = parseWslconfig(rawContent.value);
    // If file was empty or didn't exist, show defaults so user can configure
    if (!entries.value.length) entries.value = defaultEntries;
  } catch { entries.value = defaultEntries; }
  loading.value = false;
}

async function save() {
  if (!isTauri) return;
  try {
    const { writeWslconfig } = await import("../hooks/useTauri");
    await writeWslconfig(toWslconfig());
    log("wslconfig.save", "Saved .wslconfig");
  } catch (e) { log("wslconfig.save", `Failed: ${e}`, "error"); }
}

async function restartWsl() {
  if (!isTauri) return;
  try {
    const { invoke } = await import("@tauri-apps/api/core");
    await invoke("plugin:shell|execute", { program: "wsl", args: ["--shutdown"] });
    log("wslconfig.restart", "WSL shutdown initiated");
  } catch (e) { log("wslconfig.restart", `Failed: ${e}`, "error"); }
}

onMounted(load);

const warnings = computed(() => {
  const w: string[] = [];
  if (entries.value.find((e) => e.key === "networkingMode")?.value.toLowerCase() === "mirrored") {
    w.push("Mirrored mode + full-tunnel VPN may break WSL networking.");
    w.push("Mirrored mode on Windows Server 2025 silently falls back to NAT.");
  }
  return w;
});

const sections = computed(() => {
  const m: Record<string, Entry[]> = {};
  for (const e of entries.value) (m[e.section] ??= []).push(e);
  return m;
});
</script>

<template>
  <div>
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-lg font-semibold" :style="{ color: 'var(--text-primary)' }">.wslconfig Inspector</h2>
      <div class="flex items-center gap-2">
        <button @click="load" class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg" :style="{ color: 'var(--text-secondary)', border: '1px solid var(--border)' }"><RefreshCw :size="12" :class="{ 'animate-spin': loading }" /> Reload</button>
        <button @click="save" class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg font-medium" :style="{ background: 'var(--accent)', color: 'var(--bg-primary)' }"><Save :size="12" /> Save</button>
        <button @click="restartWsl" class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg" :style="{ color: 'var(--status-warn)', border: '1px solid var(--status-warn)' }" title="Shutdown WSL to apply .wslconfig changes (wsl --shutdown)"><RotateCcw :size="12" /> Restart WSL</button>
      </div>
    </div>
    <div v-if="warnings.length" class="rounded-lg p-3 mb-4" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--status-warn)' }">
      <div v-for="(w, i) in warnings" :key="i" class="flex items-start gap-2 text-xs mb-1 last:mb-0" :style="{ color: 'var(--status-warn)' }">
        <AlertTriangle :size="12" class="mt-0.5 shrink-0" /><span>{{ w }}</span>
      </div>
    </div>
    <div v-for="(items, sec) in sections" :key="sec" class="mb-6">
      <h3 class="text-xs font-semibold uppercase tracking-wider mb-2" :style="{ color: 'var(--text-secondary)' }">[{{ sec }}]</h3>
      <div class="space-y-1.5">
        <div v-for="e in items" :key="e.key" class="flex items-center gap-3 px-4 py-2 rounded-lg" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }">
          <span class="text-sm font-mono w-44" :style="{ color: 'var(--accent)' }">{{ e.key }}</span>
          <input v-model="e.value" class="w-32 px-2 py-1 text-sm rounded outline-none" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border)' }" />
          <span class="text-xs flex-1" :style="{ color: 'var(--text-secondary)' }">{{ e.description }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
