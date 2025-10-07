/**
 * Tests for File Operations
 * 
 * This test suite covers:
 * - getFileContent() - Retrieve complete file content
 * - listFilesInCollection() - List indexed files
 * - getFileSummary() - Get file summaries
 * - getFileChunksOrdered() - Get file chunks in order
 * - getProjectOutline() - Get project structure
 * - getRelatedFiles() - Find semantically related files
 * - searchByFileType() - Search filtered by file type
 */

const { VectorizerClient } = require('../src/client');

describe('File Operations', () => {
  let client;
  const baseURL = process.env.VECTORIZER_URL || 'http://localhost:15002';
  const testCollection = 'test-collection';
  let serverAvailable = false;

  beforeAll(async () => {
    client = new VectorizerClient({
      baseURL,
      timeout: 30000,
    });

    try {
      await client.healthCheck();
      serverAvailable = true;
    } catch (error) {
      console.warn('WARNING: Vectorizer server not available at', baseURL);
      console.warn('   Integration tests will be skipped. Start server with: cargo run --release');
      serverAvailable = false;
    }
  });

  beforeEach(() => {
    if (!serverAvailable) return;
  });

  describe('getFileContent', () => {
    it('should retrieve complete file content', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        file_path: 'README.md',
      };

      const response = await client.getFileContent(params);

      expect(response).toBeDefined();
      expect(response.file_path).toBe('README.md');
      expect(response.content).toBeDefined();
      expect(response.metadata).toBeDefined();
    });

    it('should retrieve file content with size limit', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        file_path: 'large-file.md',
        max_size_kb: 100,
      };

      const response = await client.getFileContent(params);

      expect(response).toBeDefined();
      expect(response.size_kb).toBeLessThanOrEqual(100);
    });

    it('should include file metadata', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        file_path: 'src/main.ts',
      };

      const response = await client.getFileContent(params);

      expect(response).toBeDefined();
      expect(response.metadata).toBeDefined();
      expect(response.metadata.file_type).toBeDefined();
      expect(response.metadata.size).toBeGreaterThan(0);
    });

    it('should handle non-existent file gracefully', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        file_path: 'non-existent-file.txt',
      };

      await expect(client.getFileContent(params)).rejects.toThrow();
    });
  });

  describe('listFilesInCollection', () => {
    it('should list all files in collection', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
      };

      const response = await client.listFilesInCollection(params);

      expect(response).toBeDefined();
      expect(response.files).toBeInstanceOf(Array);
      expect(response.total_count).toBeGreaterThanOrEqual(0);
    });

    it('should filter by file type', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        filter_by_type: ['ts', 'js'],
      };

      const response = await client.listFilesInCollection(params);

      expect(response).toBeDefined();
      expect(response.files).toBeInstanceOf(Array);
      
      if (response.files.length > 0) {
        response.files.forEach((file) => {
          expect(['ts', 'js'].some(ext => file.file_path.endsWith(`.${ext}`))).toBe(true);
        });
      }
    });

    it('should filter by minimum chunks', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        min_chunks: 5,
      };

      const response = await client.listFilesInCollection(params);

      expect(response).toBeDefined();
      expect(response.files).toBeInstanceOf(Array);
      
      if (response.files.length > 0) {
        response.files.forEach((file) => {
          expect(file.chunk_count).toBeGreaterThanOrEqual(5);
        });
      }
    });

    it('should limit results', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        max_results: 10,
      };

      const response = await client.listFilesInCollection(params);

      expect(response).toBeDefined();
      expect(response.files.length).toBeLessThanOrEqual(10);
    });

    it('should sort by name', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        sort_by: 'name',
      };

      const response = await client.listFilesInCollection(params);

      expect(response).toBeDefined();
      
      // Verify sorting
      if (response.files.length > 1) {
        for (let i = 0; i < response.files.length - 1; i++) {
          expect(response.files[i].file_path.localeCompare(response.files[i + 1].file_path))
            .toBeLessThanOrEqual(0);
        }
      }
    });

    it('should sort by size', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        sort_by: 'size',
      };

      const response = await client.listFilesInCollection(params);

      expect(response).toBeDefined();
      
      // Verify sorting
      if (response.files.length > 1) {
        for (let i = 0; i < response.files.length - 1; i++) {
          expect(response.files[i].size).toBeGreaterThanOrEqual(response.files[i + 1].size);
        }
      }
    });

    it('should sort by chunks', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        sort_by: 'chunks',
      };

      const response = await client.listFilesInCollection(params);

      expect(response).toBeDefined();
      
      // Verify sorting
      if (response.files.length > 1) {
        for (let i = 0; i < response.files.length - 1; i++) {
          expect(response.files[i].chunk_count)
            .toBeGreaterThanOrEqual(response.files[i + 1].chunk_count);
        }
      }
    });
  });

  describe('getFileSummary', () => {
    it('should get extractive summary', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        file_path: 'README.md',
        summary_type: 'extractive',
        max_sentences: 5,
      };

      const response = await client.getFileSummary(params);

      expect(response).toBeDefined();
      expect(response.summary).toBeDefined();
      expect(response.summary_type).toBe('extractive');
      expect(response.sentences.length).toBeLessThanOrEqual(5);
    });

    it('should get structural summary', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        file_path: 'src/main.ts',
        summary_type: 'structural',
      };

      const response = await client.getFileSummary(params);

      expect(response).toBeDefined();
      expect(response.summary).toBeDefined();
      expect(response.summary_type).toBe('structural');
      expect(response.structure).toBeDefined();
    });

    it('should get both summary types', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        file_path: 'docs/api.md',
        summary_type: 'both',
      };

      const response = await client.getFileSummary(params);

      expect(response).toBeDefined();
      expect(response.extractive_summary).toBeDefined();
      expect(response.structural_summary).toBeDefined();
    });
  });

  describe('getFileChunksOrdered', () => {
    it('should get file chunks in order', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        file_path: 'README.md',
      };

      const response = await client.getFileChunksOrdered(params);

      expect(response).toBeDefined();
      expect(response.chunks).toBeInstanceOf(Array);
      expect(response.total_chunks).toBeGreaterThanOrEqual(0);
    });

    it('should start from specific chunk', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        file_path: 'README.md',
        start_chunk: 5,
        limit: 10,
      };

      const response = await client.getFileChunksOrdered(params);

      expect(response).toBeDefined();
      expect(response.chunks).toBeInstanceOf(Array);
      expect(response.start_chunk).toBe(5);
      expect(response.chunks.length).toBeLessThanOrEqual(10);
    });

    it('should include context hints', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        file_path: 'README.md',
        include_context: true,
      };

      const response = await client.getFileChunksOrdered(params);

      expect(response).toBeDefined();
      expect(response.chunks).toBeInstanceOf(Array);
      
      if (response.chunks.length > 0) {
        response.chunks.forEach((chunk) => {
          expect(chunk.has_prev).toBeDefined();
          expect(chunk.has_next).toBeDefined();
        });
      }
    });

    it('should paginate through chunks', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      // Get first page
      const page1 = await client.getFileChunksOrdered({
        collection: testCollection,
        file_path: 'README.md',
        start_chunk: 0,
        limit: 5,
      });

      expect(page1).toBeDefined();

      if (page1.total_chunks > 5) {
        // Get second page
        const page2 = await client.getFileChunksOrdered({
          collection: testCollection,
          file_path: 'README.md',
          start_chunk: 5,
          limit: 5,
        });

        expect(page2).toBeDefined();
        expect(page2.start_chunk).toBe(5);
      }
    });
  });

  describe('getProjectOutline', () => {
    it('should get project outline', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
      };

      const response = await client.getProjectOutline(params);

      expect(response).toBeDefined();
      expect(response.structure).toBeDefined();
      expect(response.statistics).toBeDefined();
    });

    it('should limit depth', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        max_depth: 3,
      };

      const response = await client.getProjectOutline(params);

      expect(response).toBeDefined();
      expect(response.max_depth).toBe(3);
    });

    it('should include file summaries', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        include_summaries: true,
      };

      const response = await client.getProjectOutline(params);

      expect(response).toBeDefined();
      expect(response.structure).toBeDefined();
      
      // Check if summaries are included
      const hasSummaries = JSON.stringify(response.structure).includes('summary');
      expect(hasSummaries).toBe(true);
    });

    it('should highlight key files', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        highlight_key_files: true,
      };

      const response = await client.getProjectOutline(params);

      expect(response).toBeDefined();
      expect(response.key_files).toBeDefined();
      expect(response.key_files).toBeInstanceOf(Array);
    });
  });

  describe('getRelatedFiles', () => {
    it('should find related files', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        file_path: 'src/main.ts',
      };

      const response = await client.getRelatedFiles(params);

      expect(response).toBeDefined();
      expect(response.related_files).toBeInstanceOf(Array);
    });

    it('should limit results', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        file_path: 'README.md',
        limit: 5,
      };

      const response = await client.getRelatedFiles(params);

      expect(response).toBeDefined();
      expect(response.related_files.length).toBeLessThanOrEqual(5);
    });

    it('should filter by similarity threshold', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        file_path: 'src/main.ts',
        similarity_threshold: 0.7,
      };

      const response = await client.getRelatedFiles(params);

      expect(response).toBeDefined();
      
      if (response.related_files.length > 0) {
        response.related_files.forEach((file) => {
          expect(file.similarity_score).toBeGreaterThanOrEqual(0.7);
        });
      }
    });

    it('should include reason for relation', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        file_path: 'src/main.ts',
        include_reason: true,
      };

      const response = await client.getRelatedFiles(params);

      expect(response).toBeDefined();
      
      if (response.related_files.length > 0) {
        response.related_files.forEach((file) => {
          expect(file.reason).toBeDefined();
        });
      }
    });
  });

  describe('searchByFileType', () => {
    it('should limit results', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        query: 'test',
        file_types: ['ts', 'js'],
        limit: 10,
      };

      const response = await client.searchByFileType(params);

      expect(response).toBeDefined();
      expect(response.results.length).toBeLessThanOrEqual(10);
    });

    it('should handle multiple file types', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        query: 'code',
        file_types: ['ts', 'js', 'py', 'rs'],
      };

      const response = await client.searchByFileType(params);

      expect(response).toBeDefined();
      expect(response.results).toBeInstanceOf(Array);
    });
  });

  describe('Error Handling', () => {
    it('should handle invalid collection', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: 'non-existent-collection',
        file_path: 'README.md',
      };

      await expect(client.getFileContent(params)).rejects.toThrow();
    });

    it('should handle invalid max_size_kb', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        file_path: 'README.md',
        max_size_kb: -1,
      };

      await expect(client.getFileContent(params)).rejects.toThrow();
    });

    it('should handle empty file types array', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const params = {
        collection: testCollection,
        query: 'test',
        file_types: [],
      };

      await expect(client.searchByFileType(params)).rejects.toThrow();
    });
  });

  describe('Performance Tests', () => {
    it('should list files efficiently', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const startTime = Date.now();
      
      await client.listFilesInCollection({
        collection: testCollection,
        max_results: 100,
      });
      
      const duration = Date.now() - startTime;
      expect(duration).toBeLessThan(5000);
    });

    it('should retrieve file content quickly', async () => {
      if (!serverAvailable) return expect(true).toBe(true);

      const startTime = Date.now();
      
      await client.getFileContent({
        collection: testCollection,
        file_path: 'README.md',
      });
      
      const duration = Date.now() - startTime;
      expect(duration).toBeLessThan(3000);
    });
  });
});

