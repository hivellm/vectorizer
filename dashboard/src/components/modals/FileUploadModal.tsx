/**
 * File Upload Modal Component
 * Supports transmutation for PDF and other document formats
 */

import { useState, useRef, useCallback, useEffect } from 'react';
import { useFileUpload } from '@/hooks/useFileUpload';
import { useCollections } from '@/hooks/useCollections';
import { useAuth } from '@/contexts/AuthContext';
import Modal from '@/components/ui/Modal';
import Button from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Upload01, File04, X, CheckCircle, AlertCircle } from '@untitledui/icons';

interface FileUploadModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess?: () => void;
}

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
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Load collections when modal opens
  const loadCollections = useCallback(async () => {
    try {
      const data = await listCollections();
      const collectionsArray = Array.isArray(data) ? data : [];
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
    return ['pdf', 'docx', 'xlsx', 'pptx', 'html', 'htm', 'xml', 'jpg', 'jpeg', 'png', 'tiff', 'tif', 'bmp', 'gif', 'webp'].includes(ext || '');
  };

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setSelectedFile(file);
      // Auto-enable transmutation for supported formats
      if (isTransmutationSupported(file.name)) {
        setUseTransmutation(true);
      }
    }
  };

  const handleDrop = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    const file = e.dataTransfer.files[0];
    if (file) {
      setSelectedFile(file);
      // Auto-enable transmutation for supported formats
      if (isTransmutationSupported(file.name)) {
        setUseTransmutation(true);
      }
    }
  };

  const handleDragOver = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
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
        useTransmutation: useTransmutation && isTransmutationSupported(selectedFile.name),
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

  const canUpload = selectedFile && collectionName.trim() && !state.uploading && (!authRequired || isAuthenticated);

  return (
    <Modal isOpen={isOpen} onClose={handleClose} title="Upload File">
      <div className="space-y-6">
        {/* File Selection */}
        <div>
          <label className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-2">
            Select File
          </label>
          <div
            className="border-2 border-dashed border-neutral-300 dark:border-neutral-700 rounded-lg p-8 text-center cursor-pointer hover:border-primary-500 dark:hover:border-primary-500 transition-colors"
            onDrop={handleDrop}
            onDragOver={handleDragOver}
            onClick={() => fileInputRef.current?.click()}
          >
            <input
              ref={fileInputRef}
              type="file"
              className="hidden"
              onChange={handleFileSelect}
              accept=".pdf,.docx,.xlsx,.pptx,.html,.htm,.xml,.jpg,.jpeg,.png,.tiff,.tif,.bmp,.gif,.webp,.txt,.md,.rs,.py,.js,.ts,.tsx,.jsx,.go,.java,.c,.cpp,.h,.hpp,.cs,.rb,.php,.swift,.kt,.scala,.r,.jl,.lua,.pl,.sh,.bash,.zsh,.fish,.ps1,.bat,.cmd,.json,.yaml,.yml,.toml,.ini,.cfg,.conf,.csv,.tsv"
            />
            {selectedFile ? (
              <div className="space-y-2">
                <File04 className="w-12 h-12 mx-auto text-primary-500" />
                <div>
                  <p className="font-medium text-neutral-900 dark:text-white">{selectedFile.name}</p>
                  <p className="text-sm text-neutral-500 dark:text-neutral-400">
                    {formatFileSize(selectedFile.size)}
                  </p>
                </div>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={(e) => {
                    e.stopPropagation();
                    setSelectedFile(null);
                    if (fileInputRef.current) {
                      fileInputRef.current.value = '';
                    }
                  }}
                >
                  <X className="w-4 h-4 mr-1" />
                  Remove
                </Button>
              </div>
            ) : (
              <div className="space-y-2">
                <Upload01 className="w-12 h-12 mx-auto text-neutral-400" />
                <div>
                  <p className="text-sm font-medium text-neutral-900 dark:text-white">
                    Click to upload or drag and drop
                  </p>
                  <p className="text-xs text-neutral-500 dark:text-neutral-400 mt-1">
                    PDF, DOCX, XLSX, PPTX, images, code files, and more
                  </p>
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Collection Selection */}
        <div>
          <label className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-2">
            Collection Name
          </label>
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
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-2">
              Chunk Size
            </label>
            <Input
              type="number"
              value={chunkSize}
              onChange={(e) => setChunkSize(parseInt(e.target.value) || 2048)}
              min={100}
              max={10000}
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-2">
              Chunk Overlap
            </label>
            <Input
              type="number"
              value={chunkOverlap}
              onChange={(e) => setChunkOverlap(parseInt(e.target.value) || 256)}
              min={0}
              max={1000}
            />
          </div>
        </div>

        {/* Transmutation Option */}
        {selectedFile && isTransmutationSupported(selectedFile.name) && (
          <div className="flex items-center gap-2">
            <input
              type="checkbox"
              id="use-transmutation"
              checked={useTransmutation}
              onChange={(e) => setUseTransmutation(e.target.checked)}
              className="w-4 h-4 text-primary-600 border-neutral-300 rounded focus:ring-primary-500"
            />
            <label htmlFor="use-transmutation" className="text-sm text-neutral-700 dark:text-neutral-300">
              Use Transmutation (convert {selectedFile.name.split('.').pop()?.toUpperCase()} to Markdown)
            </label>
          </div>
        )}

        {/* Upload Progress */}
        {state.uploading && (
          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span className="text-neutral-600 dark:text-neutral-400">Uploading...</span>
              <span className="text-neutral-600 dark:text-neutral-400">{state.progress}%</span>
            </div>
            <div className="w-full bg-neutral-200 dark:bg-neutral-700 rounded-full h-2">
              <div
                className="bg-primary-600 h-2 rounded-full transition-all"
                style={{ width: `${state.progress}%` }}
              />
            </div>
          </div>
        )}

        {/* Success Message */}
        {state.response && state.response.success && (
          <div className="flex items-center gap-2 p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg">
            <CheckCircle className="w-5 h-5 text-green-600 dark:text-green-400" />
            <div className="flex-1">
              <p className="text-sm font-medium text-green-800 dark:text-green-300">
                Upload successful!
              </p>
              <p className="text-xs text-green-600 dark:text-green-400">
                {state.response.chunks_created} chunks created, {state.response.vectors_created} vectors indexed
              </p>
            </div>
          </div>
        )}

        {/* Authentication Warning */}
        {authRequired && !isAuthenticated && (
          <div className="flex items-center gap-2 p-3 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-lg">
            <AlertCircle className="w-5 h-5 text-yellow-600 dark:text-yellow-400" />
            <div className="flex-1">
              <p className="text-sm font-medium text-yellow-800 dark:text-yellow-300">
                Authentication Required
              </p>
              <p className="text-xs text-yellow-600 dark:text-yellow-400">
                Please log in or provide an API key to upload files.
              </p>
            </div>
          </div>
        )}

        {/* Error Message */}
        {state.error && (
          <div className="flex items-center gap-2 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
            <AlertCircle className="w-5 h-5 text-red-600 dark:text-red-400" />
            <div className="flex-1">
              <p className="text-sm font-medium text-red-800 dark:text-red-300">
                Upload failed
              </p>
              <p className="text-xs text-red-600 dark:text-red-400">{state.error}</p>
              {state.error.includes('Authentication required') && (
                <p className="text-xs text-red-600 dark:text-red-400 mt-1">
                  Please log in or provide an API key to upload files.
                </p>
              )}
            </div>
          </div>
        )}

        {/* Actions */}
        <div className="flex justify-end gap-3 pt-4 border-t border-neutral-200 dark:border-neutral-700">
          <Button variant="secondary" onClick={handleClose} disabled={state.uploading}>
            Cancel
          </Button>
          <Button
            variant="primary"
            onClick={handleUpload}
            disabled={!canUpload}
            isLoading={state.uploading}
          >
            Upload
          </Button>
        </div>
      </div>
    </Modal>
  );
}

export default FileUploadModal;
