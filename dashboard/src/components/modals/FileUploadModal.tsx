/**
 * File Upload Modal — console design.
 * Supports transmutation for PDF and other document formats.
 */

import { useState, useRef, useCallback, useEffect } from 'react';
import { useFileUpload } from '@/hooks/useFileUpload';
import { useCollections } from '@/hooks/useCollections';
import { useAuth } from '@/contexts/AuthContext';
import Modal from '@/components/ui/Modal';
import { Input } from '@/components/ui/Input';
import { Icons } from '@/components/console';
import type { CSSProperties } from 'react';

interface FileUploadModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess?: () => void;
}

const ALERT_BASE: CSSProperties = {
  display: 'flex',
  alignItems: 'flex-start',
  gap: 10,
  padding: 12,
  border: '1px solid var(--border)',
  borderRadius: 6,
};

function FileUploadModal({ isOpen, onClose, onSuccess }: FileUploadModalProps) {
  const { uploadFile, state, reset } = useFileUpload();
  const { listCollections } = useCollections();
  const { isAuthenticated, authRequired } = useAuth();
  const [collections, setCollections] = useState<string[]>([]);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [collectionName, setCollectionName] = useState('');
  const [chunkSize, setChunkSize] = useState<number>(2048);
  const [chunkOverlap, setChunkOverlap] = useState<number>(256);
  const [useTransmutation, setUseTransmutation] = useState(true);
  const [isDragOver, setIsDragOver] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Load collections when modal opens
  const loadCollections = useCallback(async () => {
    try {
      const data = await listCollections();
      const collectionsArray = Array.isArray(data) ? data : [];
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const collectionNames = collectionsArray.map((col: any) => col.name);
      setCollections(collectionNames);
    } catch (error) {
      console.error('Failed to load collections:', error);
    }
  }, [listCollections]);

  // Load collections when modal opens
  useEffect(() => {
    if (isOpen) {
      loadCollections();
    }
  }, [isOpen, loadCollections]);

  // Check if file format supports transmutation
  const isTransmutationSupported = (filename: string): boolean => {
    const ext = filename.toLowerCase().split('.').pop();
    return [
      'pdf', 'docx', 'xlsx', 'pptx', 'html', 'htm', 'xml',
      'jpg', 'jpeg', 'png', 'tiff', 'tif', 'bmp', 'gif', 'webp',
    ].includes(ext || '');
  };

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setSelectedFile(file);
      if (isTransmutationSupported(file.name)) {
        setUseTransmutation(true);
      }
    }
  };

  const handleDrop = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    setIsDragOver(false);
    const file = e.dataTransfer.files[0];
    if (file) {
      setSelectedFile(file);
      if (isTransmutationSupported(file.name)) {
        setUseTransmutation(true);
      }
    }
  };

  const handleDragOver = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    setIsDragOver(true);
  };

  const handleDragLeave = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    setIsDragOver(false);
  };

  const handleUpload = async () => {
    if (!selectedFile || !collectionName.trim()) {
      return;
    }

    try {
      const response = await uploadFile(selectedFile, {
        collectionName: collectionName.trim(),
        chunkSize,
        chunkOverlap,
        useTransmutation:
          useTransmutation && isTransmutationSupported(selectedFile.name),
      });

      if (response.success) {
        // Wait a bit before calling onSuccess to allow state to update
        setTimeout(() => {
          onSuccess?.();
          handleClose();
        }, 500);
      }
    } catch (error) {
      console.error('Upload failed:', error);
    }
  };

  const handleClose = () => {
    reset();
    setSelectedFile(null);
    setCollectionName('');
    setChunkSize(2048);
    setChunkOverlap(256);
    setUseTransmutation(true);
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }
    onClose();
  };

  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  const canUpload =
    selectedFile &&
    collectionName.trim() &&
    !state.uploading &&
    (!authRequired || isAuthenticated);

  const dropZoneStyle: CSSProperties = {
    border: `2px dashed ${
      isDragOver ? 'var(--teal)' : 'var(--border)'
    }`,
    borderRadius: 8,
    padding: '32px 16px',
    textAlign: 'center',
    cursor: 'pointer',
    background: isDragOver ? 'var(--teal-dim)' : 'transparent',
    transition: 'border-color 120ms ease, background 120ms ease',
  };

  return (
    <Modal isOpen={isOpen} onClose={handleClose} title="Upload File">
      <div style={{ display: 'flex', flexDirection: 'column', gap: 22 }}>
        {/* File Selection */}
        <div className="field">
          <label className="field-label">Select File</label>
          <div
            style={dropZoneStyle}
            onDrop={handleDrop}
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onClick={() => fileInputRef.current?.click()}
          >
            <input
              ref={fileInputRef}
              type="file"
              style={{ display: 'none' }}
              onChange={handleFileSelect}
              accept=".pdf,.docx,.xlsx,.pptx,.html,.htm,.xml,.jpg,.jpeg,.png,.tiff,.tif,.bmp,.gif,.webp,.txt,.md,.rs,.py,.js,.ts,.tsx,.jsx,.go,.java,.c,.cpp,.h,.hpp,.cs,.rb,.php,.swift,.kt,.scala,.r,.jl,.lua,.pl,.sh,.bash,.zsh,.fish,.ps1,.bat,.cmd,.json,.yaml,.yml,.toml,.ini,.cfg,.conf,.csv,.tsv"
            />
            {selectedFile ? (
              <div
                style={{
                  display: 'flex',
                  flexDirection: 'column',
                  alignItems: 'center',
                  gap: 8,
                }}
              >
                <div style={{ color: 'var(--teal)' }}>
                  <Icons.layers size={36} />
                </div>
                <div>
                  <div style={{ fontWeight: 500, color: 'var(--text)', fontSize: 13 }}>
                    {selectedFile.name}
                  </div>
                  <div style={{ fontSize: 11, color: 'var(--text-2)', marginTop: 2 }}>
                    {formatFileSize(selectedFile.size)}
                  </div>
                </div>
                <button
                  type="button"
                  className="btn sm"
                  onClick={(e) => {
                    e.stopPropagation();
                    setSelectedFile(null);
                    if (fileInputRef.current) {
                      fileInputRef.current.value = '';
                    }
                  }}
                  style={{ display: 'inline-flex', alignItems: 'center', gap: 6 }}
                >
                  <Icons.x size={12} />
                  Remove
                </button>
              </div>
            ) : (
              <div
                style={{
                  display: 'flex',
                  flexDirection: 'column',
                  alignItems: 'center',
                  gap: 8,
                }}
              >
                <div style={{ color: 'var(--text-2)' }}>
                  <Icons.arrowUp size={36} />
                </div>
                <div>
                  <div style={{ fontWeight: 500, color: 'var(--text)', fontSize: 13 }}>
                    Click to upload or drag and drop
                  </div>
                  <div style={{ fontSize: 11, color: 'var(--text-2)', marginTop: 2 }}>
                    PDF, DOCX, XLSX, PPTX, images, code files, and more
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Collection Selection */}
        <div className="field">
          <label className="field-label">Collection Name</label>
          <Input
            type="text"
            value={collectionName}
            onChange={(e) => setCollectionName(e.target.value)}
            placeholder="Enter collection name"
            list="collections-list"
          />
          <datalist id="collections-list">
            {collections.map((col) => (
              <option key={col} value={col} />
            ))}
          </datalist>
        </div>

        {/* Chunking Options */}
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(2, minmax(0, 1fr))',
            gap: 14,
          }}
        >
          <Input
            label="Chunk Size"
            type="number"
            value={chunkSize}
            onChange={(e) => setChunkSize(parseInt(e.target.value) || 2048)}
            min={100}
            max={10000}
          />
          <Input
            label="Chunk Overlap"
            type="number"
            value={chunkOverlap}
            onChange={(e) => setChunkOverlap(parseInt(e.target.value) || 256)}
            min={0}
            max={1000}
          />
        </div>

        {/* Transmutation Option */}
        {selectedFile && isTransmutationSupported(selectedFile.name) && (
          <label
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 8,
              cursor: 'pointer',
              fontSize: 13,
              color: 'var(--text-1)',
            }}
          >
            <input
              type="checkbox"
              id="use-transmutation"
              checked={useTransmutation}
              onChange={(e) => setUseTransmutation(e.target.checked)}
              style={{ accentColor: 'var(--teal)' }}
            />
            Use Transmutation (convert{' '}
            {selectedFile.name.split('.').pop()?.toUpperCase()} to Markdown)
          </label>
        )}

        {/* Upload Progress */}
        {state.uploading && (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            <div
              style={{
                display: 'flex',
                justifyContent: 'space-between',
                fontSize: 12,
                color: 'var(--text-2)',
              }}
            >
              <span>Uploading...</span>
              <span style={{ fontVariantNumeric: 'tabular-nums' }}>
                {state.progress}%
              </span>
            </div>
            <div
              style={{
                width: '100%',
                background: 'var(--bg-3)',
                borderRadius: 999,
                height: 6,
                overflow: 'hidden',
              }}
            >
              <div
                style={{
                  background: 'var(--teal)',
                  height: '100%',
                  borderRadius: 999,
                  width: `${state.progress}%`,
                  transition: 'width 200ms ease',
                }}
              />
            </div>
          </div>
        )}

        {/* Success Message */}
        {state.response && state.response.success && (
          <div
            style={{
              ...ALERT_BASE,
              background: 'rgba(76,195,138,0.10)',
              borderColor: 'rgba(76,195,138,0.35)',
            }}
          >
            <div style={{ color: 'var(--green)', flexShrink: 0, marginTop: 1 }}>
              <Icons.check size={16} />
            </div>
            <div style={{ flex: 1 }}>
              <div
                style={{ fontSize: 13, fontWeight: 500, color: 'var(--green)' }}
              >
                Upload successful!
              </div>
              <div
                style={{
                  fontSize: 11,
                  color: 'var(--green)',
                  opacity: 0.85,
                  marginTop: 2,
                }}
              >
                {state.response.chunks_created} chunks created,{' '}
                {state.response.vectors_created} vectors indexed
              </div>
            </div>
          </div>
        )}

        {/* Authentication Warning */}
        {authRequired && !isAuthenticated && (
          <div
            style={{
              ...ALERT_BASE,
              background: 'var(--amber-dim)',
              borderColor: 'rgba(240,168,58,0.35)',
            }}
          >
            <div style={{ color: 'var(--amber)', flexShrink: 0, marginTop: 1 }}>
              <Icons.bell size={16} />
            </div>
            <div style={{ flex: 1 }}>
              <div
                style={{ fontSize: 13, fontWeight: 500, color: 'var(--amber)' }}
              >
                Authentication Required
              </div>
              <div
                style={{
                  fontSize: 11,
                  color: 'var(--amber)',
                  opacity: 0.85,
                  marginTop: 2,
                }}
              >
                Please log in or provide an API key to upload files.
              </div>
            </div>
          </div>
        )}

        {/* Error Message */}
        {state.error && (
          <div
            style={{
              ...ALERT_BASE,
              background: 'rgba(229,72,77,0.10)',
              borderColor: 'rgba(229,72,77,0.35)',
            }}
          >
            <div style={{ color: 'var(--red)', flexShrink: 0, marginTop: 1 }}>
              <Icons.x size={16} />
            </div>
            <div style={{ flex: 1 }}>
              <div style={{ fontSize: 13, fontWeight: 500, color: 'var(--red)' }}>
                Upload failed
              </div>
              <div
                style={{
                  fontSize: 11,
                  color: 'var(--red)',
                  opacity: 0.85,
                  marginTop: 2,
                }}
              >
                {state.error}
              </div>
              {state.error.includes('Authentication required') && (
                <div
                  style={{
                    fontSize: 11,
                    color: 'var(--red)',
                    opacity: 0.85,
                    marginTop: 4,
                  }}
                >
                  Please log in or provide an API key to upload files.
                </div>
              )}
            </div>
          </div>
        )}

        {/* Actions */}
        <div
          style={{
            display: 'flex',
            justifyContent: 'flex-end',
            gap: 8,
            paddingTop: 16,
            borderTop: '1px solid var(--border)',
          }}
        >
          <button
            type="button"
            className="btn"
            onClick={handleClose}
            disabled={state.uploading}
          >
            Cancel
          </button>
          <button
            type="button"
            className="btn primary"
            onClick={handleUpload}
            disabled={!canUpload}
          >
            {state.uploading ? 'Uploading...' : 'Upload'}
          </button>
        </div>
      </div>
    </Modal>
  );
}

export default FileUploadModal;
