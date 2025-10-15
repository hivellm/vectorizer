<template>
  <div :class="[
    'bg-bg-secondary border border-border rounded-xl shadow-md p-6 flex gap-4 items-center transition-all hover:-translate-y-0.5 hover:shadow-lg hover:bg-bg-hover hover:border-border-light',
    {
      'hover:bg-info/10': variant === 'primary' || variant === 'info',
      'hover:bg-bg-hover': variant === 'secondary',
      'hover:bg-success/10': variant === 'success',
      'hover:bg-warning/10': variant === 'warning',
      'hover:bg-error/10': variant === 'danger'
    }
  ]">
    <div :class="[
      'w-12 h-12 rounded-lg flex items-center justify-center text-2xl flex-shrink-0',
      {
        'bg-blue-500/20 text-blue-400': variant === 'primary' || variant === 'info',
        'bg-bg-tertiary text-text-secondary': variant === 'secondary',
        'bg-green-500/20 text-green-400': variant === 'success',
        'bg-yellow-500/20 text-yellow-400': variant === 'warning',
        'bg-red-500/20 text-red-400': variant === 'danger'
      }
    ]">
      <i :class="icon"></i>
    </div>
    <div class="flex-1 min-w-0">
      <div class="text-3xl font-bold text-text-primary leading-tight">{{ value }}</div>
      <div class="text-sm text-text-secondary mt-1">{{ label }}</div>
      <div v-if="subtitle" class="text-xs text-text-muted mt-1">{{ subtitle }}</div>
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

