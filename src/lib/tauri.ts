/// Whether the app is running inside Tauri (vs browser dev mode).
/// Tauri v2 uses __TAURI_INTERNALS__, while withGlobalTauri adds __TAURI__.
export const isTauri = typeof window !== "undefined" && ("__TAURI__" in window || "__TAURI_INTERNALS__" in window);
