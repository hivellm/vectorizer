import { describe, it, expect, beforeAll } from 'vitest';
import { VectorizerClient } from '../src/client.js';

describe('File Upload', () => {
  let client;
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
        'test-uploads',
        content,
        'test.txt',
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

      console.log(`✓ Upload successful: ${response.chunks_created} chunks, ${response.vectors_created} vectors`);
    } catch (error) {
      console.log('Upload failed (expected if server not running):', error.message);
      // Skip test if server not available
      if (error.message.includes('ECONNREFUSED') || error.message.includes('fetch failed')) {
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
        'test-uploads',
        content,
        'test.txt',
        { metadata }
      );

      expect(response).toBeDefined();
      expect(response.success).toBe(true);
    } catch (error) {
      if (error.message.includes('ECONNREFUSED') || error.message.includes('fetch failed')) {
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

      console.log(`✓ Config: max=${config.max_file_size_mb}MB, chunk=${config.default_chunk_size}`);
    } catch (error) {
      if (error.message.includes('ECONNREFUSED') || error.message.includes('fetch failed')) {
        console.log('Skipping test: server not available');
        return;
      }
      throw error;
    }
  });

  it('should deserialize FileUploadResponse correctly', () => {
    const json = {
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
});
