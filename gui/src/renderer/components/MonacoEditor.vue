<template>
  <div ref="editorContainer" class="monaco-editor-container"></div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue';
import loader from '@monaco-editor/loader';

interface Props {
  value: string;
  language?: string;
  readOnly?: boolean;
  height?: string;
}

const props = withDefaults(defineProps<Props>(), {
  language: 'json',
  readOnly: true,
  height: '300px'
});

const emit = defineEmits<{
  'update:value': [value: string];
  'change': [value: string];
}>();

const editorContainer = ref<HTMLElement>();
let editor: any = null;

// Initialize Monaco Editor
async function initEditor() {
  if (!editorContainer.value) return;

  try {
    // Load Monaco Editor using the loader
    const monaco = await loader.init();
    
    // Set theme
    monaco.editor.defineTheme('vectorizer-dark', {
      base: 'vs-dark',
      inherit: true,
      rules: [
        { token: 'comment', foreground: '6A9955' },
        { token: 'keyword', foreground: '569CD6' },
        { token: 'string', foreground: 'CE9178' },
        { token: 'number', foreground: 'B5CEA8' },
        { token: 'type', foreground: '4EC9B0' },
        { token: 'function', foreground: 'DCDCAA' },
        { token: 'variable', foreground: '9CDCFE' },
        { token: 'property', foreground: '9CDCFE' }
      ],
      colors: {
        'editor.background': '#1a1a1a',
        'editor.foreground': '#d4d4d4',
        'editor.lineHighlightBackground': '#2a2d2e',
        'editor.selectionBackground': '#264f78',
        'editor.inactiveSelectionBackground': '#3a3d41',
        'editorCursor.foreground': '#aeafad',
        'editor.lineHighlightBorder': '#282828'
      }
    });

    // Create editor
    editor = monaco.editor.create(editorContainer.value, {
      value: props.value,
      language: props.language,
      theme: 'vectorizer-dark',
      readOnly: props.readOnly,
      automaticLayout: true,
      minimap: { enabled: false },
      scrollBeyondLastLine: false,
      wordWrap: 'on',
      lineNumbers: 'on',
      folding: true,
      renderWhitespace: 'selection',
      fontSize: 13,
      fontFamily: 'JetBrains Mono, Consolas, Monaco, monospace'
    });

    // Listen for changes
    editor.onDidChangeModelContent(() => {
      const value = editor.getValue();
      emit('update:value', value);
      emit('change', value);
    });

    // Set container height
    editorContainer.value.style.height = props.height;

  } catch (error) {
    console.error('Failed to initialize Monaco Editor:', error);
  }
}

// Update editor value when prop changes
watch(() => props.value, (newValue: string) => {
  if (editor && editor.getValue() !== newValue) {
    editor.setValue(newValue);
  }
});

// Update language when prop changes
watch(() => props.language, async (newLanguage: string) => {
  if (editor) {
    try {
      const monaco = await loader.init();
      const model = editor.getModel();
      if (model) {
        monaco.editor.setModelLanguage(model, newLanguage);
      }
    } catch (error) {
      console.error('Failed to update language:', error);
    }
  }
});

// Update readOnly when prop changes
watch(() => props.readOnly, (newReadOnly: boolean) => {
  if (editor) {
    editor.updateOptions({ readOnly: newReadOnly });
  }
});

onMounted(async () => {
  await nextTick();
  await initEditor();
});

onUnmounted(() => {
  if (editor) {
    editor.dispose();
  }
});
</script>

<style scoped>
.monaco-editor-container {
  width: 100%;
  border: 1px solid var(--border);
  border-radius: 6px;
  overflow: hidden;
}
</style>
