import { ref, watch } from "vue";
import type { ThemeTokens } from "../types";
import { themes } from "../themes/themes";

const STORAGE_KEY = "wsl-porthole-theme";

const currentTheme = ref<string>(
  (() => {
    try {
      return localStorage.getItem(STORAGE_KEY) || "mission-control";
    } catch {
      return "mission-control";
    }
  })()
);

function applyTheme(tokens: ThemeTokens) {
  const root = document.documentElement;
  for (const [key, value] of Object.entries(tokens)) {
    root.style.setProperty(key, value);
  }
}

// Apply on load
const initial = themes.find((t) => t.name === currentTheme.value);
if (initial) applyTheme(initial.tokens);

watch(currentTheme, (name) => {
  const theme = themes.find((t) => t.name === name);
  if (theme) {
    applyTheme(theme.tokens);
    try {
      localStorage.setItem(STORAGE_KEY, name);
    } catch {}
  }
});

export function useTheme() {
  return { currentTheme, themes, setTheme: (name: string) => (currentTheme.value = name) };
}
