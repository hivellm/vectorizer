import { ref } from 'vue';

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface Toast {
  id: string;
  message: string;
  type: ToastType;
  duration: number;
}

const toasts = ref<Toast[]>([]);
let toastId = 0;

export function useToast() {
  const show = (message: string, type: ToastType = 'info', duration = 3000): void => {
    const id = `toast-${toastId++}`;
    const toast: Toast = {
      id,
      message,
      type,
      duration
    };

    toasts.value.push(toast);

    setTimeout(() => {
      remove(id);
    }, duration);
  };

  const remove = (id: string): void => {
    toasts.value = toasts.value.filter(t => t.id !== id);
  };

  const success = (message: string, duration?: number): void => {
    show(message, 'success', duration);
  };

  const error = (message: string, duration?: number): void => {
    show(message, 'error', duration);
  };

  const warning = (message: string, duration?: number): void => {
    show(message, 'warning', duration);
  };

  const info = (message: string, duration?: number): void => {
    show(message, 'info', duration);
  };

  return {
    toasts,
    show,
    remove,
    success,
    error,
    warning,
    info
  };
}

