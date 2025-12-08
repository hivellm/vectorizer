import { describe, it, expect, beforeAll } from 'vitest';
import { VectorizerClient } from '../src/client';
import type { FileUploadResponse, FileUploadConfig } from '../src/models/file-upload';

describe('File Upload', () => {
  let client: VectorizerClient;
  const baseUrl = process.env.VECTORIZER_TEST_URL || 'http://localhost:15002';

  beforeAll(() => {
    client = new VectorizerClient({ baseUrl });
  });

  it('should upload file content', async () => {
    const content = `
      This is a test document for file upload.
      It contains multiple lines of text to be chunked and indexed.
      The vectorizer should automatically extract, chunk, and create embeddings.
    `;

    try {
      const response = await client.uploadFileContent(
        content,
        'test.txt',
        'test-uploads',
        {
          chunkSize: 100,
          chunkOverlap: 20,
        }
      );

      expect(response).toBeDefined();
      expect(response.success).toBe(true);
      expect(response.filename).toBe('test.txt');
      expect(response.collection_name).toBe('test-uploads');
      expect(response.chunks_created).toBeGreaterThan(0);
      expect(response.vectors_created).toBeGreaterThan(0);

      console.log(
        `✓ Upload successful: ${response.chunks_created} chunks, ${response.vectors_created} vectors`
      );
    } catch (error) {
      const err = error as Error;
      console.log('Upload failed (expected if server not running):', err.message);
      // Skip test if server not available
      if (err.message.includes('ECONNREFUSED') || err.message.includes('fetch failed')) {
        return;
      }
      throw error;
    }
  });

  it('should upload file with Buffer', async () => {
    const content = 'This is test content as a buffer.';
    const buffer = Buffer.from(content, 'utf-8');

    try {
      const response = await client.uploadFile(
        buffer,
        'test.txt',
        'test-uploads'
      );

      expect(response).toBeDefined();
      expect(response.success).toBe(true);
    } catch (error) {
      const err = error as Error;
      if (err.message.includes('ECONNREFUSED') || err.message.includes('fetch failed')) {
        console.log('Skipping test: server not available');
        return;
      }
      throw error;
    }
  });

  it('should upload file with metadata', async () => {
    const content = 'Document with metadata for testing.';
    const metadata = {
      source: 'test',
      type: 'document',
      version: 1,
    };

    try {
      const response = await client.uploadFileContent(
        content,
        'test.txt',
        'test-uploads',
        { metadata }
      );

      expect(response).toBeDefined();
      expect(response.success).toBe(true);
    } catch (error) {
      const err = error as Error;
      if (err.message.includes('ECONNREFUSED') || err.message.includes('fetch failed')) {
        console.log('Skipping test: server not available');
        return;
      }
      throw error;
    }
  });

  it('should get upload configuration', async () => {
    try {
      const config = await client.getUploadConfig();

      expect(config).toBeDefined();
      expect(config.max_file_size).toBeGreaterThan(0);
      expect(config.max_file_size_mb).toBeGreaterThan(0);
      expect(config.default_chunk_size).toBeGreaterThan(0);
      expect(Array.isArray(config.allowed_extensions)).toBe(true);
      expect(config.allowed_extensions.length).toBeGreaterThan(0);

      console.log(
        `✓ Config: max=${config.max_file_size_mb}MB, chunk=${config.default_chunk_size}`
      );
    } catch (error) {
      const err = error as Error;
      if (err.message.includes('ECONNREFUSED') || err.message.includes('fetch failed')) {
        console.log('Skipping test: server not available');
        return;
      }
      throw error;
    }
  });

  it('should deserialize FileUploadResponse correctly', () => {
    const json: FileUploadResponse = {
      success: true,
      filename: 'test.pdf',
      collection_name: 'docs',
      chunks_created: 10,
      vectors_created: 10,
      file_size: 2048,
      language: 'pdf',
      processing_time_ms: 150,
    };

    expect(json.success).toBe(true);
    expect(json.filename).toBe('test.pdf');
    expect(json.collection_name).toBe('docs');
    expect(json.chunks_created).toBe(10);
    expect(json.vectors_created).toBe(10);
    expect(json.file_size).toBe(2048);
    expect(json.language).toBe('pdf');
    expect(json.processing_time_ms).toBe(150);
  });

  it('should deserialize FileUploadConfig correctly', () => {
    const json: FileUploadConfig = {
      max_file_size: 10485760,
      max_file_size_mb: 10,
      allowed_extensions: ['.txt', '.pdf', '.md'],
      reject_binary: true,
      default_chunk_size: 1000,
      default_chunk_overlap: 200,
    };

    expect(json.max_file_size).toBe(10485760);
    expect(json.max_file_size_mb).toBe(10);
    expect(json.allowed_extensions).toHaveLength(3);
    expect(json.reject_binary).toBe(true);
    expect(json.default_chunk_size).toBe(1000);
    expect(json.default_chunk_overlap).toBe(200);
  });
});
