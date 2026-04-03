/// Whether the app is running inside Tauri (vs browser dev mode).
export const isTauri = typeof window !== "undefined" && "__TAURI__" in window;
