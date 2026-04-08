import { ref, onUnmounted } from "vue";

/**
 * Returns a ref that is `true` while the component is mounted and `false`
 * after it unmounts. Use it to guard async callbacks so they don't update
 * state on a page the user has already navigated away from.
 */
export function useAlive() {
  const alive = ref(true);
  onUnmounted(() => { alive.value = false; });
  return alive;
}
