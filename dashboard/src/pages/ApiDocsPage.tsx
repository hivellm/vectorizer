/**
 * API Documentation Page
 * Internal documentation with interactive sandbox
 */

import { useState, useMemo, useEffect } from 'react';
import { Link } from 'react-router-dom';
import Card from '@/components/ui/Card';
import Button from '@/components/ui/Button';
import { SearchMd, ChevronRight, ChevronDown, Copy01, Check, Play, XClose, Key01, AlertCircle, Send01, Clock, Terminal, Code01 } from '@untitledui/icons';

// API Endpoint definition
interface ApiEndpoint {
  method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  path: string;
  description: string;
  category: string;
  requestBody?: {
    description: string;
    example: object;
  };
  responseExample?: object;
  queryParams?: Array<{
    name: string;
    type: string;
    required: boolean;
    description: string;
  }>;
  pathParams?: Array<{
    name: string;
    type: string;
    description: string;
  }>;
}

// API Endpoints catalog
const API_ENDPOINTS: ApiEndpoint[] = [
  // Health & Status
  {
    method: 'GET',
    path: '/health',
    description: 'Check server health status',
    category: 'Health',
    responseExample: { status: 'healthy', version: '2.2.0' },
  },
  {
    method: 'GET',
    path: '/setup/status',
    description: 'Check if initial setup is needed',
    category: 'Setup',
    responseExample: {
      needs_setup: false,
      version: '2.2.0',
      deployment_type: 'binary',
      has_workspace_config: true,
      project_count: 1,
      collection_count: 3,
    },
  },
  // Collections
  {
    method: 'GET',
    path: '/collections',
    description: 'List all collections',
    category: 'Collections',
    responseExample: {
      collections: [
        { name: 'my-collection', vector_count: 1500, dimension: 384 }
      ]
    },
  },
  {
    method: 'POST',
    path: '/collections',
    description: 'Create a new collection',
    category: 'Collections',
    requestBody: {
      description: 'Collection configuration',
      example: {
        name: 'my-collection',
        dimension: 384,
        metric: 'cosine',
      },
    },
    responseExample: { success: true, message: 'Collection created' },
  },
  {
    method: 'GET',
    path: '/collections/{name}',
    description: 'Get collection details',
    category: 'Collections',
    pathParams: [{ name: 'name', type: 'string', description: 'Collection name' }],
    responseExample: {
      name: 'my-collection',
      vector_count: 1500,
      dimension: 384,
      metric: 'cosine',
    },
  },
  {
    method: 'DELETE',
    path: '/collections/{name}',
    description: 'Delete a collection',
    category: 'Collections',
    pathParams: [{ name: 'name', type: 'string', description: 'Collection name' }],
    responseExample: { success: true },
  },
  // Vectors
  {
    method: 'POST',
    path: '/collections/{name}/vectors',
    description: 'Insert vectors into a collection',
    category: 'Vectors',
    pathParams: [{ name: 'name', type: 'string', description: 'Collection name' }],
    requestBody: {
      description: 'Vectors to insert',
      example: {
        vectors: [
          { id: 'vec-1', vector: [0.1, 0.2, 0.3], payload: { text: 'Hello' } }
        ]
      },
    },
    responseExample: { inserted: 1 },
  },
  {
    method: 'GET',
    path: '/collections/{name}/vectors/{id}',
    description: 'Get a specific vector by ID',
    category: 'Vectors',
    pathParams: [
      { name: 'name', type: 'string', description: 'Collection name' },
      { name: 'id', type: 'string', description: 'Vector ID' },
    ],
    responseExample: {
      id: 'vec-1',
      vector: [0.1, 0.2, 0.3],
      payload: { text: 'Hello' },
    },
  },
  {
    method: 'DELETE',
    path: '/collections/{name}/vectors/{id}',
    description: 'Delete a vector by ID',
    category: 'Vectors',
    pathParams: [
      { name: 'name', type: 'string', description: 'Collection name' },
      { name: 'id', type: 'string', description: 'Vector ID' },
    ],
    responseExample: { success: true },
  },
  // Search
  {
    method: 'POST',
    path: '/collections/{name}/search',
    description: 'Search for similar vectors',
    category: 'Search',
    pathParams: [{ name: 'name', type: 'string', description: 'Collection name' }],
    requestBody: {
      description: 'Search query',
      example: {
        query: 'semantic search example',
        limit: 10,
      },
    },
    responseExample: {
      results: [
        { id: 'vec-1', score: 0.95, payload: { text: 'Relevant result' } }
      ]
    },
  },
  {
    method: 'POST',
    path: '/search/intelligent',
    description: 'Intelligent search with AI-powered query expansion',
    category: 'Search',
    requestBody: {
      description: 'Intelligent search configuration',
      example: {
        query: 'how to implement authentication',
        collections: ['docs', 'code'],
        max_results: 10,
        mmr_enabled: true,
        domain_expansion: true,
      },
    },
    responseExample: {
      results: [
        { collection: 'docs', id: 'auth-1', score: 0.92, content: '...' }
      ]
    },
  },
  {
    method: 'POST',
    path: '/search/semantic',
    description: 'Semantic search with reranking',
    category: 'Search',
    requestBody: {
      description: 'Semantic search configuration',
      example: {
        query: 'vector database optimization',
        collection: 'knowledge-base',
        max_results: 15,
        semantic_reranking: true,
      },
    },
    responseExample: {
      results: [{ id: 'doc-1', score: 0.88, content: '...' }]
    },
  },
  {
    method: 'POST',
    path: '/discover',
    description: 'Full discovery pipeline with structured output',
    category: 'Discovery',
    requestBody: {
      description: 'Discovery configuration',
      example: {
        query: 'How does the indexing system work?',
        max_bullets: 20,
        broad_k: 50,
        focus_k: 15,
      },
    },
    responseExample: {
      sections: [
        { title: 'Overview', bullets: ['...'] }
      ]
    },
  },
  // Setup
  {
    method: 'POST',
    path: '/setup/analyze',
    description: 'Analyze a directory for project setup',
    category: 'Setup',
    requestBody: {
      description: 'Directory path to analyze',
      example: { path: '/path/to/project' },
    },
    responseExample: {
      project_name: 'my-project',
      project_types: ['Rust', 'TypeScript'],
      languages: { rust: 45, typescript: 30 },
      suggested_collections: [],
    },
  },
  {
    method: 'POST',
    path: '/setup/apply',
    description: 'Apply setup configuration',
    category: 'Setup',
    requestBody: {
      description: 'Configuration to apply',
      example: {
        projects: [{
          name: 'my-project',
          path: '/path/to/project',
          collections: []
        }]
      },
    },
    responseExample: { success: true, workspace_file: 'workspace.yml' },
  },
  {
    method: 'GET',
    path: '/setup/templates',
    description: 'Get available configuration templates',
    category: 'Setup',
    responseExample: [
      { id: 'rag', name: 'RAG', description: '...' }
    ],
  },
  // Workspace
  {
    method: 'GET',
    path: '/workspace/config',
    description: 'Get current workspace configuration',
    category: 'Workspace',
    responseExample: {
      global_settings: {},
      projects: [],
    },
  },
  {
    method: 'POST',
    path: '/workspace/config',
    description: 'Update workspace configuration',
    category: 'Workspace',
    requestBody: {
      description: 'New workspace configuration',
      example: { projects: [] },
    },
    responseExample: { success: true },
  },
  // File Upload
  {
    method: 'POST',
    path: '/files/upload',
    description: 'Upload a file for indexing (multipart/form-data)',
    category: 'File Upload',
    requestBody: {
      description: 'Multipart form with file and metadata. Use form-data with: file (binary), collection_name (string), chunk_size (optional), chunk_overlap (optional), metadata (optional JSON)',
      example: {
        collection_name: 'my-docs',
        chunk_size: 500,
        chunk_overlap: 100,
        metadata: { source: 'upload', author: 'user' }
      },
    },
    responseExample: {
      success: true,
      filename: 'document.pdf',
      collection_name: 'my-docs',
      chunks_created: 15,
      vectors_created: 15,
      file_size: 102400,
      language: 'text',
      processing_time_ms: 1234
    },
  },
  {
    method: 'GET',
    path: '/files/config',
    description: 'Get file upload configuration (allowed extensions, size limits)',
    category: 'File Upload',
    responseExample: {
      max_file_size: 52428800,
      allowed_extensions: ['txt', 'md', 'pdf', 'docx', 'html', 'json', 'yaml', 'rs', 'py', 'js', 'ts'],
      default_chunk_size: 500,
      default_chunk_overlap: 100,
      reject_binary: true
    },
  },
  // File Operations
  {
    method: 'POST',
    path: '/file/content',
    description: 'Get complete file content from an indexed collection',
    category: 'File Operations',
    requestBody: {
      description: 'File content request',
      example: {
        collection: 'codebase',
        file_path: 'src/main.rs',
        max_size_kb: 500
      },
    },
    responseExample: {
      file_path: 'src/main.rs',
      content: 'fn main() {\n    println!("Hello!");\n}',
      metadata: {
        size_kb: 2,
        chunk_count: 1,
        language: 'rust'
      }
    },
  },
  {
    method: 'POST',
    path: '/file/list',
    description: 'List all indexed files in a collection',
    category: 'File Operations',
    requestBody: {
      description: 'File list request',
      example: {
        collection: 'codebase',
        filter_by_type: ['rs', 'md'],
        max_results: 100,
        sort_by: 'name'
      },
    },
    responseExample: {
      collection: 'codebase',
      files: [
        { path: 'src/main.rs', chunk_count: 5, file_type: 'rs' }
      ],
      total: 1
    },
  },
  {
    method: 'POST',
    path: '/file/summary',
    description: 'Get a summary of a file\'s content',
    category: 'File Operations',
    requestBody: {
      description: 'File summary request',
      example: {
        collection: 'codebase',
        file_path: 'src/main.rs',
        summary_type: 'extractive',
        max_length: 500
      },
    },
    responseExample: {
      file_path: 'src/main.rs',
      summary: 'Main entry point that initializes the server...',
      key_points: ['Server initialization', 'Route setup']
    },
  },
  {
    method: 'POST',
    path: '/file/outline',
    description: 'Get project structure outline from indexed files',
    category: 'File Operations',
    requestBody: {
      description: 'Project outline request',
      example: {
        collection: 'codebase',
        max_depth: 3,
        include_files: true
      },
    },
    responseExample: {
      collection: 'codebase',
      outline: { src: { type: 'directory', files: ['main.rs'] } },
      total_files: 1
    },
  },
  {
    method: 'POST',
    path: '/file/related',
    description: 'Find files semantically related to a given file',
    category: 'File Operations',
    requestBody: {
      description: 'Related files request',
      example: {
        collection: 'codebase',
        file_path: 'src/main.rs',
        max_results: 10,
        similarity_threshold: 0.7
      },
    },
    responseExample: {
      file_path: 'src/main.rs',
      related_files: [
        { path: 'src/lib.rs', similarity: 0.85, reason: 'Shared imports' }
      ]
    },
  },
  {
    method: 'POST',
    path: '/file/search_by_type',
    description: 'Search for files of specific types',
    category: 'File Operations',
    requestBody: {
      description: 'Type-based search request',
      example: {
        collection: 'codebase',
        file_types: ['rs', 'toml'],
        query: 'configuration',
        limit: 20
      },
    },
    responseExample: {
      results: [
        { file_path: 'Cargo.toml', score: 0.92, matches: [] }
      ],
      total: 1
    },
  },
  // Directory Browse (for Setup Wizard)
  {
    method: 'POST',
    path: '/setup/browse',
    description: 'Browse directories for file picker in setup wizard',
    category: 'Setup',
    requestBody: {
      description: 'Directory path to browse (empty for home directory)',
      example: { path: '/home/user/projects' },
    },
    responseExample: {
      current_path: '/home/user/projects',
      parent_path: '/home/user',
      entries: [
        { name: 'my-project', path: '/home/user/projects/my-project', is_directory: true, is_project: true }
      ],
      valid: true
    },
  },
];

const METHOD_COLORS: Record<string, string> = {
  GET: 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400',
  POST: 'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400',
  PUT: 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400',
  DELETE: 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400',
  PATCH: 'bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-400',
};

function ApiDocsPage() {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null);
  const [expandedEndpoint, setExpandedEndpoint] = useState<string | null>(null);
  const [sandboxEndpoint, setSandboxEndpoint] = useState<ApiEndpoint | null>(null);

  // Get unique categories
  const categories = useMemo(() => {
    return [...new Set(API_ENDPOINTS.map(e => e.category))];
  }, []);

  // Filter endpoints
  const filteredEndpoints = useMemo(() => {
    return API_ENDPOINTS.filter(endpoint => {
      const matchesSearch = !searchQuery || 
        endpoint.path.toLowerCase().includes(searchQuery.toLowerCase()) ||
        endpoint.description.toLowerCase().includes(searchQuery.toLowerCase());
      const matchesCategory = !selectedCategory || endpoint.category === selectedCategory;
      return matchesSearch && matchesCategory;
    });
  }, [searchQuery, selectedCategory]);

  // Group by category
  const groupedEndpoints = useMemo(() => {
    const groups: Record<string, ApiEndpoint[]> = {};
    filteredEndpoints.forEach(endpoint => {
      if (!groups[endpoint.category]) {
        groups[endpoint.category] = [];
      }
      groups[endpoint.category].push(endpoint);
    });
    return groups;
  }, [filteredEndpoints]);

  const toggleEndpoint = (key: string) => {
    setExpandedEndpoint(expandedEndpoint === key ? null : key);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-neutral-900 dark:text-white">
          API Documentation
        </h1>
        <p className="text-neutral-600 dark:text-neutral-400 mt-1">
          Explore and test the Vectorizer REST API
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
        {/* Sidebar */}
        <div className="lg:col-span-1">
          <Card className="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800/50 p-4 sticky top-4">
            {/* Search */}
            <div className="relative mb-4">
              <SearchMd className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-neutral-400" />
              <input
                type="text"
                placeholder="Search endpoints..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full pl-10 pr-4 py-2 text-sm border border-neutral-300 dark:border-neutral-700 rounded-lg 
                         bg-white dark:bg-neutral-800 text-neutral-900 dark:text-white
                         focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
              />
            </div>

            {/* Categories */}
            <div className="space-y-1">
              <button
                onClick={() => setSelectedCategory(null)}
                className={`w-full text-left px-3 py-2 rounded-lg text-sm transition-colors ${
                  selectedCategory === null
                    ? 'bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-400'
                    : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800'
                }`}
              >
                All Endpoints ({API_ENDPOINTS.length})
              </button>
              {categories.map(category => (
                <button
                  key={category}
                  onClick={() => setSelectedCategory(category)}
                  className={`w-full text-left px-3 py-2 rounded-lg text-sm transition-colors ${
                    selectedCategory === category
                      ? 'bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-400'
                      : 'text-neutral-600 dark:text-neutral-400 hover:bg-neutral-100 dark:hover:bg-neutral-800'
                  }`}
                >
                  {category} ({API_ENDPOINTS.filter(e => e.category === category).length})
                </button>
              ))}
            </div>
          </Card>
        </div>

        {/* Main Content */}
        <div className="lg:col-span-3 space-y-4">
          {Object.entries(groupedEndpoints).map(([category, endpoints]) => (
            <div key={category}>
              <h2 className="text-lg font-semibold text-neutral-900 dark:text-white mb-3">
                {category}
              </h2>
              <div className="space-y-2">
                {endpoints.map((endpoint, idx) => {
                  const key = `${endpoint.method}-${endpoint.path}-${idx}`;
                  const isExpanded = expandedEndpoint === key;
                  
                  return (
                    <Card
                      key={key}
                      className="bg-white dark:bg-neutral-900 border border-neutral-200 dark:border-neutral-800/50 overflow-hidden"
                    >
                      {/* Header */}
                      <button
                        onClick={() => toggleEndpoint(key)}
                        className="w-full flex items-center gap-3 p-4 text-left hover:bg-neutral-50 dark:hover:bg-neutral-800/50 transition-colors"
                      >
                        <span className={`px-2 py-1 text-xs font-mono font-bold rounded ${METHOD_COLORS[endpoint.method]}`}>
                          {endpoint.method}
                        </span>
                        <code className="text-sm font-mono text-neutral-700 dark:text-neutral-300 flex-1">
                          {endpoint.path}
                        </code>
                        <span className="text-sm text-neutral-500 dark:text-neutral-400 hidden sm:block">
                          {endpoint.description}
                        </span>
                        {isExpanded ? (
                          <ChevronDown className="w-4 h-4 text-neutral-400" />
                        ) : (
                          <ChevronRight className="w-4 h-4 text-neutral-400" />
                        )}
                      </button>

                      {/* Expanded Details */}
                      {isExpanded && (
                        <div className="border-t border-neutral-200 dark:border-neutral-700 p-4 space-y-4">
                          <p className="text-sm text-neutral-600 dark:text-neutral-400">
                            {endpoint.description}
                          </p>

                          {/* Path Parameters */}
                          {endpoint.pathParams && endpoint.pathParams.length > 0 && (
                            <div>
                              <h4 className="text-sm font-medium text-neutral-900 dark:text-white mb-2">
                                Path Parameters
                              </h4>
                              <div className="bg-neutral-50 dark:bg-neutral-800/50 rounded-lg p-3 space-y-2">
                                {endpoint.pathParams.map(param => (
                                  <div key={param.name} className="flex items-start gap-2 text-sm">
                                    <code className="text-primary-600 dark:text-primary-400">{param.name}</code>
                                    <span className="text-neutral-400">({param.type})</span>
                                    <span className="text-neutral-600 dark:text-neutral-400">- {param.description}</span>
                                  </div>
                                ))}
                              </div>
                            </div>
                          )}

                          {/* Request Body */}
                          {endpoint.requestBody && (
                            <div>
                              <h4 className="text-sm font-medium text-neutral-900 dark:text-white mb-2">
                                Request Body
                              </h4>
                              <p className="text-sm text-neutral-500 dark:text-neutral-400 mb-2">
                                {endpoint.requestBody.description}
                              </p>
                              <CodeBlock code={endpoint.requestBody.example} />
                            </div>
                          )}

                          {/* Response Example */}
                          {endpoint.responseExample && (
                            <div>
                              <h4 className="text-sm font-medium text-neutral-900 dark:text-white mb-2">
                                Response Example
                              </h4>
                              <CodeBlock code={endpoint.responseExample} />
                            </div>
                          )}

                          {/* Try it button */}
                          <Button
                            variant="primary"
                            size="sm"
                            onClick={() => setSandboxEndpoint(endpoint)}
                          >
                            <Play className="w-4 h-4 mr-2" />
                            Try it in Sandbox
                          </Button>
                        </div>
                      )}
                    </Card>
                  );
                })}
              </div>
            </div>
          ))}

          {filteredEndpoints.length === 0 && (
            <div className="text-center py-12 text-neutral-500 dark:text-neutral-400">
              No endpoints found matching your search.
            </div>
          )}
        </div>
      </div>

      {/* Sandbox Modal */}
      {sandboxEndpoint && (
        <SandboxModal
          endpoint={sandboxEndpoint}
          onClose={() => setSandboxEndpoint(null)}
        />
      )}
    </div>
  );
}

// Code Block Component
function CodeBlock({ code }: { code: object }) {
  const [copied, setCopied] = useState(false);
  const json = JSON.stringify(code, null, 2);

  const handleCopy = () => {
    navigator.clipboard.writeText(json);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="relative">
      <pre className="bg-neutral-900 dark:bg-neutral-950 text-neutral-100 rounded-lg p-4 text-sm font-mono overflow-x-auto">
        {json}
      </pre>
      <button
        onClick={handleCopy}
        className="absolute top-2 right-2 p-2 rounded-lg bg-neutral-800 hover:bg-neutral-700 transition-colors"
      >
        {copied ? (
          <Check className="w-4 h-4 text-green-400" />
        ) : (
          <Copy01 className="w-4 h-4 text-neutral-400" />
        )}
      </button>
    </div>
  );
}

// Sandbox Modal Component
function SandboxModal({ endpoint, onClose }: { endpoint: ApiEndpoint; onClose: () => void }) {
  const [requestBody, setRequestBody] = useState(
    endpoint.requestBody ? JSON.stringify(endpoint.requestBody.example, null, 2) : ''
  );
  const [pathParams, setPathParams] = useState<Record<string, string>>({});
  const [response, setResponse] = useState<{ status: number; body: string; time: number } | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'response' | 'curl' | 'typescript' | 'python' | 'rust' | 'go'>('response');
  const [apiKey, setApiKey] = useState<string>('');
  const [hasApiKeys, setHasApiKeys] = useState<boolean | null>(null);
  const [useApiKey, setUseApiKey] = useState(false);

  // Check if API keys exist
  useEffect(() => {
    const checkApiKeys = async () => {
      try {
        const res = await fetch('/api-keys');
        if (res.ok) {
          const data = await res.json();
          setHasApiKeys(data.keys && data.keys.length > 0);
        } else {
          setHasApiKeys(false);
        }
      } catch {
        setHasApiKeys(false);
      }
    };
    checkApiKeys();
  }, []);

  // Build URL with path params
  const buildUrl = () => {
    let url = endpoint.path;
    if (endpoint.pathParams) {
      endpoint.pathParams.forEach(param => {
        url = url.replace(`{${param.name}}`, pathParams[param.name] || `:${param.name}`);
      });
    }
    return url;
  };

  const executeRequest = async () => {
    setLoading(true);
    setError(null);
    setResponse(null);

    const startTime = Date.now();
    
    try {
      const url = buildUrl();
      const headers: Record<string, string> = {
        'Content-Type': 'application/json',
      };

      // Add API key if enabled
      if (useApiKey && apiKey) {
        headers['X-API-Key'] = apiKey;
      }

      const options: RequestInit = {
        method: endpoint.method,
        headers,
      };

      if (endpoint.method !== 'GET' && requestBody) {
        options.body = requestBody;
      }

      const res = await fetch(url, options);
      const body = await res.text();
      const time = Date.now() - startTime;

      setResponse({
        status: res.status,
        body: body,
        time,
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Request failed');
    } finally {
      setLoading(false);
    }
  };

  // Generate code examples
  const generateCurl = () => {
    const url = `http://localhost:15002${buildUrl()}`;
    let cmd = `curl -X ${endpoint.method} '${url}'`;
    if (endpoint.method !== 'GET' && requestBody) {
      cmd += ` \\\n  -H 'Content-Type: application/json' \\\n  -d '${requestBody.replace(/\n/g, '')}'`;
    }
    return cmd;
  };

  const generateTypeScript = () => {
    const url = buildUrl();
    if (endpoint.method === 'GET') {
      return `const response = await fetch('${url}');
const data = await response.json();
console.log(data);`;
    }
    return `const response = await fetch('${url}', {
  method: '${endpoint.method}',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify(${requestBody || '{}'})
});
const data = await response.json();
console.log(data);`;
  };

  const generatePython = () => {
    const url = buildUrl();
    if (endpoint.method === 'GET') {
      return `import requests

response = requests.get('http://localhost:15002${url}')
print(response.json())`;
    }
    return `import requests

response = requests.${endpoint.method.toLowerCase()}(
    'http://localhost:15002${url}',
    json=${requestBody || '{}'}
)
print(response.json())`;
  };

  const generateRust = () => {
    const url = buildUrl();
    if (endpoint.method === 'GET') {
      return `use reqwest;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let response = reqwest::get("http://localhost:15002${url}")
        .await?
        .json::<serde_json::Value>()
        .await?;
    
    println!("{:?}", response);
    Ok(())
}`;
    }
    return `use reqwest;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client
        .${endpoint.method.toLowerCase()}("http://localhost:15002${url}")
        .json(&json!(${requestBody || '{}'}))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    
    println!("{:?}", response);
    Ok(())
}`;
  };

  const generateGo = () => {
    const url = buildUrl();
    if (endpoint.method === 'GET') {
      return `package main

import (
    "encoding/json"
    "fmt"
    "io"
    "net/http"
)

func main() {
    resp, err := http.Get("http://localhost:15002${url}")
    if err != nil {
        panic(err)
    }
    defer resp.Body.Close()

    body, _ := io.ReadAll(resp.Body)
    var result map[string]interface{}
    json.Unmarshal(body, &result)
    fmt.Printf("%+v\\n", result)
}`;
    }
    return `package main

import (
    "bytes"
    "encoding/json"
    "fmt"
    "io"
    "net/http"
)

func main() {
    data := ${requestBody || '{}'}
    jsonData, _ := json.Marshal(data)

    req, _ := http.NewRequest("${endpoint.method}", "http://localhost:15002${url}", bytes.NewBuffer(jsonData))
    req.Header.Set("Content-Type", "application/json")

    client := &http.Client{}
    resp, err := client.Do(req)
    if err != nil {
        panic(err)
    }
    defer resp.Body.Close()

    body, _ := io.ReadAll(resp.Body)
    var result map[string]interface{}
    json.Unmarshal(body, &result)
    fmt.Printf("%+v\\n", result)
}`;
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <div className="bg-white dark:bg-neutral-900 rounded-xl shadow-2xl max-w-4xl w-full max-h-[90vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-neutral-200 dark:border-neutral-700">
          <div className="flex items-center gap-3">
            <Terminal className="w-5 h-5 text-neutral-500 dark:text-neutral-400" />
            <span className={`px-2 py-1 text-xs font-mono font-bold rounded ${METHOD_COLORS[endpoint.method]}`}>
              {endpoint.method}
            </span>
            <code className="text-sm font-mono text-neutral-700 dark:text-neutral-300">
              {endpoint.path}
            </code>
          </div>
          <button
            onClick={onClose}
            className="p-1 rounded-lg text-neutral-500 hover:text-neutral-700 dark:hover:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800 transition-colors"
            title="Close"
          >
            <XClose className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-auto p-4 space-y-4">
          {/* API Key Section */}
          <div className="bg-neutral-50 dark:bg-neutral-800/50 border border-neutral-200 dark:border-neutral-700 rounded-lg p-4">
            <div className="flex items-center justify-between mb-3">
              <div className="flex items-center gap-2">
                <Key01 className="w-4 h-4 text-neutral-500 dark:text-neutral-400" />
                <h4 className="text-sm font-medium text-neutral-900 dark:text-white">
                  Authentication
                </h4>
              </div>
              <label className="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={useApiKey}
                  onChange={(e) => setUseApiKey(e.target.checked)}
                  className="w-4 h-4 text-primary-600 rounded border-neutral-300 dark:border-neutral-600"
                />
                <span className="text-xs text-neutral-600 dark:text-neutral-400">Use API Key</span>
              </label>
            </div>
            
            {useApiKey && (
              <div className="space-y-2">
                <input
                  type="password"
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  placeholder="Enter your API key"
                  className="w-full px-3 py-2 text-sm border border-neutral-300 dark:border-neutral-700 rounded-lg 
                           bg-white dark:bg-neutral-800 text-neutral-900 dark:text-white font-mono"
                />
                {hasApiKeys === false && (
                  <div className="flex items-center gap-2 text-xs">
                    <AlertCircle className="w-3 h-3 text-amber-500" />
                    <span className="text-amber-600 dark:text-amber-400">
                      No API keys found.
                    </span>
                    <Link to="/api-keys" className="text-primary-600 dark:text-primary-400 hover:underline">
                      Create one
                    </Link>
                  </div>
                )}
              </div>
            )}
            
            {!useApiKey && (
              <p className="text-xs text-neutral-500 dark:text-neutral-400">
                Enable to send requests with API key authentication (required for protected endpoints)
              </p>
            )}
          </div>

          {/* Path Parameters */}
          {endpoint.pathParams && endpoint.pathParams.length > 0 && (
            <div>
              <h4 className="text-sm font-medium text-neutral-900 dark:text-white mb-2 flex items-center gap-2">
                <Code01 className="w-4 h-4 text-neutral-400" />
                Path Parameters
              </h4>
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                {endpoint.pathParams.map(param => (
                  <div key={param.name}>
                    <label className="block text-xs text-neutral-500 dark:text-neutral-400 mb-1">
                      {param.name} <span className="text-neutral-400">({param.type})</span>
                    </label>
                    <input
                      type="text"
                      value={pathParams[param.name] || ''}
                      onChange={(e) => setPathParams({ ...pathParams, [param.name]: e.target.value })}
                      placeholder={param.description}
                      className="w-full px-3 py-2 text-sm border border-neutral-300 dark:border-neutral-700 rounded-lg 
                               bg-white dark:bg-neutral-800 text-neutral-900 dark:text-white font-mono"
                    />
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Request Body */}
          {endpoint.method !== 'GET' && (
            <div>
              <h4 className="text-sm font-medium text-neutral-900 dark:text-white mb-2 flex items-center gap-2">
                <Code01 className="w-4 h-4 text-neutral-400" />
                Request Body
              </h4>
              <textarea
                value={requestBody}
                onChange={(e) => setRequestBody(e.target.value)}
                rows={8}
                className="w-full px-3 py-2 text-sm font-mono border border-neutral-300 dark:border-neutral-700 rounded-lg 
                         bg-neutral-50 dark:bg-neutral-800 text-neutral-900 dark:text-white"
              />
            </div>
          )}

          {/* Execute Button */}
          <Button
            variant="primary"
            onClick={executeRequest}
            disabled={loading}
            className="flex items-center gap-2"
          >
            {loading ? (
              <>
                <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                Sending...
              </>
            ) : (
              <>
                <Send01 className="w-4 h-4" />
                Send Request
              </>
            )}
          </Button>

          {/* Error */}
          {error && (
            <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-3 text-sm text-red-700 dark:text-red-300">
              {error}
            </div>
          )}

          {/* Response / Code Examples Tabs */}
          <div className="border border-neutral-200 dark:border-neutral-700 rounded-lg overflow-hidden">
            <div className="flex gap-0 bg-neutral-50 dark:bg-neutral-800/50 border-b border-neutral-200 dark:border-neutral-700 overflow-x-auto">
              {(['response', 'curl', 'typescript', 'python', 'rust', 'go'] as const).map(tab => (
                <button
                  key={tab}
                  onClick={() => setActiveTab(tab)}
                  className={`px-4 py-2.5 text-xs font-medium transition-colors whitespace-nowrap flex items-center gap-2 border-b-2 -mb-[1px] ${
                    activeTab === tab
                      ? 'border-primary-500 text-primary-600 dark:text-primary-400 bg-white dark:bg-neutral-900'
                      : 'border-transparent text-neutral-500 hover:text-neutral-700 dark:hover:text-neutral-300 hover:bg-neutral-100 dark:hover:bg-neutral-800'
                  }`}
                >
                  {tab === 'response' && <Terminal className="w-3.5 h-3.5" />}
                  {tab === 'curl' && <Code01 className="w-3.5 h-3.5" />}
                  {tab === 'typescript' && <Code01 className="w-3.5 h-3.5" />}
                  {tab === 'python' && <Code01 className="w-3.5 h-3.5" />}
                  {tab === 'rust' && <Code01 className="w-3.5 h-3.5" />}
                  {tab === 'go' && <Code01 className="w-3.5 h-3.5" />}
                  {tab === 'response' ? 'Response' : tab === 'curl' ? 'cURL' : tab === 'typescript' ? 'TypeScript' : tab === 'python' ? 'Python' : tab === 'rust' ? 'Rust' : 'Go'}
                </button>
              ))}
            </div>

            <div className="p-4 bg-white dark:bg-neutral-900">
              {activeTab === 'response' && response && (
                <div>
                  <div className="flex items-center gap-3 mb-3">
                    <span className={`px-2 py-1 text-xs font-bold rounded flex items-center gap-1.5 ${
                      response.status >= 200 && response.status < 300
                        ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400'
                        : 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400'
                    }`}>
                      {response.status >= 200 && response.status < 300 ? (
                        <Check className="w-3 h-3" />
                      ) : (
                        <AlertCircle className="w-3 h-3" />
                      )}
                      {response.status}
                    </span>
                    <span className="text-xs text-neutral-500 dark:text-neutral-400 flex items-center gap-1">
                      <Clock className="w-3 h-3" />
                      {response.time}ms
                    </span>
                  </div>
                  <div className="max-h-64 overflow-auto rounded-lg">
                    <SyntaxHighlight 
                      code={(() => {
                        try {
                          return JSON.stringify(JSON.parse(response.body), null, 2);
                        } catch {
                          return response.body;
                        }
                      })()}
                      language="json" 
                    />
                  </div>
                </div>
              )}

              {activeTab === 'response' && !response && (
                <div className="text-center py-12 text-neutral-500 dark:text-neutral-400">
                  <Terminal className="w-8 h-8 mx-auto mb-2 text-neutral-400 dark:text-neutral-500" />
                  <p className="text-sm">Send a request to see the response</p>
                </div>
              )}

              {activeTab === 'curl' && (
                <div className="relative">
                  <SyntaxHighlight code={generateCurl()} language="bash" />
                  <CopyButton text={generateCurl()} />
                </div>
              )}

              {activeTab === 'typescript' && (
                <div className="relative">
                  <SyntaxHighlight code={generateTypeScript()} language="typescript" />
                  <CopyButton text={generateTypeScript()} />
                </div>
              )}

              {activeTab === 'python' && (
                <div className="relative">
                  <SyntaxHighlight code={generatePython()} language="python" />
                  <CopyButton text={generatePython()} />
                </div>
              )}

              {activeTab === 'rust' && (
                <div className="relative">
                  <SyntaxHighlight code={generateRust()} language="rust" />
                  <CopyButton text={generateRust()} />
                </div>
              )}

              {activeTab === 'go' && (
                <div className="relative">
                  <SyntaxHighlight code={generateGo()} language="go" />
                  <CopyButton text={generateGo()} />
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

// Copy Button Component
function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <button
      onClick={handleCopy}
      className="absolute top-2 right-2 p-2 rounded-lg bg-neutral-800 hover:bg-neutral-700 transition-colors"
      title="Copy to clipboard"
    >
      {copied ? (
        <Check className="w-4 h-4 text-green-400" />
      ) : (
        <Copy01 className="w-4 h-4 text-neutral-400" />
      )}
    </button>
  );
}

// Syntax Highlighting Component
function SyntaxHighlight({ code, language }: { code: string; language: 'json' | 'typescript' | 'python' | 'rust' | 'go' | 'bash' }) {
  const highlightCode = (text: string, lang: string): string => {
    let html = text
      // Escape HTML first
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');

    // Keywords by language
    const keywords: Record<string, string[]> = {
      typescript: ['const', 'let', 'var', 'function', 'async', 'await', 'return', 'import', 'from', 'export', 'default', 'if', 'else', 'try', 'catch', 'new', 'typeof', 'interface', 'type'],
      python: ['import', 'from', 'def', 'async', 'await', 'return', 'if', 'else', 'elif', 'try', 'except', 'class', 'print', 'True', 'False', 'None', 'with', 'as', 'for', 'in'],
      rust: ['use', 'fn', 'async', 'await', 'let', 'mut', 'pub', 'struct', 'impl', 'return', 'Ok', 'Err', 'Some', 'None', 'match', 'if', 'else', 'loop', 'for', 'in', 'mod', 'crate', 'self'],
      go: ['package', 'import', 'func', 'var', 'const', 'return', 'if', 'else', 'for', 'range', 'defer', 'go', 'chan', 'map', 'struct', 'type', 'interface', 'nil', 'true', 'false'],
      bash: ['curl', 'echo', 'export', 'cd', 'ls', 'cat', 'grep', 'awk', 'sed'],
      json: [],
    };

    // Highlight strings (double and single quotes)
    html = html.replace(
      /(["'`])(?:(?!\1|\\).|\\.)*\1/g,
      '<span class="text-green-400">$&</span>'
    );

    // Highlight numbers
    html = html.replace(
      /\b(\d+\.?\d*)\b/g,
      '<span class="text-amber-400">$1</span>'
    );

    // Highlight keywords
    const langKeywords = keywords[lang] || [];
    langKeywords.forEach(keyword => {
      const regex = new RegExp(`\\b(${keyword})\\b`, 'g');
      html = html.replace(regex, '<span class="text-purple-400 font-medium">$1</span>');
    });

    // Highlight comments
    if (lang === 'python') {
      html = html.replace(/(#.*)$/gm, '<span class="text-neutral-500 italic">$1</span>');
    } else if (lang === 'rust' || lang === 'typescript' || lang === 'go') {
      html = html.replace(/(\/\/.*)$/gm, '<span class="text-neutral-500 italic">$1</span>');
    } else if (lang === 'bash') {
      html = html.replace(/(#.*)$/gm, '<span class="text-neutral-500 italic">$1</span>');
    }

    // Highlight HTTP methods
    html = html.replace(/\b(GET|POST|PUT|DELETE|PATCH)\b/g, '<span class="text-blue-400 font-bold">$1</span>');

    // Highlight URLs
    html = html.replace(
      /(https?:\/\/[^\s'"]+)/g,
      '<span class="text-cyan-400 underline">$1</span>'
    );

    // Highlight JSON keys (for JSON output)
    if (lang === 'json') {
      html = html.replace(
        /("[\w_]+")(\s*:)/g,
        '<span class="text-blue-300">$1</span>$2'
      );
      // Highlight boolean values
      html = html.replace(/\b(true|false|null)\b/g, '<span class="text-amber-400">$1</span>');
    }

    return html;
  };

  return (
    <pre
      className="bg-neutral-900 dark:bg-neutral-950 rounded-lg p-4 text-sm font-mono overflow-x-auto text-neutral-100"
      dangerouslySetInnerHTML={{ __html: highlightCode(code, language) }}
    />
  );
}

export default ApiDocsPage;
