import { ref } from 'vue';

interface DialogOptions {
  title?: string;
  message: string;
  type?: 'alert' | 'confirm';
  confirmText?: string;
  cancelText?: string;
}

const isDialogOpen = ref(false);
const dialogOptions = ref<DialogOptions>({
  title: 'Notification',
  message: '',
  type: 'alert',
  confirmText: 'OK',
  cancelText: 'Cancel'
});

let resolvePromise: ((value: boolean) => void) | null = null;

export function useDialog() {
  const showDialog = (options: DialogOptions): Promise<boolean> => {
    return new Promise((resolve) => {
      dialogOptions.value = {
        title: options.title || 'Notification',
        message: options.message,
        type: options.type || 'alert',
        confirmText: options.confirmText || 'OK',
        cancelText: options.cancelText || 'Cancel'
      };
      isDialogOpen.value = true;
      resolvePromise = resolve;
    });
  };

  const alert = (message: string, title?: string): Promise<boolean> => {
    return showDialog({ message, title, type: 'alert' });
  };

  const confirm = (message: string, title?: string): Promise<boolean> => {
    return showDialog({ message, title, type: 'confirm' });
  };

  const handleConfirm = () => {
    if (resolvePromise) {
      resolvePromise(true);
      resolvePromise = null;
    }
    isDialogOpen.value = false;
  };

  const handleCancel = () => {
    if (resolvePromise) {
      resolvePromise(false);
      resolvePromise = null;
    }
    isDialogOpen.value = false;
  };

  const close = () => {
    if (resolvePromise) {
      resolvePromise(false);
      resolvePromise = null;
    }
    isDialogOpen.value = false;
  };

  return {
    isDialogOpen,
    dialogOptions,
    alert,
    confirm,
    handleConfirm,
    handleCancel,
    close
  };
}

