<template>
  <div :class="['stat-card', variant]">
    <div class="stat-icon">
      <i :class="icon"></i>
    </div>
    <div class="stat-content">
      <div class="stat-value">{{ value }}</div>
      <div class="stat-label">{{ label }}</div>
      <div v-if="subtitle" class="stat-subtitle">{{ subtitle }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  icon: string;
  value: string | number;
  label: string;
  subtitle?: string;
  variant?: 'primary' | 'secondary' | 'success' | 'warning' | 'danger' | 'info';
}

withDefaults(defineProps<Props>(), {
  variant: 'primary'
});
</script>

<style scoped>
.stat-card {
  background: white;
  border-radius: var(--border-radius);
  box-shadow: var(--shadow-md);
  padding: 1.5rem;
  display: flex;
  gap: 1rem;
  align-items: center;
  transition: transform 0.2s, box-shadow 0.2s;
}

.stat-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-lg);
}

.stat-icon {
  width: 48px;
  height: 48px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.5rem;
  flex-shrink: 0;
}

.stat-card.primary .stat-icon {
  background: #eff6ff;
  color: #3b82f6;
}

.stat-card.secondary .stat-icon {
  background: #f3f4f6;
  color: #6b7280;
}

.stat-card.success .stat-icon {
  background: #ecfdf5;
  color: #10b981;
}

.stat-card.warning .stat-icon {
  background: #fffbeb;
  color: #f59e0b;
}

.stat-card.danger .stat-icon {
  background: #fef2f2;
  color: #ef4444;
}

.stat-card.info .stat-icon {
  background: #ecfeff;
  color: #06b6d4;
}

.stat-content {
  flex: 1;
  min-width: 0;
}

.stat-value {
  font-size: 2rem;
  font-weight: 700;
  color: var(--text-primary);
  line-height: 1.2;
}

.stat-label {
  color: var(--text-secondary);
  font-size: 0.875rem;
  margin-top: 0.25rem;
}

.stat-subtitle {
  color: var(--text-light);
  font-size: 0.75rem;
  margin-top: 0.25rem;
}
</style>

