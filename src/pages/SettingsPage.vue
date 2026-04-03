<script setup lang="ts">
import { Info } from "lucide-vue-next";
</script>

<template>
  <div>
    <h2 class="text-lg font-semibold mb-4" :style="{ color: 'var(--text-primary)' }">Settings</h2>
    <div class="space-y-4">
      <!-- General -->
      <div class="rounded-lg overflow-hidden" :style="{ border: '1px solid var(--border)' }">
        <div class="px-4 py-2 text-xs font-semibold uppercase tracking-wider" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-secondary)' }">General</div>
        <div :style="{ background: 'var(--bg-secondary)' }">
          <div class="flex items-center justify-between px-4 py-3" :style="{ borderBottom: '1px solid var(--border)' }">
            <div><div class="text-sm" :style="{ color: 'var(--text-primary)' }">Start minimized to tray</div><div class="text-xs" :style="{ color: 'var(--text-secondary)' }">Launch in the system tray</div></div>
            <input type="checkbox" class="accent-[var(--accent)]" />
          </div>
          <div class="flex items-center justify-between px-4 py-3" :style="{ borderBottom: '1px solid var(--border)' }">
            <div><div class="text-sm" :style="{ color: 'var(--text-primary)' }">Minimize to tray on close</div><div class="text-xs" :style="{ color: 'var(--text-secondary)' }">Keep running in background</div></div>
            <input type="checkbox" checked class="accent-[var(--accent)]" />
          </div>
          <div class="flex items-center justify-between px-4 py-3">
            <div><div class="text-sm" :style="{ color: 'var(--text-primary)' }">Health check interval</div><div class="text-xs" :style="{ color: 'var(--text-secondary)' }">Seconds between TCP health checks</div></div>
            <input type="number" value="60" class="w-20 text-sm px-2 py-1 rounded text-right" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border)' }" />
          </div>
        </div>
      </div>

      <!-- Networking -->
      <div class="rounded-lg overflow-hidden" :style="{ border: '1px solid var(--border)' }">
        <div class="px-4 py-2 text-xs font-semibold uppercase tracking-wider" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-secondary)' }">Networking</div>
        <div :style="{ background: 'var(--bg-secondary)' }">
          <div class="flex items-center justify-between px-4 py-3" :style="{ borderBottom: '1px solid var(--border)' }">
            <div><div class="text-sm" :style="{ color: 'var(--text-primary)' }">IP settle delay</div><div class="text-xs" :style="{ color: 'var(--text-secondary)' }">Seconds to wait after WSL IP change</div></div>
            <input type="number" value="5" class="w-20 text-sm px-2 py-1 rounded text-right" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border)' }" />
          </div>
          <div class="flex items-center justify-between px-4 py-3" :style="{ borderBottom: '1px solid var(--border)' }">
            <div><div class="text-sm" :style="{ color: 'var(--text-primary)' }">Polling interval</div><div class="text-xs" :style="{ color: 'var(--text-secondary)' }">Fallback polling in seconds</div></div>
            <input type="number" value="30" class="w-20 text-sm px-2 py-1 rounded text-right" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border)' }" />
          </div>
          <div class="flex items-center justify-between px-4 py-3">
            <div><div class="text-sm" :style="{ color: 'var(--text-primary)' }">Default listen address</div><div class="text-xs" :style="{ color: 'var(--text-secondary)' }">Default bind address for new rules</div></div>
            <select class="text-sm px-2 py-1 rounded" :style="{ background: 'var(--bg-tertiary)', color: 'var(--text-primary)', border: '1px solid var(--border)' }">
              <option value="0.0.0.0">0.0.0.0 (LAN)</option><option value="127.0.0.1">127.0.0.1 (local)</option>
            </select>
          </div>
        </div>
      </div>

      <!-- About -->
      <div class="rounded-lg p-4" :style="{ background: 'var(--bg-secondary)', border: '1px solid var(--border)' }">
        <div class="flex items-center gap-2 mb-2"><Info :size="14" :style="{ color: 'var(--accent)' }" /><h3 class="text-sm font-semibold" :style="{ color: 'var(--text-primary)' }">About</h3></div>
        <div class="text-xs space-y-1" :style="{ color: 'var(--text-secondary)' }">
          <p><strong>WSL PortHole</strong> v0.1.0</p>
          <p>Manages netsh portproxy rules between Windows, WSL2, and Docker.</p>
          <p>Built with Tauri v2 + Vue 3 + Rust</p>
        </div>
      </div>
    </div>
  </div>
</template>
