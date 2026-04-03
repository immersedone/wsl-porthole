import { ref, computed } from "vue";
import type { AuditEntry } from "../types";

const MAX_ENTRIES = 1000;
const entries = ref<AuditEntry[]>([]);

export function useAuditLog() {
  function log(event: string, detail: string, level: AuditEntry["level"] = "info") {
    entries.value.unshift({
      timestamp: new Date().toISOString(),
      event,
      detail,
      level,
    });
    if (entries.value.length > MAX_ENTRIES) entries.value.length = MAX_ENTRIES;
  }

  function clear() {
    entries.value = [];
  }

  function exportLog() {
    return entries.value
      .map((e) => `[${e.timestamp}] [${e.level.toUpperCase()}] ${e.event}: ${e.detail}`)
      .join("\n");
  }

  return { entries, log, clear, exportLog };
}
