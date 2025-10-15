<template>
  <label class="inline-flex items-center gap-2 cursor-pointer select-none">
    <input
      :id="id"
      :checked="modelValue"
      type="checkbox"
      :disabled="disabled"
      @change="handleChange"
      class="checkbox-input"
    />
    <span v-if="label" class="text-sm text-text-primary leading-none">{{ label }}</span>
  </label>
</template>

<script setup lang="ts">
interface Props {
  modelValue: boolean;
  label?: string;
  disabled?: boolean;
  id?: string;
}

interface Emits {
  (e: 'update:modelValue', value: boolean): void;
  (e: 'change', value: boolean): void;
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
  id: undefined
});

const emit = defineEmits<Emits>();

const handleChange = (event: Event) => {
  const target = event.target as HTMLInputElement;
  emit('update:modelValue', target.checked);
  emit('change', target.checked);
};
</script>

<style scoped>
.checkbox-input {
  width: 16px;
  height: 16px;
  border-radius: 2px;
  appearance: none;
  -webkit-appearance: none;
  -moz-appearance: none;
  background-color: var(--color-bg-tertiary);
  border: 1px solid var(--color-border-light);
  cursor: pointer;
  position: relative;
  transition: all 0.2s;
}

.checkbox-input:hover {
  border-color: var(--color-border-focus);
}

.checkbox-input:checked {
  background-color: #3b82f6;
  border-color: #3b82f6;
}

.checkbox-input:checked::after {
  content: '';
  position: absolute;
  left: 5px;
  top: 2px;
  width: 4px;
  height: 8px;
  border: solid white;
  border-width: 0 2px 2px 0;
  transform: rotate(45deg);
}

.checkbox-input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
