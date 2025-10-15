<template>
  <div class="toast-container">
    <transition-group name="toast">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        :class="['toast', `toast-${toast.type}`]"
      >
        <i :class="['fas', getIcon(toast.type)]"></i>
        <span class="toast-message">{{ toast.message }}</span>
        <button @click="remove(toast.id)" class="toast-close">
          <i class="fas fa-times"></i>
        </button>
      </div>
    </transition-group>
  </div>
</template>

<script setup lang="ts">
import { useToast, type ToastType } from '../composables/useToast';

const { toasts, remove } = useToast();

function getIcon(type: ToastType): string {
  const icons: Record<ToastType, string> = {
    success: 'fa-check-circle',
    error: 'fa-exclamation-circle',
    warning: 'fa-exclamation-triangle',
    info: 'fa-info-circle'
  };
  return icons[type];
}
</script>

<style scoped>
.toast-container {
  position: fixed;
  top: 1rem;
  right: 1rem;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.toast {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 1rem 1.5rem;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  min-width: 300px;
  max-width: 500px;
  color: white;
  font-size: 0.875rem;
}

.toast-success {
  background: #10b981;
}

.toast-error {
  background: #ef4444;
}

.toast-warning {
  background: #f59e0b;
}

.toast-info {
  background: #3b82f6;
}

.toast-message {
  flex: 1;
}

.toast-close {
  border: none;
  background: transparent;
  color: white;
  cursor: pointer;
  padding: 0.25rem;
  opacity: 0.7;
  transition: opacity 0.2s;
}

.toast-close:hover {
  opacity: 1;
}

/* Transitions */
.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}

.toast-enter-from {
  transform: translateX(100%);
  opacity: 0;
}

.toast-leave-to {
  transform: translateX(100%);
  opacity: 0;
}
</style>

