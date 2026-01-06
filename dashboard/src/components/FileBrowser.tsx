/**
 * File Browser Component
 * Directory browser for selecting folders in the Setup Wizard
 */

import { useState, useEffect, useCallback } from 'react';
import Button from '@/components/ui/Button';
import LoadingSpinner from '@/components/LoadingSpinner';
import { Folder, FolderCode, File06, ChevronRight, ChevronUp, Home03, Check, RefreshCw01 } from '@untitledui/icons';

interface DirectoryEntry {
  name: string;
  path: string;
  is_directory: boolean;
  size?: number;
  is_project: boolean;
}

interface BrowseResponse {
  current_path: string;
  parent_path: string | null;
  entries: DirectoryEntry[];
  valid: boolean;
  error?: string;
}

interface FileBrowserProps {
  onSelect: (path: string) => void;
  onCancel: () => void;
  initialPath?: string;
}

function FileBrowser({ onSelect, onCancel, initialPath }: FileBrowserProps) {
  const [currentPath, setCurrentPath] = useState<string>(initialPath || '');
  const [entries, setEntries] = useState<DirectoryEntry[]>([]);
  const [parentPath, setParentPath] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [manualPath, setManualPath] = useState(initialPath || '');
  const [selectedPath, setSelectedPath] = useState<string | null>(null);

  const fetchDirectory = useCallback(async (path: string) => {
    setLoading(true);
    setError(null);

    try {
      const response = await fetch('/setup/browse', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path: path || null }),
      });

      if (!response.ok) {
        throw new Error(`HTTP error: ${response.status}`);
      }

      const data: BrowseResponse = await response.json();

      if (!data.valid) {
        setError(data.error || 'Invalid path');
        return;
      }

      setCurrentPath(data.current_path);
      setParentPath(data.parent_path);
      setEntries(data.entries);
      setManualPath(data.current_path);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to browse directory');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchDirectory(initialPath || '');
  }, [fetchDirectory, initialPath]);

  const handleNavigate = (path: string) => {
    setSelectedPath(null);
    fetchDirectory(path);
  };

  const handleGoUp = () => {
    if (parentPath) {
      handleNavigate(parentPath);
    }
  };

  const handleGoHome = () => {
    handleNavigate('');
  };

  const handleManualPathSubmit = () => {
    handleNavigate(manualPath);
  };

  const handleEntryClick = (entry: DirectoryEntry) => {
    if (entry.is_directory) {
      setSelectedPath(entry.path);
    }
  };

  const handleEntryDoubleClick = (entry: DirectoryEntry) => {
    if (entry.is_directory) {
      handleNavigate(entry.path);
    }
  };

  const handleConfirmSelection = () => {
    onSelect(selectedPath || currentPath);
  };

  const formatSize = (bytes?: number): string => {
    if (!bytes) return '';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <div className="bg-white dark:bg-neutral-900 rounded-xl shadow-2xl w-full max-w-3xl max-h-[80vh] flex flex-col">
        {/* Header */}
        <div className="p-4 border-b border-neutral-200 dark:border-neutral-700">
          <h2 className="text-lg font-semibold text-neutral-900 dark:text-white mb-3">
            Select Project Folder
          </h2>

          {/* Path Input */}
          <div className="flex gap-2">
            <input
              type="text"
              value={manualPath}
              onChange={(e) => setManualPath(e.target.value)}
              placeholder="/path/to/project"
              className="flex-1 px-3 py-2 text-sm border border-neutral-300 dark:border-neutral-700 rounded-lg 
                       bg-white dark:bg-neutral-800 text-neutral-900 dark:text-white font-mono"
              onKeyDown={(e) => e.key === 'Enter' && handleManualPathSubmit()}
            />
            <Button variant="secondary" size="sm" onClick={handleManualPathSubmit}>
              Go
            </Button>
          </div>
        </div>

        {/* Toolbar */}
        <div className="flex items-center gap-2 px-4 py-2 bg-neutral-50 dark:bg-neutral-800/50 border-b border-neutral-200 dark:border-neutral-700">
          <button
            onClick={handleGoUp}
            disabled={!parentPath}
            className="p-2 rounded-lg hover:bg-neutral-200 dark:hover:bg-neutral-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            title="Go to parent folder"
          >
            <ChevronUp className="w-4 h-4 text-neutral-600 dark:text-neutral-400" />
          </button>
          <button
            onClick={handleGoHome}
            className="p-2 rounded-lg hover:bg-neutral-200 dark:hover:bg-neutral-700 transition-colors"
            title="Go to home folder"
          >
            <Home03 className="w-4 h-4 text-neutral-600 dark:text-neutral-400" />
          </button>
          <button
            onClick={() => fetchDirectory(currentPath)}
            className="p-2 rounded-lg hover:bg-neutral-200 dark:hover:bg-neutral-700 transition-colors"
            title="Refresh"
          >
            <RefreshCw01 className="w-4 h-4 text-neutral-600 dark:text-neutral-400" />
          </button>
          <div className="flex-1 text-sm text-neutral-600 dark:text-neutral-400 font-mono truncate">
            {currentPath}
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-auto p-2">
          {loading ? (
            <div className="flex items-center justify-center py-12">
              <LoadingSpinner size="lg" />
            </div>
          ) : error ? (
            <div className="text-center py-12">
              <p className="text-red-500 dark:text-red-400 mb-4">{error}</p>
              <Button variant="secondary" size="sm" onClick={() => fetchDirectory('')}>
                Go to Home
              </Button>
            </div>
          ) : entries.length === 0 ? (
            <div className="text-center py-12 text-neutral-500 dark:text-neutral-400">
              This folder is empty
            </div>
          ) : (
            <div className="space-y-0.5">
              {entries.map((entry) => (
                <div
                  key={entry.path}
                  onClick={() => handleEntryClick(entry)}
                  onDoubleClick={() => handleEntryDoubleClick(entry)}
                  className={`flex items-center gap-3 px-3 py-2 rounded-lg cursor-pointer transition-colors ${selectedPath === entry.path
                      ? 'bg-primary-100 dark:bg-primary-900/30 border border-primary-500'
                      : 'hover:bg-neutral-100 dark:hover:bg-neutral-800 border border-transparent'
                    }`}
                >
                  {/* Icon */}
                  <div className="flex-shrink-0">
                    {entry.is_directory ? (
                      entry.is_project ? (
                        <FolderCode className="w-5 h-5 text-primary-500" />
                      ) : (
                        <Folder className="w-5 h-5 text-yellow-500" />
                      )
                    ) : (
                      <File06 className="w-5 h-5 text-neutral-400" />
                    )}
                  </div>

                  {/* Name */}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className={`text-sm truncate ${entry.is_project
                          ? 'font-medium text-primary-700 dark:text-primary-400'
                          : 'text-neutral-900 dark:text-white'
                        }`}>
                        {entry.name}
                      </span>
                      {entry.is_project && (
                        <span className="px-1.5 py-0.5 text-xs bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-400 rounded">
                          Project
                        </span>
                      )}
                    </div>
                  </div>

                  {/* Size / Navigate Arrow */}
                  <div className="flex items-center gap-2 text-sm text-neutral-500 dark:text-neutral-400">
                    {!entry.is_directory && entry.size && (
                      <span>{formatSize(entry.size)}</span>
                    )}
                    {entry.is_directory && (
                      <ChevronRight className="w-4 h-4" />
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Selected Path Display */}
        {selectedPath && (
          <div className="px-4 py-2 bg-primary-50 dark:bg-primary-900/20 border-t border-primary-200 dark:border-primary-800">
            <div className="flex items-center gap-2 text-sm">
              <Check className="w-4 h-4 text-primary-600 dark:text-primary-400" />
              <span className="text-primary-700 dark:text-primary-300">Selected:</span>
              <span className="font-mono text-primary-900 dark:text-primary-100 truncate">
                {selectedPath}
              </span>
            </div>
          </div>
        )}

        {/* Footer */}
        <div className="flex justify-between items-center gap-3 p-4 border-t border-neutral-200 dark:border-neutral-700">
          <p className="text-xs text-neutral-500 dark:text-neutral-400">
            Double-click to navigate, single-click to select
          </p>
          <div className="flex gap-2">
            <Button variant="secondary" onClick={onCancel}>
              Cancel
            </Button>
            <Button
              variant="primary"
              onClick={handleConfirmSelection}
              disabled={!selectedPath && !currentPath}
            >
              <Check className="w-4 h-4 mr-2" />
              Select {selectedPath ? 'Folder' : 'Current Folder'}
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}

export default FileBrowser;
