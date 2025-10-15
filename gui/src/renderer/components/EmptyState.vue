<template>
  <div class="empty-state">
    <i :class="icon"></i>
    <h3>{{ title }}</h3>
    <p v-if="description">{{ description }}</p>
    <slot name="actions"></slot>
  </div>
</template>

<script setup lang="ts">
interface Props {
  icon: string;
  title: string;
  description?: string;
}

defineProps<Props>();
</script>

<style scoped>
.empty-state {
  text-align: center;
  padding: 4rem 2rem;
  color: var(--text-secondary);
}

.empty-state i {
  font-size: 4rem;
  margin-bottom: 1.5rem;
  color: var(--text-light);
  display: block;
}

.empty-state h3 {
  font-size: 1.25rem;
  margin-bottom: 0.5rem;
  color: var(--text-primary);
}

.empty-state p {
  margin-bottom: 1.5rem;
  max-width: 400px;
  margin-left: auto;
  margin-right: auto;
}
</style>

