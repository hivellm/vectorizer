import { ref, watch } from 'vue';
import { useDebounceFn } from '@vueuse/core';

export interface AutoSaveOptions<T> {
  data: () => T;
  saveFn: (data: T) => Promise<void>;
  delay?: number;
  onSuccess?: () => void;
  onError?: (error: Error) => void;
}

export function useAutoSave<T>(options: AutoSaveOptions<T>) {
  const {
    data,
    saveFn,
    delay = 3000,
    onSuccess,
    onError
  } = options;

  const isSaving = ref(false);
  const lastSaved = ref<Date | null>(null);
  const error = ref<Error | null>(null);

  const save = async (): Promise<void> => {
    isSaving.value = true;
    error.value = null;

    try {
      await saveFn(data());
      lastSaved.value = new Date();
      onSuccess?.();
    } catch (err) {
      error.value = err instanceof Error ? err : new Error('Unknown error');
      onError?.(error.value);
    } finally {
      isSaving.value = false;
    }
  };

  const debouncedSave = useDebounceFn(save, delay);

  // Watch for changes and trigger auto-save
  watch(data, () => {
    debouncedSave();
  }, { deep: true });

  return {
    isSaving,
    lastSaved,
    error,
    save,
    triggerSave: debouncedSave
  };
}

