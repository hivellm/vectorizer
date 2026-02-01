/**
 * Hook for file upload functionality
 * Supports transmutation for PDF and other document formats
 */

import { useState } from 'react';

export interface FileUploadOptions {
  collectionName: string;
  chunkSize?: number;
  chunkOverlap?: number;
  metadata?: Record<string, unknown>;
  useTransmutation?: boolean; // Enable transmutation for supported formats
}

export interface FileUploadResponse {
  success: boolean;
  filename: string;
  collection_name: string;
  chunks_created: number;
  vectors_created: number;
  file_size: number;
  language: string;
  processing_time_ms: number;
}

export interface FileUploadState {
  uploading: boolean;
  progress: number;
  error: string | null;
  response: FileUploadResponse | null;
}

export function useFileUpload() {
  const [state, setState] = useState<FileUploadState>({
    uploading: false,
    progress: 0,
    error: null,
    response: null,
  });

  const uploadFile = async (
    file: File,
    options: FileUploadOptions
  ): Promise<FileUploadResponse> => {
    setState({
      uploading: true,
      progress: 0,
      error: null,
      response: null,
    });

    try {
      // Create FormData
      const formData = new FormData();
      formData.append('file', file);
      formData.append('collection_name', options.collectionName);

      if (options.chunkSize) {
        formData.append('chunk_size', options.chunkSize.toString());
      }

      if (options.chunkOverlap) {
        formData.append('chunk_overlap', options.chunkOverlap.toString());
      }

      if (options.metadata) {
        formData.append('metadata', JSON.stringify(options.metadata));
      }

      // Add transmutation flag if enabled
      if (options.useTransmutation !== undefined) {
        formData.append('use_transmutation', options.useTransmutation.toString());
      }

      // Get API base URL
      const baseUrl = import.meta.env.VITE_API_BASE_URL ||
        (import.meta.env.DEV
          ? 'http://localhost:15002'
          : '');

      const url = `${baseUrl}/files/upload`;

      // Get authentication token from localStorage
      const TOKEN_KEY = 'vectorizer_dashboard_token';
      const API_KEY_KEY = 'vectorizer_api_key'; // Alternative storage for API key
      const token = localStorage.getItem(TOKEN_KEY);
      const apiKey = localStorage.getItem(API_KEY_KEY);

      // Create XMLHttpRequest for progress tracking
      const xhr = new XMLHttpRequest();

      const promise = new Promise<FileUploadResponse>((resolve, reject) => {
        xhr.upload.addEventListener('progress', (e) => {
          if (e.lengthComputable) {
            const progress = Math.round((e.loaded / e.total) * 100);
            setState((prev) => ({ ...prev, progress }));
          }
        });

        xhr.addEventListener('load', () => {
          if (xhr.status >= 200 && xhr.status < 300) {
            try {
              const response: FileUploadResponse = JSON.parse(xhr.responseText);
              setState({
                uploading: false,
                progress: 100,
                error: null,
                response,
              });
              resolve(response);
            } catch (e) {
              const error = e instanceof Error ? e.message : 'Failed to parse response';
              setState({
                uploading: false,
                progress: 0,
                error,
                response: null,
              });
              reject(new Error(error));
            }
          } else {
            let errorMessage = 'Upload failed';
            try {
              const errorData = JSON.parse(xhr.responseText);
              errorMessage = errorData.message || errorData.error || errorMessage;
            } catch {
              errorMessage = xhr.statusText || `HTTP ${xhr.status}`;
            }
            setState({
              uploading: false,
              progress: 0,
              error: errorMessage,
              response: null,
            });
            reject(new Error(errorMessage));
          }
        });

        xhr.addEventListener('error', () => {
          const error = 'Network error during upload';
          setState({
            uploading: false,
            progress: 0,
            error,
            response: null,
          });
          reject(new Error(error));
        });

        xhr.addEventListener('abort', () => {
          const error = 'Upload cancelled';
          setState({
            uploading: false,
            progress: 0,
            error,
            response: null,
          });
          reject(new Error(error));
        });

        xhr.open('POST', url);

        // Add Authorization header - prefer JWT token, fallback to API key
        if (token) {
          xhr.setRequestHeader('Authorization', `Bearer ${token}`);
        } else if (apiKey) {
          // API key can be sent as Bearer token or X-API-Key header
          xhr.setRequestHeader('X-API-Key', apiKey);
        }

        xhr.send(formData);
      });

      return promise;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Upload failed';
      setState({
        uploading: false,
        progress: 0,
        error: errorMessage,
        response: null,
      });
      throw error;
    }
  };

  const reset = () => {
    setState({
      uploading: false,
      progress: 0,
      error: null,
      response: null,
    });
  };

  return {
    uploadFile,
    state,
    reset,
  };
}
