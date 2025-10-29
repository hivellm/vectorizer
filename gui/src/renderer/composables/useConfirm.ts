import { ref } from 'vue';

interface ConfirmOptions {
  title?: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  type?: 'info' | 'warning' | 'danger';
}

const isOpen = ref(false);
const options = ref<ConfirmOptions | null>(null);
const resolver = ref<((value: boolean) => void) | null>(null);

export function useConfirm() {
  const show = (opts: ConfirmOptions): Promise<boolean> => {
    return new Promise((resolve) => {
      options.value = {
        confirmText: 'Confirm',
        cancelText: 'Cancel',
        type: 'info',
        ...opts
      };
      isOpen.value = true;
      resolver.value = resolve;
    });
  };

  const confirm = (): void => {
    if (resolver.value) {
      resolver.value(true);
      reset();
    }
  };

  const cancel = (): void => {
    if (resolver.value) {
      resolver.value(false);
      reset();
    }
  };

  const reset = (): void => {
    isOpen.value = false;
    options.value = null;
    resolver.value = null;
  };

  return {
    isOpen,
    options,
    show,
    confirm,
    cancel
  };
}

