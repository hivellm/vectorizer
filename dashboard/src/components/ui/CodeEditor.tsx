/**
 * Monaco Code Editor component
 * Install with: npm install @monaco-editor/react monaco-editor
 */

import { useTheme } from '@/providers/ThemeProvider';

interface CodeEditorProps {
  value: string;
  onChange?: (value: string | undefined) => void;
  language?: string;
  height?: string;
  readOnly?: boolean;
  theme?: 'light' | 'dark' | 'vs-dark';
  options?: any;
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
  
  // Try to use Monaco Editor if available
  try {
    // eslint-disable-next-line @typescript-eslint/no-var-requires
    const Editor = require('@monaco-editor/react').default;
    
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

    return (
      <div className="border border-neutral-200 dark:border-neutral-800 rounded-lg overflow-hidden">
        <Editor
          height={height}
          language={language}
          value={value}
          onChange={onChange}
          theme={editorTheme}
          options={defaultOptions}
          loading={<div className="flex items-center justify-center h-full text-sm text-neutral-500 dark:text-neutral-400">Loading editor...</div>}
        />
      </div>
    );
  } catch (e) {
    // Fallback to styled textarea
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
}
