<template>
  <Teleport to="body">
    <transition name="modal">
      <div v-if="modelValue" class="modal-overlay" @click="handleOverlayClick">
        <div class="modal-dialog" :class="sizeClass" @click.stop>
          <div class="modal-content">
            <div v-if="!hideHeader" class="modal-header">
              <h2 v-if="title" class="modal-title">
                <i v-if="icon" :class="icon"></i>
                {{ title }}
              </h2>
              <slot name="header"></slot>
              <button v-if="closeable" @click="close" class="modal-close">
                <i class="fas fa-times"></i>
              </button>
            </div>
            
            <div class="modal-body">
              <slot></slot>
            </div>
            
            <div v-if="!hideFooter" class="modal-footer">
              <slot name="footer">
                <button @click="close" class="btn btn-secondary">Close</button>
              </slot>
            </div>
          </div>
        </div>
      </div>
    </transition>
  </Teleport>
</template>

<script setup lang="ts">
import { computed } from 'vue';

interface Props {
  modelValue: boolean;
  title?: string;
  icon?: string;
  size?: 'small' | 'medium' | 'large' | 'full';
  closeable?: boolean;
  closeOnOverlay?: boolean;
  hideHeader?: boolean;
  hideFooter?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  size: 'medium',
  closeable: true,
  closeOnOverlay: true,
  hideHeader: false,
  hideFooter: false
});

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  close: [];
}>();

const sizeClass = computed(() => `modal-${props.size}`);

function close(): void {
  emit('update:modelValue', false);
  emit('close');
}

function handleOverlayClick(): void {
  if (props.closeOnOverlay) {
    close();
  }
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9998;
  padding: 2rem;
}

.modal-dialog {
  background: white;
  border-radius: var(--border-radius);
  box-shadow: var(--shadow-lg);
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.modal-small {
  max-width: 400px;
  width: 100%;
}

.modal-medium {
  max-width: 600px;
  width: 100%;
}

.modal-large {
  max-width: 900px;
  width: 100%;
}

.modal-full {
  max-width: 95vw;
  width: 100%;
}

.modal-content {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.5rem;
  border-bottom: 1px solid var(--border-color);
}

.modal-title {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.modal-close {
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0.5rem;
  font-size: 1.5rem;
  line-height: 1;
  transition: color 0.2s;
}

.modal-close:hover {
  color: var(--text-primary);
}

.modal-body {
  flex: 1;
  overflow-y: auto;
  padding: 1.5rem;
}

.modal-footer {
  padding: 1.5rem;
  border-top: 1px solid var(--border-color);
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
}

/* Transitions */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-active .modal-dialog,
.modal-leave-active .modal-dialog {
  transition: transform 0.3s ease;
}

.modal-enter-from .modal-dialog,
.modal-leave-to .modal-dialog {
  transform: scale(0.95);
}
</style>

