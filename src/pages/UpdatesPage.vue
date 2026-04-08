<script setup lang="ts">
import { ref, onMounted } from "vue";
import { Download, RefreshCw, CheckCircle, AlertCircle, Info } from "lucide-vue-next";
import { isTauri } from "../lib/tauri";
import { useToast } from "../hooks/useToast";
import { useAlive } from "../hooks/useAlive";

declare const __APP_VERSION__: string;

const alive = useAlive();
const { show: showToast } = useToast();

const currentVersion = ref("...");
const latestVersion = ref<string | null>(null);
const checking = ref(false);
const error = ref<string | null>(null);
const lastChecked = ref<string | null>(null);
const autoUpdate = ref(true);
const downloading = ref(false);
const downloadProgress = ref("");

async function loadVersion() {
  if (isTauri) {
    try {
      const { getVersion } = await import("@tauri-apps/api/app");
      currentVersion.value = await getVersion();
    } catch { currentVersion.value = __APP_VERSION__; }
  } else {
    currentVersion.value = __APP_VERSION__;
  }
}

async function checkForUpdates() {
  checking.value = true;
  error.value = null;
  try {
    // Check GitHub releases API directly for the latest version
    if (isTauri) {
      const { checkForAppUpdates } = await import("../hooks/useTauri");
      const result = await checkForAppUpdates();
      if (!alive.value) return;
      latestVersion.value = result;
    }
    lastChecked.value = new Date().toLocaleTimeString();
    if (latestVersion.value) {
      showToast(`Update available: v${latestVersion.value}`, "info");
    } else {
      showToast("You're on the latest version", "success");
    }
  } catch (e: any) {
    if (!alive.value) return;
    error.value = `Update check failed: ${e}`;
    showToast(error.value, "error");
  }
  checking.value = false;
}

const hasUpdate = () => latestVersion.value && latestVersion.value !== currentVersion.value;

async function handleDownloadUpdate() {
  if (!latestVersion.value || !isTauri) return;
  downloading.value = true;
  downloadProgress.value = "Downloading installer...";
  try {
    const { downloadAndInstallUpdate } = await import("../hooks/useTauri");
    downloadProgress.value = "Installing — the app will restart...";
    await downloadAndInstallUpdate(latestVersion.value);
  } catch (e: any) {
    downloading.value = false;
    downloadProgress.value = "";
    error.value = `Update failed: ${e}`;
    showToast(`Update failed: ${e}`, "error");
  }
}

onMounted(() => {
  loadVersion();
  checkForUpdates();
});
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold mb-2" :style="{ color: 'var(--text-primary)' }">Updates</h2>
    <p class="text-sm mb-6" :style="{ color: 'var(--text-secondary)' }">Check for new versions of WSL PortHole.</p>

    <!-- Current version card -->
    <div class="rounded-lg p-5 mb-4" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }">
      <div class="flex items-center gap-3 mb-4">
        <component :is="hasUpdate() ? Download : CheckCircle" :size="20"
          :style="{ color: hasUpdate() ? 'var(--accent)' : 'var(--status-ok)' }" />
        <div>
          <div class="text-sm font-medium" :style="{ color: 'var(--text-primary)' }">
            {{ hasUpdate() ? 'Update available!' : 'You\'re up to date' }}
          </div>
          <div class="text-xs" :style="{ color: 'var(--text-secondary)' }">
            Current version: <strong :style="{ color: 'var(--accent)' }">v{{ currentVersion }}</strong>
            <template v-if="hasUpdate()">
              &rarr; <strong :style="{ color: 'var(--status-ok)' }">v{{ latestVersion }}</strong>
            </template>
          </div>
        </div>
        <div class="flex-1" />
        <button @click="checkForUpdates" class="flex items-center gap-1.5 px-3 py-1.5 text-xs rounded-lg"
          :style="{ color: 'var(--accent)', border: '1px solid var(--border)' }"
          :disabled="checking" title="Check for new versions">
          <RefreshCw :size="12" :class="{ 'animate-spin': checking }" />
          {{ checking ? 'Checking...' : 'Check now' }}
        </button>
      </div>

      <div v-if="hasUpdate()" class="mt-3">
        <button @click="handleDownloadUpdate" :disabled="downloading"
          class="flex items-center gap-1.5 px-4 py-2 text-sm rounded-lg font-medium"
          :style="{ background: downloading ? 'var(--accent-dim)' : 'var(--accent)', color: 'var(--bg-primary)' }"
          title="Download and install the update automatically">
          <RefreshCw v-if="downloading" :size="14" class="animate-spin" />
          <Download v-else :size="14" />
          {{ downloading ? downloadProgress : `Install v${latestVersion}` }}
        </button>
      </div>

      <div v-if="lastChecked" class="text-[10px] mt-3" :style="{ color: 'var(--text-secondary)' }">
        Last checked: {{ lastChecked }}
      </div>
    </div>

    <!-- Error -->
    <div v-if="error" class="flex items-center gap-2 p-3 rounded-lg mb-4"
      :style="{ background: 'rgba(248,81,73,0.1)', border: '1px solid var(--status-err)', color: 'var(--status-err)' }">
      <AlertCircle :size="14" /><span class="text-xs">{{ error }}</span>
    </div>

    <!-- Settings -->
    <div class="rounded-lg overflow-hidden" :style="{ border: '1px solid var(--border)' }">
      <div class="px-4 py-2 text-xs font-semibold uppercase tracking-wider"
        :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-secondary)' }">Update Settings</div>
      <div :style="{ background: 'var(--bg-secondary)' }">
        <div class="flex items-center justify-between px-4 py-3">
          <div>
            <div class="text-sm" :style="{ color: 'var(--text-primary)' }">Automatic updates</div>
            <div class="text-xs" :style="{ color: 'var(--text-secondary)' }">Automatically check for and install updates on startup</div>
          </div>
          <input type="checkbox" v-model="autoUpdate" class="accent-[var(--accent)]" />
        </div>
      </div>
    </div>

    <!-- Info -->
    <div class="flex items-start gap-2 mt-4 p-3 rounded-lg" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }">
      <Info :size="14" class="mt-0.5 shrink-0" :style="{ color: 'var(--accent)' }" />
      <div class="text-xs" :style="{ color: 'var(--text-secondary)' }">
        <p>Updates are checked against GitHub releases. When a new version is available, click the download button to open the release page where you can download the latest installer.</p>
      </div>
    </div>
  </div>
</template>
