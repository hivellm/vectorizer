<template>
  <div :class="['loading-spinner', size]">
    <i class="fas fa-spinner fa-spin"></i>
    <span v-if="message" class="loading-message">{{ message }}</span>
  </div>
</template>

<script setup lang="ts">
interface Props {
  message?: string;
  size?: 'small' | 'medium' | 'large';
}

withDefaults(defineProps<Props>(), {
  size: 'medium'
});
</script>

<style scoped>
.loading-spinner {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  padding: 2rem;
  color: var(--text-secondary);
}

.loading-spinner.small {
  font-size: 1rem;
  padding: 1rem;
}

.loading-spinner.medium {
  font-size: 2rem;
  padding: 2rem;
}

.loading-spinner.large {
  font-size: 3rem;
  padding: 3rem;
}

.loading-message {
  font-size: 0.875rem;
}
</style>

