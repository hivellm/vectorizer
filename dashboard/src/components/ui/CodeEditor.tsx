/**
 * Monaco Code Editor component with lazy loading
 * Install with: npm install @monaco-editor/react monaco-editor
 */

import { lazy, Suspense, useState, useEffect } from 'react';
import { useTheme } from '@/providers/ThemeProvider';

// Lazy load Monaco Editor for code splitting
const MonacoEditor = lazy(() => import('@monaco-editor/react'));

interface CodeEditorProps {
  value: string;
  onChange?: (value: string | undefined) => void;
  language?: string;
  height?: string;
  readOnly?: boolean;
  theme?: 'light' | 'dark' | 'vs-dark';
  options?: Record<string, unknown>;
}

// Fallback textarea component
function TextareaEditor({
  value,
  onChange,
  height,
  readOnly,
}: {
  value: string;
  onChange?: (value: string | undefined) => void;
  height: string;
  readOnly: boolean;
}) {
  return (
    <div className="border border-neutral-200 dark:border-neutral-800 rounded-lg overflow-hidden bg-white dark:bg-neutral-900">
      <div className="relative">
        <textarea
          value={value}
          onChange={(e) => onChange?.(e.target.value)}
          readOnly={readOnly}
          className="w-full p-4 font-mono text-sm leading-relaxed bg-transparent text-neutral-900 dark:text-neutral-100 border-none outline-none resize-none"
          style={{ 
            height, 
            minHeight: height,
            tabSize: 2,
          }}
          spellCheck={false}
          wrap="off"
        />
      </div>
    </div>
  );
}

export default function CodeEditor({
  value,
  onChange,
  language = 'json',
  height = '400px',
  readOnly = false,
  theme,
  options = {},
}: CodeEditorProps) {
  const { theme: appTheme } = useTheme();
  const [useMonaco, setUseMonaco] = useState(false);
  const [monacoError, setMonacoError] = useState(false);
  
  useEffect(() => {
    // Try to load Monaco Editor
    import('@monaco-editor/react')
      .then(() => setUseMonaco(true))
      .catch(() => setMonacoError(true));
  }, []);

  const editorTheme = theme || (appTheme === 'dark' ? 'vs-dark' : 'light');

  const defaultOptions = {
    readOnly,
    minimap: { enabled: false },
    scrollBeyondLastLine: false,
    fontSize: 14,
    lineNumbers: 'on' as const,
    roundedSelection: false,
    cursorStyle: 'line' as const,
    automaticLayout: true,
    tabSize: 2,
    wordWrap: 'on' as const,
    formatOnPaste: true,
    formatOnType: true,
    ...options,
  };

  // Use Monaco if available, otherwise fallback to textarea
  if (monacoError || !useMonaco) {
    return <TextareaEditor value={value} onChange={onChange} height={height} readOnly={readOnly} />;
  }

  return (
    <div className="border border-neutral-200 dark:border-neutral-800 rounded-lg overflow-hidden">
      <Suspense fallback={
        <div className="flex items-center justify-center h-full text-sm text-neutral-500 dark:text-neutral-400" style={{ height }}>
          Loading editor...
        </div>
      }>
        <MonacoEditor
          height={height}
          language={language}
          value={value}
          onChange={onChange}
          theme={editorTheme}
          options={defaultOptions}
          loading={<div className="flex items-center justify-center h-full text-sm text-neutral-500 dark:text-neutral-400">Loading editor...</div>}
        />
      </Suspense>
    </div>
  );
}
