/**
 * File Browser — console design.
 *
 * Directory browser used by the Setup Wizard and the Workspace page
 * to pick a project folder. The imperative selection API
 * (`onSelect`, `onCancel`, `initialPath`) is preserved.
 */

import { useState, useEffect, useCallback, type CSSProperties } from 'react';
import Button from '@/components/ui/Button';
import LoadingSpinner from '@/components/LoadingSpinner';
import { Pill } from '@/components/console';
import {
  Folder,
  FolderCode,
  File06,
  ChevronRight,
  ChevronUp,
  Home03,
  Check,
  RefreshCw01,
} from '@untitledui/icons';

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

  const overlayStyle: CSSProperties = {
    position: 'fixed',
    inset: 0,
    background: 'rgba(0, 0, 0, 0.6)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 50,
    padding: 16,
  };

  const panelStyle: CSSProperties = {
    background: 'var(--bg-1)',
    border: '1px solid var(--border)',
    borderRadius: 12,
    boxShadow: '0 24px 48px rgba(0, 0, 0, 0.4)',
    width: '100%',
    maxWidth: 768,
    maxHeight: '80vh',
    display: 'flex',
    flexDirection: 'column',
  };

  return (
    <div style={overlayStyle} role="dialog" aria-modal aria-label="Select Project Folder">
      <div style={panelStyle} onClick={(e) => e.stopPropagation()}>
        {/* Header */}
        <div
          style={{
            padding: 16,
            borderBottom: '1px solid var(--border)',
            flexShrink: 0,
          }}
        >
          <h2
            style={{
              fontSize: 16,
              fontWeight: 600,
              color: 'var(--text)',
              margin: '0 0 12px',
              letterSpacing: '-0.01em',
            }}
          >
            Select Project Folder
          </h2>

          {/* Path Input */}
          <div style={{ display: 'flex', gap: 8 }}>
            <input
              type="text"
              value={manualPath}
              onChange={(e) => setManualPath(e.target.value)}
              placeholder="/path/to/project"
              className="input mono"
              style={{ flex: 1 }}
              onKeyDown={(e) => e.key === 'Enter' && handleManualPathSubmit()}
            />
            <Button variant="secondary" size="sm" onClick={handleManualPathSubmit}>
              Go
            </Button>
          </div>
        </div>

        {/* Toolbar */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 8,
            padding: '8px 16px',
            background: 'var(--bg-2)',
            borderBottom: '1px solid var(--border)',
            flexShrink: 0,
          }}
        >
          <button
            type="button"
            className="icon-btn"
            onClick={handleGoUp}
            disabled={!parentPath}
            title="Go to parent folder"
            aria-label="Go to parent folder"
          >
            <ChevronUp width={16} height={16} />
          </button>
          <button
            type="button"
            className="icon-btn"
            onClick={handleGoHome}
            title="Go to home folder"
            aria-label="Go to home folder"
          >
            <Home03 width={16} height={16} />
          </button>
          <button
            type="button"
            className="icon-btn"
            onClick={() => fetchDirectory(currentPath)}
            title="Refresh"
            aria-label="Refresh"
          >
            <RefreshCw01 width={16} height={16} />
          </button>
          <div
            style={{
              flex: 1,
              fontSize: 12,
              color: 'var(--text-2)',
              fontFamily: 'var(--font-mono)',
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              whiteSpace: 'nowrap',
            }}
          >
            {currentPath}
          </div>
        </div>

        {/* Content */}
        <div style={{ flex: 1, overflow: 'auto', padding: 8 }}>
          {loading ? (
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                padding: '48px 0',
              }}
            >
              <LoadingSpinner size="lg" />
            </div>
          ) : error ? (
            <div style={{ textAlign: 'center', padding: '48px 0' }}>
              <p style={{ color: 'var(--red)', marginBottom: 16 }}>{error}</p>
              <Button variant="secondary" size="sm" onClick={() => fetchDirectory('')}>
                Go to Home
              </Button>
            </div>
          ) : entries.length === 0 ? (
            <div
              style={{
                textAlign: 'center',
                padding: '48px 0',
                color: 'var(--text-2)',
                fontSize: 13,
              }}
            >
              This folder is empty
            </div>
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
              {entries.map((entry) => {
                const isSelected = selectedPath === entry.path;
                const rowStyle: CSSProperties = {
                  display: 'flex',
                  alignItems: 'center',
                  gap: 12,
                  padding: '8px 12px',
                  borderRadius: 8,
                  cursor: 'pointer',
                  border: '1px solid transparent',
                  background: isSelected ? 'var(--teal-dim)' : 'transparent',
                  borderColor: isSelected ? 'var(--teal)' : 'transparent',
                  transition: 'background 120ms ease, border-color 120ms ease',
                };
                return (
                  <div
                    key={entry.path}
                    onClick={() => handleEntryClick(entry)}
                    onDoubleClick={() => handleEntryDoubleClick(entry)}
                    style={rowStyle}
                    onMouseEnter={(e) => {
                      if (!isSelected) {
                        (e.currentTarget as HTMLDivElement).style.background = 'var(--bg-2)';
                      }
                    }}
                    onMouseLeave={(e) => {
                      if (!isSelected) {
                        (e.currentTarget as HTMLDivElement).style.background = 'transparent';
                      }
                    }}
                  >
                    {/* Icon */}
                    <div style={{ flexShrink: 0, display: 'flex', alignItems: 'center' }}>
                      {entry.is_directory ? (
                        entry.is_project ? (
                          <FolderCode
                            width={20}
                            height={20}
                            style={{ color: 'var(--teal-hi)' }}
                          />
                        ) : (
                          <Folder
                            width={20}
                            height={20}
                            style={{ color: 'var(--amber)' }}
                          />
                        )
                      ) : (
                        <File06
                          width={20}
                          height={20}
                          style={{ color: 'var(--text-2)' }}
                        />
                      )}
                    </div>

                    {/* Name */}
                    <div style={{ flex: 1, minWidth: 0 }}>
                      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                        <span
                          style={{
                            fontSize: 13,
                            color: entry.is_project ? 'var(--teal-hi)' : 'var(--text)',
                            fontWeight: entry.is_project ? 500 : 400,
                            overflow: 'hidden',
                            textOverflow: 'ellipsis',
                            whiteSpace: 'nowrap',
                          }}
                        >
                          {entry.name}
                        </span>
                        {entry.is_project && <Pill tone="teal">Project</Pill>}
                      </div>
                    </div>

                    {/* Size / Navigate */}
                    <div
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: 8,
                        fontSize: 12,
                        color: 'var(--text-2)',
                      }}
                    >
                      {!entry.is_directory && entry.size && (
                        <span>{formatSize(entry.size)}</span>
                      )}
                      {entry.is_directory && <ChevronRight width={16} height={16} />}
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>

        {/* Selected Path Display */}
        {selectedPath && (
          <div
            style={{
              padding: '10px 16px',
              background: 'var(--teal-dim)',
              borderTop: '1px solid var(--border)',
              flexShrink: 0,
            }}
          >
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: 8,
                fontSize: 12,
                color: 'var(--teal-hi)',
                minWidth: 0,
              }}
            >
              <Check width={16} height={16} />
              <span>Selected:</span>
              <span
                style={{
                  fontFamily: 'var(--font-mono)',
                  color: 'var(--text)',
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                  whiteSpace: 'nowrap',
                  minWidth: 0,
                }}
              >
                {selectedPath}
              </span>
            </div>
          </div>
        )}

        {/* Footer */}
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            gap: 12,
            padding: 16,
            borderTop: '1px solid var(--border)',
            flexShrink: 0,
          }}
        >
          <p style={{ fontSize: 11, color: 'var(--text-2)', margin: 0 }}>
            Double-click to navigate, single-click to select
          </p>
          <div style={{ display: 'flex', gap: 8 }}>
            <Button variant="secondary" onClick={onCancel}>
              Cancel
            </Button>
            <Button
              variant="primary"
              onClick={handleConfirmSelection}
              disabled={!selectedPath && !currentPath}
              leftIcon={<Check width={16} height={16} />}
            >
              Select {selectedPath ? 'Folder' : 'Current Folder'}
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}

export default FileBrowser;
