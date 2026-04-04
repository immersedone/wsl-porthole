<script setup lang="ts">
import { computed } from "vue";
import {
  Network, FolderOpen, Container, Radio, Globe, Shield, Server,
  Zap, HardDrive, FileText, BookOpen, Palette, Settings, Download,
} from "lucide-vue-next";
import type { Component } from "vue";
import type { Page, StatusInfo } from "../types";

defineProps<{ activePage: Page; status: StatusInfo | null }>();
const emit = defineEmits<{ navigate: [page: Page] }>();

interface NavItem { page: Page; label: string; icon: Component; section?: string; tooltip?: string }

const navItems: NavItem[] = [
  { page: "rules", label: "Port Rules", icon: Network, section: "Manage", tooltip: "Manage port forwarding rules" },
  { page: "groups", label: "Groups", icon: FolderOpen, tooltip: "Group rules into profiles for one-click enable/disable" },
  { page: "docker", label: "Docker Sync", icon: Container, tooltip: "Discover Docker containers and forward their ports" },
  { page: "mcp", label: "MCP Servers", icon: Radio, tooltip: "Detect MCP servers on Windows Docker engine" },
  { page: "lan", label: "LAN Access", icon: Globe, section: "Network", tooltip: "View LAN-accessible rules with QR codes" },
  { page: "firewall", label: "Firewall", icon: Shield, tooltip: "Windows Defender firewall rules managed by WSL PortHole" },
  { page: "distros", label: "Distros", icon: Server, tooltip: "Installed WSL distributions and their status" },
  { page: "startup", label: "Startup Actions", icon: Zap, section: "System", tooltip: "Commands to run on WSL startup" },
  { page: "service", label: "Boot Service", icon: HardDrive, tooltip: "Install and manage the WSL PortHole Windows Service" },
  { page: "wslconfig", label: ".wslconfig", icon: FileText, tooltip: "Inspect and edit WSL configuration" },
  { page: "audit", label: "Audit Log", icon: BookOpen, tooltip: "Event log of rule changes and service events" },
  { page: "appearance", label: "Appearance", icon: Palette, section: "Preferences", tooltip: "Theme selection and customization" },
  { page: "updates", label: "Updates", icon: Download, tooltip: "Check for and install app updates" },
  { page: "settings", label: "Settings", icon: Settings, tooltip: "Application preferences and configuration" },
];

// Computed section breaks — no side effects during render
const navWithSections = computed(() => {
  let lastSection = "";
  return navItems.map((item) => {
    const showSection = item.section && item.section !== lastSection;
    if (item.section) lastSection = item.section;
    return { ...item, showSection: !!showSection };
  });
});
</script>

<template>
  <aside class="w-52 h-full flex flex-col border-r"
    :style="{ background: 'var(--bg-secondary)', borderColor: 'var(--border)' }">
    <div class="px-4 py-4 border-b" :style="{ borderColor: 'var(--border)' }">
      <h1 class="text-base font-bold tracking-tight" :style="{ color: 'var(--accent)' }">WSL PortHole</h1>
      <p class="text-xs mt-0.5" :style="{ color: 'var(--text-secondary)' }">Port forwarding manager</p>
    </div>
    <nav class="flex-1 overflow-y-auto py-2">
      <template v-for="item in navWithSections" :key="item.page">
        <div v-if="item.showSection" class="px-4 pt-4 pb-1 text-[10px] font-semibold uppercase tracking-wider"
          :style="{ color: 'var(--text-secondary)' }">{{ item.section }}</div>
        <button @click="emit('navigate', item.page)"
          class="w-full flex items-center gap-2.5 px-4 py-1.5 text-sm transition-colors"
          :style="{ color: activePage === item.page ? 'var(--accent)' : 'var(--text-secondary)', background: activePage === item.page ? 'var(--bg-tertiary)' : 'transparent' }"
          :title="item.tooltip">
          <component :is="item.icon" :size="15" />
          <span>{{ item.label }}</span>
        </button>
      </template>
    </nav>
    <div v-if="status" class="px-4 py-3 text-xs border-t"
      :style="{ borderColor: 'var(--border)', color: 'var(--text-secondary)' }">
      <div class="flex items-center gap-1.5">
        <span class="w-1.5 h-1.5 rounded-full"
          :style="{ background: status.wsl_ip ? 'var(--status-ok)' : 'var(--status-err)' }" />
        <span>{{ status.active_rules }} active rules</span>
      </div>
    </div>
  </aside>
</template>
