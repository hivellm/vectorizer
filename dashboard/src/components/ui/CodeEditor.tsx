/**
 * Monaco Code Editor — console design language.
 *
 * Public API preserved. Monaco's mount logic (lazy import, fallback
 * textarea) is untouched; only the wrapper styling is moved from
 * Tailwind utilities to console palette tokens.
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

const wrapperStyle: React.CSSProperties = {
  border: '1px solid var(--border)',
  borderRadius: 'var(--radius)',
  overflow: 'hidden',
  background: 'var(--bg-2)',
};

const placeholderStyle = (height: string): React.CSSProperties => ({
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  fontSize: 13,
  color: 'var(--text-2)',
  background: 'var(--bg-2)',
  height,
});

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
    <div style={wrapperStyle}>
      <textarea
        value={value}
        onChange={(e) => onChange?.(e.target.value)}
        readOnly={readOnly}
        style={{
          width: '100%',
          padding: 16,
          fontFamily: 'var(--font-mono)',
          fontSize: 13,
          lineHeight: 1.5,
          background: 'transparent',
          color: 'var(--text)',
          border: 'none',
          outline: 'none',
          resize: 'none',
          height,
          minHeight: height,
          tabSize: 2,
          boxSizing: 'border-box',
        }}
        spellCheck={false}
        wrap="off"
      />
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

  // Console design is dark-first; fall back to vs-dark unless an
  // explicit override or the app theme indicates light.
  const editorTheme = theme || (appTheme === 'light' ? 'light' : 'vs-dark');

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
    return (
      <TextareaEditor value={value} onChange={onChange} height={height} readOnly={readOnly} />
    );
  }

  return (
    <div style={wrapperStyle}>
      <Suspense
        fallback={<div style={placeholderStyle(height)}>Loading editor...</div>}
      >
        <MonacoEditor
          height={height}
          language={language}
          value={value}
          onChange={onChange}
          theme={editorTheme}
          options={defaultOptions}
          loading={<div style={placeholderStyle(height)}>Loading editor...</div>}
        />
      </Suspense>
    </div>
  );
}
