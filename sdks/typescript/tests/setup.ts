/**
 * Test setup for TypeScript SDK tests.
 */

import { vi } from 'vitest';

// Global test setup
beforeAll(() => {
  // Set test timeout (Vitest uses testTimeout in config)
});

// Global test teardown
afterAll(() => {
  // Cleanup if needed
});

// Mock fetch for tests
global.fetch = vi.fn().mockImplementation((url: string | URL, options?: RequestInit) => {
  const urlStr = typeof url === 'string' ? url : url.toString();
  const method = options?.method || 'GET';
  
  // Parse request body if present
  let requestBody: any = {};
  try {
    if (options?.body && typeof options.body === 'string') {
      requestBody = JSON.parse(options.body);
    }
  } catch (e) {
    // Ignore parse errors
  }
  
  // Validation helpers
  const hasInvalidParam = (value: any, type: 'empty' | 'negative' | 'outOfRange' | 'nonExistent') => {
    if (type === 'empty') return value === '' || (Array.isArray(value) && value.length === 0);
    if (type === 'negative') return typeof value === 'number' && value < 0;
    if (type === 'outOfRange') return typeof value === 'number' && (value < 0 || value > 1);
    if (type === 'nonExistent') return value?.includes('non-existent') || value?.includes('nonexistent') || value?.includes('invalid');
    return false;
  };

  // Check for validation errors in common parameters
  const validateRequest = () => {
    // Check for empty queries
    if (requestBody.query !== undefined && hasInvalidParam(requestBody.query, 'empty')) {
      return { status: 400, message: 'Query cannot be empty' };
    }
    
    // Check for empty collections
    if (requestBody.collections !== undefined && hasInvalidParam(requestBody.collections, 'empty')) {
      return { status: 400, message: 'Collections array cannot be empty' };
    }
    
    // Check for empty collection name
    if (requestBody.collection !== undefined && hasInvalidParam(requestBody.collection, 'empty')) {
      return { status: 400, message: 'Collection name cannot be empty' };
    }
    
    // Check for negative values
    if (hasInvalidParam(requestBody.max_bullets, 'negative') || 
        hasInvalidParam(requestBody.max_size_kb, 'negative')) {
      return { status: 400, message: 'Parameter cannot be negative' };
    }
    
    // Check for out of range values (0-1)
    if (hasInvalidParam(requestBody.similarity_threshold, 'outOfRange') ||
        hasInvalidParam(requestBody.name_match_weight, 'outOfRange') ||
        hasInvalidParam(requestBody.term_boost_weight, 'outOfRange') ||
        hasInvalidParam(requestBody.signal_boost_weight, 'outOfRange')) {
      return { status: 400, message: 'Parameter must be between 0 and 1' };
    }
    
    // Check for non-existent resources
    if (hasInvalidParam(requestBody.collection, 'nonExistent') ||
        hasInvalidParam(requestBody.file_path, 'nonExistent')) {
      return { status: 404, message: 'Resource not found' };
    }
    
    // Check URL path for invalid/nonexistent markers
    if (urlStr.includes('invalid') || urlStr.includes('nonexistent') || urlStr.includes('non-existent')) {
      return { status: 404, message: 'Resource not found' };
    }
    
    // Check for empty file_types array
    if (requestBody.file_types !== undefined && hasInvalidParam(requestBody.file_types, 'empty')) {
      return { status: 400, message: 'File types array cannot be empty' };
    }
    
    return null;
  };

  const validationError = validateRequest();
  
  if (validationError) {
    return Promise.resolve({
      ok: false,
      status: validationError.status,
      statusText: validationError.status === 400 ? 'Bad Request' : 'Not Found',
      headers: {
        get: (name: string) => {
          if (name.toLowerCase() === 'content-type') {
            return 'application/json';
          }
          return null;
        },
      },
      json: () => Promise.resolve({ error: validationError.message }),
      text: () => Promise.resolve(JSON.stringify({ error: validationError.message })),
    } as Response);
  }
  
  // Determine response based on URL and method
  let responseData: unknown = {
    status: 'healthy',
    version: '1.0.0',
  };

  // File operations
  if (urlStr.includes('/file/content')) {
    responseData = {
      file_path: 'README.md',
      content: '# Test File\nThis is a test file.',
      metadata: { file_type: 'md', size: 100 },
      size_kb: 50,
    };
  } else if (urlStr.includes('/file/list')) {
    const filterByType = requestBody.filter_by_type || [];
    const minChunks = requestBody.min_chunks || 0;
    const sortBy = requestBody.sort_by || 'name';
    
    let files = [
      { path: 'README.md', file_path: 'README.md', size: 100, chunk_count: 5, chunks: 5, file_type: 'md' },
      { path: 'src/main.ts', file_path: 'src/main.ts', size: 200, chunk_count: 10, chunks: 10, file_type: 'ts' },
      { path: 'config.js', file_path: 'config.js', size: 150, chunk_count: 8, chunks: 8, file_type: 'js' },
      { path: 'package.json', file_path: 'package.json', size: 300, chunk_count: 15, chunks: 15, file_type: 'json' },
    ];
    
    // Filter by type if specified
    if (filterByType.length > 0) {
      files = files.filter(f => filterByType.includes(f.file_type));
    }
    
    // Filter by min chunks if specified
    if (minChunks > 0) {
      files = files.filter(f => f.chunk_count >= minChunks);
    }
    
    // Sort files
    if (sortBy === 'size') {
      files.sort((a, b) => b.size - a.size);
    } else if (sortBy === 'chunks') {
      files.sort((a, b) => b.chunk_count - a.chunk_count);
    } else {
      files.sort((a, b) => a.path.localeCompare(b.path));
    }
    
    responseData = {
      files,
      total_count: files.length,
    };
  } else if (urlStr.includes('/file/summary')) {
    const summaryType = requestBody.summary_type || 'extractive';
    responseData = {
      summary: 'This is a summary',
      summary_type: summaryType,
      sentences: ['Sentence 1', 'Sentence 2'],
      structure: { sections: [] },
      extractive_summary: 'Extractive summary',
      structural_summary: { outline: [] },
    };
  } else if (urlStr.includes('/file/chunks')) {
    const startChunk = requestBody.start_chunk || 0;
    const includeContext = requestBody.include_context || false;
    const limit = requestBody.limit || 10;
    
    const chunks = [];
    for (let i = 0; i < limit; i++) {
      const chunk: any = {
        index: startChunk + i,
        content: `Chunk ${startChunk + i}`,
      };
      if (includeContext) {
        chunk.has_prev = startChunk + i > 0;
        chunk.has_next = true;
      }
      chunks.push(chunk);
    }
    
    responseData = {
      chunks,
      total_chunks: 10,
      start_chunk: startChunk,
      has_next: true,
      has_prev: startChunk > 0,
    };
  } else if (urlStr.includes('/file/outline') || urlStr.includes('/project/outline')) {
    const includeSummaries = requestBody.include_summaries || false;
    const structure = includeSummaries 
      ? { 
          directories: [{ name: 'src', files: [{ name: 'main.ts', summary: 'Main file' }] }], 
          files: [] 
        }
      : { directories: [], files: [] };
    
    responseData = {
      structure,
      statistics: { total_files: 10, total_dirs: 5 },
      max_depth: requestBody.max_depth || 5,
      key_files: ['README.md'],
    };
  } else if (urlStr.includes('/file/related')) {
    responseData = {
      related_files: [
        { path: 'file1.ts', similarity: 0.9, similarity_score: 0.9, reason: 'Similar content' },
        { path: 'file2.ts', similarity: 0.85, similarity_score: 0.85, reason: 'Shared imports' },
      ],
    };
  } else if (urlStr.includes('/file/search-by-type') || urlStr.includes('/search/by-file-type')) {
    const returnFull = requestBody.return_full_files || false;
    responseData = {
      results: [
        { 
          file_path: 'config.yaml', 
          score: 0.95, 
          content: returnFull ? 'full yaml content here' : 'yaml preview',
          full_content: returnFull ? 'full yaml content here' : undefined 
        },
        { 
          file_path: 'settings.json', 
          score: 0.85, 
          content: returnFull ? 'full json content here' : 'json preview',
          full_content: returnFull ? 'full json content here' : undefined 
        },
      ],
    };
  }
  // Discovery operations - Order matters! More specific first
  else if (urlStr.includes('/filter_collections') || urlStr.includes('/filter-collections')) {
    responseData = {
      filtered_collections: ['col1', 'col2'],
      total_available: 2,
      excluded_count: 0,
    };
  } else if (urlStr.includes('/score_collections') || urlStr.includes('/score-collections')) {
    responseData = {
      scored_collections: [
        { name: 'col1', score: 0.95, relevance: 0.9 },
        { name: 'col2', score: 0.85, relevance: 0.8 },
      ],
      total_collections: 2,
    };
  } else if (urlStr.includes('/expand_queries') || urlStr.includes('/expand-queries')) {
    const originalQuery = requestBody.query || 'CMMV framework';
    responseData = {
      original_query: originalQuery,
      expanded_queries: [
        `What is ${originalQuery}?`,
        `${originalQuery} features`,
        `${originalQuery} architecture`,
      ],
      query_types: ['definition', 'features', 'architecture'],
    };
  } else if (urlStr.includes('/discover') || urlStr.includes('/discovery/discover')) {
    responseData = {
      prompt: 'Generated LLM prompt',
      evidence: [
        { text: 'Evidence 1', citation: 'doc1.md' },
        { text: 'Evidence 2', citation: 'doc2.md' },
      ],
      metadata: { collections_searched: ['col1', 'col2'] },
    };
  }
  // Search operations - underscore routes
  else if (urlStr.includes('/intelligent_search') || urlStr.includes('/intelligent-search')) {
    const maxResults = requestBody.max_results || 10;
    const results = [];
    for (let i = 0; i < Math.min(maxResults, 100); i++) {
      results.push({ id: `${i + 1}`, score: 0.95 - (i * 0.01), text: `Result ${i + 1}`, metadata: {} });
    }
    responseData = {
      results,
      queries_generated: ['query1', 'query2'],
      total_results: results.length,
    };
  } else if (urlStr.includes('/semantic_search') || urlStr.includes('/semantic-search')) {
    responseData = {
      results: [
        { id: '1', score: 0.9, text: 'Semantic result', metadata: {} },
        { id: '2', score: 0.8, text: 'Another result', metadata: {} },
      ],
      total_results: 2,
    };
  } else if (urlStr.includes('/contextual_search') || urlStr.includes('/contextual-search')) {
    responseData = {
      results: [
        { id: '1', score: 0.85, text: 'Contextual result', metadata: {} },
      ],
      total_results: 1,
      context_filters: requestBody.context_filters || {},
    };
  } else if (urlStr.includes('/multi_collection_search') || urlStr.includes('/multi-collection-search')) {
    const collections = requestBody.collections || ['col1', 'col2'];
    responseData = {
      results: [
        { id: '1', score: 0.9, collection: collections[0] || 'col1', text: 'Result 1', metadata: {} },
        { id: '2', score: 0.85, collection: collections[1] || 'col2', text: 'Result 2', metadata: {} },
      ],
      collections_searched: collections,
      total_results: 2,
      results_per_collection: { [collections[0] || 'col1']: 1, [collections[1] || 'col2']: 1 },
    };
  }
  // Collection operations
  else if (urlStr.includes('/collections') && method === 'GET') {
    responseData = {
      collections: [
        { name: 'col1', dimension: 384, similarity_metric: 'cosine' },
      ],
    };
  } else if (urlStr.includes('/collections') && method === 'POST') {
    responseData = { name: 'test-collection', dimension: 384 };
  }
  // Vector operations
  else if (urlStr.includes('/vectors') && method === 'POST') {
    responseData = { inserted: 2, collection: 'test-collection' };
  } else if (urlStr.includes('/search')) {
    responseData = {
      results: [
        { id: '1', score: 0.95, data: [0.1, 0.2, 0.3] },
      ],
    };
  } else if (urlStr.includes('/embed')) {
    responseData = {
      embeddings: [[0.1, 0.2, 0.3]],
    };
  }
  // Database stats
  else if (urlStr.includes('/stats')) {
    responseData = {
      total_collections: 5,
      total_vectors: 1000,
    };
  }

  return Promise.resolve({
    ok: true,
    status: 200,
    statusText: 'OK',
    headers: {
      get: (name: string) => {
        if (name.toLowerCase() === 'content-type') {
          return 'application/json';
        }
        return null;
      },
    },
    json: () => Promise.resolve(responseData),
    text: () => Promise.resolve(JSON.stringify(responseData)),
  } as Response);
}) as unknown as typeof fetch;

// Mock AbortController
global.AbortController = vi.fn().mockImplementation(() => ({
  abort: vi.fn(),
  signal: {
    aborted: false,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
  },
})) as any;

// Mock setTimeout and clearTimeout
const originalSetTimeout = global.setTimeout;
const mockSetTimeout = vi.fn((callback, delay) => {
  return originalSetTimeout(callback, delay);
});

// Add __promisify__ property for Node.js compatibility
Object.assign(mockSetTimeout, {
  __promisify__: vi.fn(),
});

global.setTimeout = mockSetTimeout as any;

const originalClearTimeout = global.clearTimeout;
global.clearTimeout = vi.fn((id) => {
  return originalClearTimeout(id);
}) as any;
