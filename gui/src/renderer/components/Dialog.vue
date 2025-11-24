<template>
  <div v-if="isOpen" class="fixed inset-0 bg-black/50 flex items-center justify-center z-modal" @click.self="handleBackdropClick">
    <div class="bg-bg-secondary border border-border rounded-xl w-full max-w-md mx-4 shadow-xl">
      <div class="flex items-center justify-between p-6 border-b border-border">
        <h2 class="text-lg font-semibold text-text-primary">{{ title }}</h2>
        <button v-if="showClose" @click="close" class="p-2 text-text-secondary hover:text-text-primary hover:bg-bg-hover rounded transition-colors">
          <i class="fas fa-times"></i>
        </button>
      </div>
      
      <div class="p-6">
        <p class="text-sm text-text-primary">{{ message }}</p>
      </div>
      
      <div class="flex items-center justify-end gap-2 p-6 border-t border-border">
        <button v-if="type === 'confirm'" @click="handleCancel" class="px-4 py-2 text-sm font-medium bg-transparent text-text-secondary border border-border rounded hover:bg-bg-hover hover:text-text-primary transition-colors">
          {{ cancelText }}
        </button>
        <button @click="handleConfirm" class="px-4 py-2 text-sm font-medium bg-bg-tertiary text-text-primary border border-border rounded hover:bg-bg-hover transition-colors">
          {{ confirmText }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

interface Props {
  title?: string;
  message?: string;
  type?: 'alert' | 'confirm';
  confirmText?: string;
  cancelText?: string;
  showClose?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  title: 'Notification',
  message: '',
  type: 'alert',
  confirmText: 'OK',
  cancelText: 'Cancel',
  showClose: true
});

const emit = defineEmits<{
  (e: 'confirm'): void;
  (e: 'cancel'): void;
  (e: 'close'): void;
}>();

const isOpen = ref(false);

function open(): void {
  isOpen.value = true;
}

function close(): void {
  isOpen.value = false;
  emit('close');
}

function handleConfirm(): void {
  emit('confirm');
  close();
}

function handleCancel(): void {
  emit('cancel');
  close();
}

function handleBackdropClick(): void {
  if (props.showClose) {
    close();
  }
}

defineExpose({
  open,
  close
});
</script>

