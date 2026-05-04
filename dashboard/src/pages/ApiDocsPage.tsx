/**
 * API Docs page — console-themed restyle.
 *
 * Visual restyle only: behaviour (the static endpoint catalog, the live
 * "Try it" sandbox with API-key auth, the request history / favorites
 * persistence, and the multi-language code-sample tabs) is preserved
 * from the pre-redesign version. The redesign brief has no dedicated
 * mockup for API Docs, so this page applies the established Phase 3
 * recipe:
 *   - `.page` + `.page-head` shell with title/sub
 *   - 2-column layout: sidebar `Card` listing categories + main `Card`
 *     stack of endpoints, each with a method `Pill` (GET=teal,
 *     POST=teal, PUT=amber, DELETE=magenta, PATCH=muted)
 *   - console `Card` / `CardHead` / `CardBody`
 *   - `.btn` actions with `Icons.*`, `.input` / `.mono` for fields
 *   - no Tailwind utility classes, no `dark:` variants
 *   - drop `@untitledui/icons` and `@/components/ui/Card`/`Button`
 *
 * The sandbox itself is rendered as an inline panel below the catalog
 * (not as a modal) — flagged with `// TODO(api-docs-sandbox)` until the
 * console design ships a modal primitive (matches the modal-deferral
 * pattern from BackupsPage / FileWatcher / Users).
 *
 * The Monaco editor is retained on the request-body / response panels.
 */

import { useState, useMemo, useEffect } from 'react';
import { Link } from 'react-router-dom';
import {
  Icons,
  Pill,
  type PillTone,
  Card,
  CardHead,
  CardBody,
} from '@/components/console';
import CodeEditor from '@/components/ui/CodeEditor';
import {
  useSandboxHistory,
  type SandboxHistoryApi,
  type SandboxRequestRecord,
} from '@/hooks/useSandboxHistory';

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

// API Endpoints catalog — preserved verbatim from the legacy page.
const API_ENDPOINTS: ApiEndpoint[] = [
  // Health & Status
  {
    method: 'GET',
    path: '/health',
    description: 'Check server health status',
    category: 'Health',
    responseExample: { status: 'healthy', version: '2.4.1' },
  },
  {
    method: 'GET',
    path: '/setup/status',
    description: 'Check if initial setup is needed',
    category: 'Setup',
    responseExample: {
      needs_setup: false,
      version: '2.4.1',
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
        { name: 'my-collection', vector_count: 1500, dimension: 384 },
      ],
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
          { id: 'vec-1', vector: [0.1, 0.2, 0.3], payload: { text: 'Hello' } },
        ],
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
        { id: 'vec-1', score: 0.95, payload: { text: 'Relevant result' } },
      ],
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
        { collection: 'docs', id: 'auth-1', score: 0.92, content: '...' },
      ],
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
      results: [{ id: 'doc-1', score: 0.88, content: '...' }],
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
        { title: 'Overview', bullets: ['...'] },
      ],
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
        projects: [
          {
            name: 'my-project',
            path: '/path/to/project',
            collections: [],
          },
        ],
      },
    },
    responseExample: { success: true, workspace_file: 'workspace.yml' },
  },
  {
    method: 'GET',
    path: '/setup/templates',
    description: 'Get available configuration templates',
    category: 'Setup',
    responseExample: [{ id: 'rag', name: 'RAG', description: '...' }],
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
      description:
        'Multipart form with file and metadata. Use form-data with: file (binary), collection_name (string), chunk_size (optional), chunk_overlap (optional), metadata (optional JSON)',
      example: {
        collection_name: 'my-docs',
        chunk_size: 500,
        chunk_overlap: 100,
        metadata: { source: 'upload', author: 'user' },
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
      processing_time_ms: 1234,
    },
  },
  {
    method: 'GET',
    path: '/files/config',
    description:
      'Get file upload configuration (allowed extensions, size limits)',
    category: 'File Upload',
    responseExample: {
      max_file_size: 52428800,
      allowed_extensions: [
        'txt', 'md', 'pdf', 'docx', 'html', 'json', 'yaml',
        'rs', 'py', 'js', 'ts',
      ],
      default_chunk_size: 500,
      default_chunk_overlap: 100,
      reject_binary: true,
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
        max_size_kb: 500,
      },
    },
    responseExample: {
      file_path: 'src/main.rs',
      content: 'fn main() {\n    println!("Hello!");\n}',
      metadata: {
        size_kb: 2,
        chunk_count: 1,
        language: 'rust',
      },
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
        sort_by: 'name',
      },
    },
    responseExample: {
      collection: 'codebase',
      files: [{ path: 'src/main.rs', chunk_count: 5, file_type: 'rs' }],
      total: 1,
    },
  },
  {
    method: 'POST',
    path: '/file/summary',
    description: "Get a summary of a file's content",
    category: 'File Operations',
    requestBody: {
      description: 'File summary request',
      example: {
        collection: 'codebase',
        file_path: 'src/main.rs',
        summary_type: 'extractive',
        max_length: 500,
      },
    },
    responseExample: {
      file_path: 'src/main.rs',
      summary: 'Main entry point that initializes the server...',
      key_points: ['Server initialization', 'Route setup'],
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
        include_files: true,
      },
    },
    responseExample: {
      collection: 'codebase',
      outline: { src: { type: 'directory', files: ['main.rs'] } },
      total_files: 1,
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
        similarity_threshold: 0.7,
      },
    },
    responseExample: {
      file_path: 'src/main.rs',
      related_files: [
        { path: 'src/lib.rs', similarity: 0.85, reason: 'Shared imports' },
      ],
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
        limit: 20,
      },
    },
    responseExample: {
      results: [{ file_path: 'Cargo.toml', score: 0.92, matches: [] }],
      total: 1,
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
        {
          name: 'my-project',
          path: '/home/user/projects/my-project',
          is_directory: true,
          is_project: true,
        },
      ],
      valid: true,
    },
  },
];

// Method → Pill tone. GET/POST stay on the brand teal accent; DELETE
// gets magenta (destructive); PUT amber (mutating); PATCH muted.
function methodTone(method: ApiEndpoint['method']): PillTone {
  switch (method) {
    case 'GET':
    case 'POST':
      return 'teal';
    case 'DELETE':
      return 'magenta';
    case 'PUT':
      return 'amber';
    case 'PATCH':
      return 'muted';
    default:
      return 'default';
  }
}

function ApiDocsPage() {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null);
  const [expandedEndpoint, setExpandedEndpoint] = useState<string | null>(null);
  const [sandboxEndpoint, setSandboxEndpoint] = useState<ApiEndpoint | null>(
    null,
  );
  const sandboxHistory = useSandboxHistory();

  const categories = useMemo(
    () => [...new Set(API_ENDPOINTS.map((e) => e.category))],
    [],
  );

  const filteredEndpoints = useMemo(() => {
    return API_ENDPOINTS.filter((endpoint) => {
      const matchesSearch =
        !searchQuery ||
        endpoint.path.toLowerCase().includes(searchQuery.toLowerCase()) ||
        endpoint.description
          .toLowerCase()
          .includes(searchQuery.toLowerCase());
      const matchesCategory =
        !selectedCategory || endpoint.category === selectedCategory;
      return matchesSearch && matchesCategory;
    });
  }, [searchQuery, selectedCategory]);

  const groupedEndpoints = useMemo(() => {
    const groups: Record<string, ApiEndpoint[]> = {};
    filteredEndpoints.forEach((endpoint) => {
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
    <div className="page">
      <div className="page-head">
        <div>
          <h1 className="page-title">API Documentation</h1>
          <p className="page-sub">
            REST + MCP reference · {API_ENDPOINTS.length} endpoints across{' '}
            {categories.length} categories
          </p>
        </div>
      </div>

      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'minmax(200px, 240px) 1fr',
          gap: 14,
          alignItems: 'start',
        }}
      >
        {/* Sidebar — search + category filter */}
        <Card>
          <CardBody>
            <div
              style={{
                display: 'flex',
                flexDirection: 'column',
                gap: 12,
              }}
            >
              <label
                style={{
                  display: 'flex',
                  flexDirection: 'column',
                  gap: 6,
                }}
              >
                <span style={{ color: 'var(--text-2)', fontSize: 12 }}>
                  Search endpoints
                </span>
                <input
                  className="input"
                  type="text"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  placeholder="path or description"
                />
              </label>

              <div
                style={{
                  display: 'flex',
                  flexDirection: 'column',
                  gap: 4,
                }}
              >
                <button
                  className={`btn sm${selectedCategory === null ? ' primary' : ''}`}
                  onClick={() => setSelectedCategory(null)}
                  style={{ justifyContent: 'space-between', width: '100%' }}
                >
                  <span>All endpoints</span>
                  <Pill tone="muted" className="mono">
                    {API_ENDPOINTS.length}
                  </Pill>
                </button>
                {categories.map((category) => {
                  const count = API_ENDPOINTS.filter(
                    (e) => e.category === category,
                  ).length;
                  const active = selectedCategory === category;
                  return (
                    <button
                      key={category}
                      className={`btn sm${active ? ' primary' : ''}`}
                      onClick={() => setSelectedCategory(category)}
                      style={{
                        justifyContent: 'space-between',
                        width: '100%',
                      }}
                    >
                      <span>{category}</span>
                      <Pill tone="muted" className="mono">
                        {count}
                      </Pill>
                    </button>
                  );
                })}
              </div>
            </div>
          </CardBody>
        </Card>

        {/* Main column — endpoints grouped by category */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 14 }}>
          {Object.entries(groupedEndpoints).map(([category, endpoints]) => (
            <Card key={category}>
              <CardHead
                title={category}
                sub={`${endpoints.length} endpoint${endpoints.length === 1 ? '' : 's'}`}
              />
              <CardBody tight>
                <div
                  style={{
                    display: 'flex',
                    flexDirection: 'column',
                  }}
                >
                  {endpoints.map((endpoint, idx) => {
                    const key = `${endpoint.method}-${endpoint.path}-${idx}`;
                    const isExpanded = expandedEndpoint === key;
                    return (
                      <div
                        key={key}
                        style={{
                          borderTop: idx === 0 ? 'none' : '1px solid var(--line)',
                        }}
                      >
                        <button
                          onClick={() => toggleEndpoint(key)}
                          aria-expanded={isExpanded}
                          style={{
                            display: 'flex',
                            alignItems: 'center',
                            gap: 10,
                            width: '100%',
                            padding: '10px 12px',
                            background: 'transparent',
                            border: 'none',
                            cursor: 'pointer',
                            textAlign: 'left',
                            color: 'var(--text-1)',
                          }}
                        >
                          <Pill tone={methodTone(endpoint.method)} className="mono">
                            {endpoint.method}
                          </Pill>
                          <code
                            className="mono"
                            style={{
                              color: 'var(--text-1)',
                              fontSize: 13,
                            }}
                          >
                            {endpoint.path}
                          </code>
                          <span
                            style={{
                              color: 'var(--text-2)',
                              fontSize: 12,
                              flex: 1,
                              overflow: 'hidden',
                              textOverflow: 'ellipsis',
                              whiteSpace: 'nowrap',
                            }}
                          >
                            {endpoint.description}
                          </span>
                          <Icons.chevron
                            size={12}
                            className="muted"
                            style={{
                              transform: isExpanded
                                ? 'rotate(90deg)'
                                : 'rotate(0deg)',
                              transition: 'transform 120ms ease',
                            }}
                          />
                        </button>
                        {isExpanded && (
                          <div
                            style={{
                              padding: '0 12px 14px',
                              display: 'flex',
                              flexDirection: 'column',
                              gap: 12,
                            }}
                          >
                            <p
                              style={{
                                color: 'var(--text-2)',
                                fontSize: 13,
                                margin: 0,
                              }}
                            >
                              {endpoint.description}
                            </p>

                            {endpoint.pathParams &&
                              endpoint.pathParams.length > 0 && (
                                <div>
                                  <h4
                                    style={{
                                      color: 'var(--text-1)',
                                      fontSize: 12,
                                      fontWeight: 600,
                                      margin: '0 0 6px 0',
                                    }}
                                  >
                                    Path parameters
                                  </h4>
                                  <div
                                    style={{
                                      display: 'flex',
                                      flexDirection: 'column',
                                      gap: 4,
                                    }}
                                  >
                                    {endpoint.pathParams.map((param) => (
                                      <div
                                        key={param.name}
                                        className="row"
                                        style={{
                                          gap: 8,
                                          fontSize: 12,
                                          alignItems: 'baseline',
                                        }}
                                      >
                                        <Pill tone="teal" className="mono">
                                          {param.name}
                                        </Pill>
                                        <span
                                          className="mono"
                                          style={{ color: 'var(--text-3)' }}
                                        >
                                          {param.type}
                                        </span>
                                        <span style={{ color: 'var(--text-2)' }}>
                                          — {param.description}
                                        </span>
                                      </div>
                                    ))}
                                  </div>
                                </div>
                              )}

                            {endpoint.requestBody && (
                              <div>
                                <h4
                                  style={{
                                    color: 'var(--text-1)',
                                    fontSize: 12,
                                    fontWeight: 600,
                                    margin: '0 0 6px 0',
                                  }}
                                >
                                  Request body
                                </h4>
                                <p
                                  style={{
                                    color: 'var(--text-2)',
                                    fontSize: 12,
                                    margin: '0 0 6px 0',
                                  }}
                                >
                                  {endpoint.requestBody.description}
                                </p>
                                <CodeSample
                                  code={JSON.stringify(
                                    endpoint.requestBody.example,
                                    null,
                                    2,
                                  )}
                                />
                              </div>
                            )}

                            {endpoint.responseExample && (
                              <div>
                                <h4
                                  style={{
                                    color: 'var(--text-1)',
                                    fontSize: 12,
                                    fontWeight: 600,
                                    margin: '0 0 6px 0',
                                  }}
                                >
                                  Response example
                                </h4>
                                <CodeSample
                                  code={JSON.stringify(
                                    endpoint.responseExample,
                                    null,
                                    2,
                                  )}
                                />
                              </div>
                            )}

                            <div>
                              <button
                                className="btn primary sm"
                                onClick={() => setSandboxEndpoint(endpoint)}
                              >
                                <Icons.zap size={11} />
                                Try it in sandbox
                              </button>
                            </div>
                          </div>
                        )}
                      </div>
                    );
                  })}
                </div>
              </CardBody>
            </Card>
          ))}

          {filteredEndpoints.length === 0 && (
            <Card>
              <CardBody>
                <div
                  style={{
                    padding: 24,
                    color: 'var(--text-2)',
                    textAlign: 'center',
                  }}
                >
                  No endpoints found matching your search.
                </div>
              </CardBody>
            </Card>
          )}

          {sandboxEndpoint && (
            <SandboxPanel
              endpoint={sandboxEndpoint}
              onClose={() => setSandboxEndpoint(null)}
              sandboxHistory={sandboxHistory}
            />
          )}
        </div>
      </div>
    </div>
  );
}

// Static JSON code sample with copy button — used in the read-only
// "Request body" and "Response example" sections of each endpoint card.
function CodeSample({ code }: { code: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div style={{ position: 'relative' }}>
      <pre
        className="mono"
        style={{
          margin: 0,
          padding: '10px 12px',
          background: 'var(--bg-3)',
          border: '1px solid var(--line)',
          borderRadius: 6,
          color: 'var(--text-1)',
          fontSize: 12,
          lineHeight: 1.5,
          overflowX: 'auto',
        }}
      >
        {code}
      </pre>
      <button
        className="btn sm"
        onClick={handleCopy}
        style={{
          position: 'absolute',
          top: 6,
          right: 6,
        }}
        aria-label="Copy code"
      >
        {copied ? <Icons.check size={11} /> : <Icons.copy size={11} />}
        {copied ? 'Copied' : 'Copy'}
      </button>
    </div>
  );
}

// Sandbox panel — preserves the full "Try it" workflow from the legacy
// modal: API-key auth, path params, request body editor (Monaco), live
// fetch with timing, multi-language code samples, history + favorites.
//
// TODO(api-docs-sandbox): rendered as an inline panel below the catalog
// instead of a true modal — the console design has not yet shipped a
// modal primitive. Matches the deferral pattern used by Users / Backups
// / FileWatcher.
function SandboxPanel({
  endpoint,
  onClose,
  sandboxHistory,
}: {
  endpoint: ApiEndpoint;
  onClose: () => void;
  sandboxHistory: SandboxHistoryApi;
}) {
  const [requestBody, setRequestBody] = useState(
    endpoint.requestBody
      ? JSON.stringify(endpoint.requestBody.example, null, 2)
      : '',
  );
  const [pathParams, setPathParams] = useState<Record<string, string>>({});
  const [response, setResponse] = useState<{
    status: number;
    body: string;
    time: number;
  } | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<
    'response' | 'curl' | 'typescript' | 'python' | 'rust' | 'go'
  >('response');
  const [apiKey, setApiKey] = useState<string>('');
  const [hasApiKeys, setHasApiKeys] = useState<boolean | null>(null);
  const [useApiKey, setUseApiKey] = useState(false);
  const [showHistoryPanel, setShowHistoryPanel] = useState(false);

  // Reset all sandbox state when the user opens a different endpoint.
  useEffect(() => {
    setRequestBody(
      endpoint.requestBody
        ? JSON.stringify(endpoint.requestBody.example, null, 2)
        : '',
    );
    setPathParams({});
    setResponse(null);
    setError(null);
    setActiveTab('response');
    setShowHistoryPanel(false);
  }, [endpoint]);

  const scopedFavorites = useMemo(
    () =>
      sandboxHistory.favorites.filter(
        (f) => f.method === endpoint.method && f.path === endpoint.path,
      ),
    [sandboxHistory.favorites, endpoint.method, endpoint.path],
  );
  const scopedHistory = useMemo(
    () =>
      sandboxHistory.history.filter(
        (h) => h.method === endpoint.method && h.path === endpoint.path,
      ),
    [sandboxHistory.history, endpoint.method, endpoint.path],
  );

  const applyRecord = (record: SandboxRequestRecord) => {
    setPathParams(record.pathParams);
    setRequestBody(record.body);
    setShowHistoryPanel(false);
  };

  // Probe `/api-keys` so the auth section can warn the user when no
  // keys exist. Behaviour preserved from the legacy modal.
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

  const buildUrl = () => {
    let url = endpoint.path;
    if (endpoint.pathParams) {
      endpoint.pathParams.forEach((param) => {
        url = url.replace(
          `{${param.name}}`,
          pathParams[param.name] || `:${param.name}`,
        );
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
        body,
        time,
      });

      sandboxHistory.recordRequest({
        method: endpoint.method,
        path: endpoint.path,
        pathParams: { ...pathParams },
        body: requestBody,
        status: res.status,
        timingMs: time,
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Request failed');
    } finally {
      setLoading(false);
    }
  };

  const handleToggleFavorite = () => {
    sandboxHistory.toggleFavorite({
      method: endpoint.method,
      path: endpoint.path,
      pathParams: { ...pathParams },
      body: requestBody,
    });
  };

  const isCurrentFavorited = sandboxHistory.isFavorited(
    endpoint.method,
    endpoint.path,
    requestBody,
  );

  // Code sample generators — preserved from the legacy modal verbatim.
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

  const codeForTab: Record<typeof activeTab, () => string> = {
    response: () => '',
    curl: generateCurl,
    typescript: generateTypeScript,
    python: generatePython,
    rust: generateRust,
    go: generateGo,
  };

  const tabLanguage: Record<typeof activeTab, string> = {
    response: 'json',
    curl: 'shell',
    typescript: 'typescript',
    python: 'python',
    rust: 'rust',
    go: 'go',
  };

  return (
    <Card>
      <CardHead
        title={
          <span className="row" style={{ gap: 8, alignItems: 'center' }}>
            <Icons.zap size={13} />
            <span>Sandbox</span>
            <Pill tone={methodTone(endpoint.method)} className="mono">
              {endpoint.method}
            </Pill>
            <code className="mono" style={{ color: 'var(--text-1)' }}>
              {endpoint.path}
            </code>
          </span>
        }
        right={
          <div className="row" style={{ gap: 4 }}>
            <button
              className={`btn sm${showHistoryPanel ? ' primary' : ''}`}
              onClick={() => setShowHistoryPanel((v) => !v)}
              aria-label="Toggle request history"
            >
              <Icons.activity size={11} />
              History · {scopedHistory.length + scopedFavorites.length}
            </button>
            <button className="btn sm" onClick={onClose} aria-label="Close sandbox">
              <Icons.x size={11} />
              Close
            </button>
          </div>
        }
      />
      <CardBody>
        {showHistoryPanel &&
          (scopedFavorites.length > 0 || scopedHistory.length > 0) && (
            <div
              style={{
                marginBottom: 12,
                padding: 10,
                background: 'var(--bg-3)',
                border: '1px solid var(--line)',
                borderRadius: 6,
                display: 'flex',
                flexDirection: 'column',
                gap: 10,
                maxHeight: 220,
                overflow: 'auto',
              }}
            >
              {scopedFavorites.length > 0 && (
                <div>
                  <div
                    className="row"
                    style={{
                      justifyContent: 'space-between',
                      marginBottom: 6,
                    }}
                  >
                    <span
                      style={{
                        color: 'var(--text-2)',
                        fontSize: 11,
                        fontWeight: 600,
                      }}
                    >
                      Favorites · {scopedFavorites.length}
                    </span>
                  </div>
                  <ul
                    style={{
                      listStyle: 'none',
                      padding: 0,
                      margin: 0,
                      display: 'flex',
                      flexDirection: 'column',
                      gap: 4,
                    }}
                  >
                    {scopedFavorites.map((fav) => (
                      <li
                        key={fav.id}
                        className="row"
                        style={{
                          gap: 6,
                          padding: '4px 6px',
                          background: 'var(--bg-2)',
                          border: '1px solid var(--line)',
                          borderRadius: 4,
                          fontSize: 11,
                        }}
                      >
                        <Pill tone="amber" className="mono">
                          fav
                        </Pill>
                        <button
                          onClick={() => applyRecord(fav)}
                          className="mono"
                          style={{
                            flex: 1,
                            textAlign: 'left',
                            background: 'transparent',
                            border: 'none',
                            cursor: 'pointer',
                            color: 'var(--text-1)',
                            overflow: 'hidden',
                            textOverflow: 'ellipsis',
                            whiteSpace: 'nowrap',
                          }}
                          title="Load into sandbox"
                        >
                          {fav.body.trim().slice(0, 80) || '(empty body)'}
                        </button>
                        <button
                          className="btn sm"
                          onClick={() => sandboxHistory.removeFavorite(fav.id)}
                          aria-label="Remove favorite"
                        >
                          <Icons.trash size={10} />
                        </button>
                      </li>
                    ))}
                  </ul>
                </div>
              )}
              {scopedHistory.length > 0 && (
                <div>
                  <div
                    className="row"
                    style={{
                      justifyContent: 'space-between',
                      marginBottom: 6,
                    }}
                  >
                    <span
                      style={{
                        color: 'var(--text-2)',
                        fontSize: 11,
                        fontWeight: 600,
                      }}
                    >
                      Recent · {scopedHistory.length}
                    </span>
                    <button
                      className="btn sm"
                      onClick={sandboxHistory.clearHistory}
                      aria-label="Clear all history"
                    >
                      <Icons.trash size={10} />
                      Clear all
                    </button>
                  </div>
                  <ul
                    style={{
                      listStyle: 'none',
                      padding: 0,
                      margin: 0,
                      display: 'flex',
                      flexDirection: 'column',
                      gap: 4,
                    }}
                  >
                    {scopedHistory.map((h) => (
                      <li
                        key={h.id}
                        className="row"
                        style={{
                          gap: 6,
                          padding: '4px 6px',
                          background: 'var(--bg-2)',
                          border: '1px solid var(--line)',
                          borderRadius: 4,
                          fontSize: 11,
                        }}
                      >
                        {typeof h.status === 'number' && (
                          <Pill
                            tone={
                              h.status >= 200 && h.status < 300
                                ? 'green'
                                : 'red'
                            }
                            className="mono"
                          >
                            {h.status}
                          </Pill>
                        )}
                        {typeof h.timingMs === 'number' && (
                          <span
                            className="mono"
                            style={{ color: 'var(--text-3)' }}
                          >
                            {h.timingMs}ms
                          </span>
                        )}
                        <button
                          onClick={() => applyRecord(h)}
                          className="mono"
                          style={{
                            flex: 1,
                            textAlign: 'left',
                            background: 'transparent',
                            border: 'none',
                            cursor: 'pointer',
                            color: 'var(--text-1)',
                            overflow: 'hidden',
                            textOverflow: 'ellipsis',
                            whiteSpace: 'nowrap',
                          }}
                        >
                          {h.body.trim().slice(0, 80) || '(empty body)'}
                        </button>
                        <button
                          className="btn sm"
                          onClick={() => sandboxHistory.removeHistory(h.id)}
                          aria-label="Remove from history"
                        >
                          <Icons.trash size={10} />
                        </button>
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          )}

        <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
          {/* Authentication */}
          <div
            style={{
              padding: 10,
              background: 'var(--bg-3)',
              border: '1px solid var(--line)',
              borderRadius: 6,
            }}
          >
            <div
              className="row"
              style={{ justifyContent: 'space-between', marginBottom: 6 }}
            >
              <span
                className="row"
                style={{
                  gap: 6,
                  color: 'var(--text-1)',
                  fontSize: 12,
                  fontWeight: 600,
                }}
              >
                <Icons.keys size={11} className="muted" />
                Authentication
              </span>
              <label
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 6,
                  fontSize: 12,
                  color: 'var(--text-2)',
                  cursor: 'pointer',
                }}
              >
                <input
                  type="checkbox"
                  checked={useApiKey}
                  onChange={(e) => setUseApiKey(e.target.checked)}
                />
                Use API key
              </label>
            </div>
            {useApiKey ? (
              <div
                style={{
                  display: 'flex',
                  flexDirection: 'column',
                  gap: 6,
                }}
              >
                <input
                  className="input mono"
                  type="password"
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  placeholder="Enter your API key"
                />
                {hasApiKeys === false && (
                  <div
                    className="row"
                    style={{ gap: 6, fontSize: 11 }}
                  >
                    <Pill tone="amber">no keys</Pill>
                    <span style={{ color: 'var(--text-2)' }}>
                      No API keys found.
                    </span>
                    <Link
                      to="/api-keys"
                      style={{ color: 'var(--accent)' }}
                    >
                      Create one
                    </Link>
                  </div>
                )}
              </div>
            ) : (
              <p
                style={{
                  color: 'var(--text-2)',
                  fontSize: 11,
                  margin: 0,
                }}
              >
                Enable to send requests with API key authentication
                (required for protected endpoints).
              </p>
            )}
          </div>

          {/* Path parameters */}
          {endpoint.pathParams && endpoint.pathParams.length > 0 && (
            <div>
              <h4
                style={{
                  color: 'var(--text-1)',
                  fontSize: 12,
                  fontWeight: 600,
                  margin: '0 0 6px 0',
                }}
              >
                Path parameters
              </h4>
              <div
                style={{
                  display: 'grid',
                  gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))',
                  gap: 8,
                }}
              >
                {endpoint.pathParams.map((param) => (
                  <label
                    key={param.name}
                    style={{
                      display: 'flex',
                      flexDirection: 'column',
                      gap: 4,
                    }}
                  >
                    <span
                      style={{
                        color: 'var(--text-2)',
                        fontSize: 11,
                      }}
                    >
                      {param.name}{' '}
                      <span style={{ color: 'var(--text-3)' }}>
                        ({param.type})
                      </span>
                    </span>
                    <input
                      className="input mono"
                      type="text"
                      value={pathParams[param.name] || ''}
                      onChange={(e) =>
                        setPathParams({
                          ...pathParams,
                          [param.name]: e.target.value,
                        })
                      }
                      placeholder={param.description}
                    />
                  </label>
                ))}
              </div>
            </div>
          )}

          {/* Request body — Monaco editor preserved */}
          {endpoint.method !== 'GET' && (
            <div>
              <h4
                style={{
                  color: 'var(--text-1)',
                  fontSize: 12,
                  fontWeight: 600,
                  margin: '0 0 6px 0',
                }}
              >
                Request body
              </h4>
              <CodeEditor
                value={requestBody}
                onChange={(next) => setRequestBody(next ?? '')}
                language="json"
                height="240px"
              />
            </div>
          )}

          {/* Execute + favorite */}
          <div className="row" style={{ gap: 8 }}>
            <button
              className="btn primary"
              onClick={executeRequest}
              disabled={loading}
            >
              {loading ? (
                <>
                  <Icons.refresh size={11} />
                  Sending…
                </>
              ) : (
                <>
                  <Icons.zap size={11} />
                  Send request
                </>
              )}
            </button>
            <button
              type="button"
              className={`btn sm${isCurrentFavorited ? ' primary' : ''}`}
              onClick={handleToggleFavorite}
              aria-label={
                isCurrentFavorited
                  ? 'Remove from favorites'
                  : 'Save as favorite'
              }
            >
              <Icons.bolt size={11} />
              {isCurrentFavorited ? 'Favorited' : 'Save'}
            </button>
          </div>

          {error && (
            <div
              className="row"
              style={{
                gap: 8,
                padding: 10,
                background: 'var(--bg-3)',
                border: '1px solid var(--line)',
                borderRadius: 6,
              }}
            >
              <Pill tone="red">error</Pill>
              <span style={{ color: 'var(--text-2)', fontSize: 12 }}>
                {error}
              </span>
            </div>
          )}

          {/* Response / code samples — Monaco editor preserved */}
          <div
            style={{
              border: '1px solid var(--line)',
              borderRadius: 6,
              overflow: 'hidden',
            }}
          >
            <div
              style={{
                display: 'flex',
                gap: 0,
                background: 'var(--bg-3)',
                borderBottom: '1px solid var(--line)',
                overflowX: 'auto',
              }}
            >
              {(
                [
                  'response',
                  'curl',
                  'typescript',
                  'python',
                  'rust',
                  'go',
                ] as const
              ).map((tab) => {
                const active = activeTab === tab;
                const label =
                  tab === 'response'
                    ? 'Response'
                    : tab === 'curl'
                      ? 'cURL'
                      : tab === 'typescript'
                        ? 'TypeScript'
                        : tab === 'python'
                          ? 'Python'
                          : tab === 'rust'
                            ? 'Rust'
                            : 'Go';
                return (
                  <button
                    key={tab}
                    onClick={() => setActiveTab(tab)}
                    style={{
                      padding: '8px 12px',
                      fontSize: 11,
                      background: active ? 'var(--bg-1)' : 'transparent',
                      border: 'none',
                      borderBottom: active
                        ? '2px solid var(--accent)'
                        : '2px solid transparent',
                      color: active ? 'var(--text-1)' : 'var(--text-2)',
                      cursor: 'pointer',
                      whiteSpace: 'nowrap',
                      fontWeight: active ? 600 : 400,
                    }}
                  >
                    {label}
                  </button>
                );
              })}
            </div>

            <div style={{ padding: 12, background: 'var(--bg-1)' }}>
              {activeTab === 'response' && response && (
                <div>
                  <div className="row" style={{ gap: 8, marginBottom: 8 }}>
                    <Pill
                      tone={
                        response.status >= 200 && response.status < 300
                          ? 'green'
                          : 'red'
                      }
                      className="mono"
                    >
                      {response.status >= 200 && response.status < 300
                        ? 'OK'
                        : 'ERR'}{' '}
                      · {response.status}
                    </Pill>
                    <span
                      className="mono"
                      style={{ color: 'var(--text-3)', fontSize: 11 }}
                    >
                      {response.time}ms
                    </span>
                  </div>
                  <CodeEditor
                    value={(() => {
                      try {
                        return JSON.stringify(
                          JSON.parse(response.body),
                          null,
                          2,
                        );
                      } catch {
                        return response.body;
                      }
                    })()}
                    language="json"
                    height="240px"
                    readOnly
                  />
                </div>
              )}

              {activeTab === 'response' && !response && (
                <div
                  style={{
                    padding: 24,
                    color: 'var(--text-2)',
                    textAlign: 'center',
                    fontSize: 12,
                  }}
                >
                  Send a request to see the response.
                </div>
              )}

              {activeTab !== 'response' && (
                <CodeSampleWithCopy
                  code={codeForTab[activeTab]()}
                  language={tabLanguage[activeTab]}
                />
              )}
            </div>
          </div>
        </div>
      </CardBody>
    </Card>
  );
}

// Read-only Monaco-backed code sample with copy affordance — used for
// the cURL / TypeScript / Python / Rust / Go sandbox tabs.
function CodeSampleWithCopy({
  code,
  language,
}: {
  code: string;
  language: string;
}) {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div style={{ position: 'relative' }}>
      <CodeEditor
        value={code}
        language={language}
        height="240px"
        readOnly
      />
      <button
        className="btn sm"
        onClick={handleCopy}
        style={{
          position: 'absolute',
          top: 6,
          right: 6,
          zIndex: 1,
        }}
        aria-label="Copy code"
      >
        {copied ? <Icons.check size={11} /> : <Icons.copy size={11} />}
        {copied ? 'Copied' : 'Copy'}
      </button>
    </div>
  );
}

export default ApiDocsPage;
