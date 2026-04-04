import { ref } from "vue";

export interface Toast {
  id: string;
  message: string;
  type: "success" | "error" | "info" | "warn";
  exiting?: boolean;
}

const toasts = ref<Toast[]>([]);

export function useToast() {
  function show(message: string, type: Toast["type"] = "info", duration = 3000) {
    const id = crypto.randomUUID();
    toasts.value.push({ id, message, type });
    setTimeout(() => dismiss(id), duration);
  }

  function dismiss(id: string) {
    const toast = toasts.value.find((t) => t.id === id);
    if (toast) {
      toast.exiting = true;
      setTimeout(() => {
        toasts.value = toasts.value.filter((t) => t.id !== id);
      }, 300);
    }
  }

  return { toasts, show, dismiss };
}
