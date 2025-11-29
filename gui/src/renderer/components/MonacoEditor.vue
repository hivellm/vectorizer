<template>
  <div v-if="!useFallback" ref="editorContainer" class="monaco-editor-container"></div>
  <textarea 
    v-else
    v-model="fallbackValue"
    @input="handleFallbackInput"
    :readonly="readOnly"
    :style="{ height: height }"
    class="fallback-editor"
  ></textarea>
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
const useFallback = ref(false);
const fallbackValue = ref(props.value);

function handleFallbackInput(event: Event) {
  const target = event.target as HTMLTextAreaElement;
  emit('update:value', target.value);
  emit('change', target.value);
}

// Initialize Monaco Editor
async function initEditor() {
  if (!editorContainer.value) return;

  try {
    // Configure Monaco loader to use CDN in production
    loader.config({
      paths: {
        vs: 'https://cdn.jsdelivr.net/npm/monaco-editor@0.45.0/min/vs'
      }
    });
    
    // Load Monaco Editor using the loader
    const monaco = await loader.init();
    
    // Check if monaco is loaded correctly
    if (!monaco || !monaco.editor) {
      throw new Error('Monaco editor not loaded properly');
    }
    
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
    console.error('Failed to initialize Monaco Editor, using fallback:', error);
    useFallback.value = true;
    fallbackValue.value = props.value;
  }
}

// Update editor value when prop changes
watch(() => props.value, (newValue: string) => {
  if (useFallback.value) {
    fallbackValue.value = newValue;
  } else if (editor && editor.getValue() !== newValue) {
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

.fallback-editor {
  width: 100%;
  min-height: 300px;
  padding: 12px;
  background-color: #1a1a1a;
  color: #d4d4d4;
  border: 1px solid var(--border);
  border-radius: 6px;
  font-family: 'JetBrains Mono', 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  line-height: 1.5;
  resize: vertical;
  overflow-y: auto;
}

.fallback-editor:focus {
  outline: none;
  border-color: var(--border-light);
}

.fallback-editor[readonly] {
  background-color: #0f0f0f;
  cursor: not-allowed;
}
</style>
