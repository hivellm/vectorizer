/**
 * File surface: indexed-content navigation + uploads.
 *
 * Splits cleanly into three groups: read-only navigation
 * (`getFileContent`, `listFilesInCollection`, etc.), file-aware search
 * (`getRelatedFiles`, `searchByFileType`), and the multipart upload
 * paths that share the same form-data builder.
 */

import { BaseClient } from './_base';
import { FileUploadConfig, FileUploadResponse, UploadFileOptions } from '../models';

export class FilesClient extends BaseClient {
  /** Retrieve complete file content from a collection. */
  public async getFileContent(params: {
    collection: string;
    file_path: string;
    max_size_kb?: number;
  }): Promise<unknown> {
    this.logger.debug('Getting file content', params);
    return this.transport.post('/file/content', params);
  }

  /** List indexed files in a collection. */
  public async listFilesInCollection(params: {
    collection: string;
    filter_by_type?: string[];
    min_chunks?: number;
    max_results?: number;
    sort_by?: 'name' | 'size' | 'chunks' | 'recent';
  }): Promise<unknown> {
    this.logger.debug('Listing files in collection', params);
    return this.transport.post('/file/list', params);
  }

  /** Get an extractive or structural summary of an indexed file. */
  public async getFileSummary(params: {
    collection: string;
    file_path: string;
    summary_type?: 'extractive' | 'structural' | 'both';
    max_sentences?: number;
  }): Promise<unknown> {
    this.logger.debug('Getting file summary', params);
    return this.transport.post('/file/summary', params);
  }

  /** Retrieve chunks in original file order for progressive reading. */
  public async getFileChunksOrdered(params: {
    collection: string;
    file_path: string;
    start_chunk?: number;
    limit?: number;
    include_context?: boolean;
  }): Promise<unknown> {
    this.logger.debug('Getting file chunks', params);
    return this.transport.post('/file/chunks', params);
  }

  /** Generate a hierarchical project structure overview. */
  public async getProjectOutline(params: {
    collection: string;
    max_depth?: number;
    include_summaries?: boolean;
    highlight_key_files?: boolean;
  }): Promise<unknown> {
    this.logger.debug('Getting project outline', params);
    return this.transport.post('/file/outline', params);
  }

  /** Find semantically related files using vector similarity. */
  public async getRelatedFiles(params: {
    collection: string;
    file_path: string;
    limit?: number;
    similarity_threshold?: number;
    include_reason?: boolean;
  }): Promise<unknown> {
    this.logger.debug('Getting related files', params);
    return this.transport.post('/file/related', params);
  }

  /** Semantic search filtered by file type. */
  public async searchByFileType(params: {
    collection: string;
    query: string;
    file_types: string[];
    limit?: number;
    return_full_files?: boolean;
  }): Promise<unknown> {
    this.logger.debug('Searching by file type', params);
    return this.transport.post('/file/search_by_type', params);
  }

  /**
   * Upload a file for indexing. Accepts a `File` object in the browser
   * or a `Buffer` / `Uint8Array` in Node.js — both arrive at the server
   * as multipart form data.
   */
  public async uploadFile(
    file: File | Buffer | Uint8Array,
    filename: string,
    collectionName: string,
    options: UploadFileOptions = {},
  ): Promise<FileUploadResponse> {
    this.logger.debug('Uploading file', { filename, collectionName, options });

    const formData = new FormData();

    if (typeof File !== 'undefined' && file instanceof File) {
      formData.append('file', file, filename);
    } else if (typeof Blob !== 'undefined') {
      const blob = new Blob([file as Buffer | Uint8Array]);
      formData.append('file', blob, filename);
    } else {
      throw new Error('File upload requires File, Buffer, or Uint8Array');
    }

    formData.append('collection_name', collectionName);

    if (options.chunkSize !== undefined) {
      formData.append('chunk_size', options.chunkSize.toString());
    }
    if (options.chunkOverlap !== undefined) {
      formData.append('chunk_overlap', options.chunkOverlap.toString());
    }
    if (options.metadata !== undefined) {
      formData.append('metadata', JSON.stringify(options.metadata));
    }
    if (options.publicKey !== undefined) {
      formData.append('public_key', options.publicKey);
    }

    const response = await this.transport.postFormData<FileUploadResponse>(
      '/files/upload',
      formData,
    );

    this.logger.info('File uploaded successfully', {
      filename,
      chunksCreated: response.chunks_created,
      vectorsCreated: response.vectors_created,
    });

    return response;
  }

  /** Upload a string payload as if it were a file. */
  public async uploadFileContent(
    content: string,
    filename: string,
    collectionName: string,
    options: UploadFileOptions = {},
  ): Promise<FileUploadResponse> {
    this.logger.debug('Uploading file content', { filename, collectionName, options });
    const buffer = new TextEncoder().encode(content);
    return this.uploadFile(buffer, filename, collectionName, options);
  }

  /** Get file upload limits and allowed extensions. */
  public async getUploadConfig(): Promise<FileUploadConfig> {
    this.logger.debug('Getting file upload configuration');
    return this.transport.get('/files/config');
  }
}
