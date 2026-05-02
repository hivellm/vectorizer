// Mock data for the Vectorizer GUI

const MOCK_COLLECTIONS = [
  { name: "vectorizer-docs", dim: 384, vectors: 6_511, indexType: "HNSW", quantization: "SQ-8bit", provider: "BM25", size: "412 MB", status: "healthy", queriesPerMin: 1240, p99: 2.4, lastIndex: "2m ago" },
  { name: "hivellm-codebase", dim: 768, vectors: 184_220, indexType: "HNSW", quantization: "SQ-8bit", provider: "BM25", size: "2.1 GB", status: "healthy", queriesPerMin: 880, p99: 3.1, lastIndex: "8m ago" },
  { name: "github-issues", dim: 384, vectors: 47_338, indexType: "HNSW", quantization: "PQ", provider: "TFIDF", size: "298 MB", status: "indexing", queriesPerMin: 312, p99: 4.2, lastIndex: "indexing…" },
  { name: "support-tickets-2026", dim: 384, vectors: 22_104, indexType: "HNSW", quantization: "SQ-8bit", provider: "BM25", size: "184 MB", status: "healthy", queriesPerMin: 92, p99: 1.9, lastIndex: "14m ago" },
  { name: "api-reference-summary", dim: 384, vectors: 1_204, indexType: "HNSW", quantization: "None", provider: "BM25", size: "12 MB", status: "healthy", queriesPerMin: 18, p99: 1.1, lastIndex: "1h ago" },
  { name: "chat-history-q1", dim: 768, vectors: 312_550, indexType: "IVF", quantization: "Binary", provider: "MiniLM", size: "3.4 GB", status: "warning", queriesPerMin: 45, p99: 8.7, lastIndex: "3h ago" },
  { name: "research-papers", dim: 1024, vectors: 8_842, indexType: "HNSW", quantization: "SQ-8bit", provider: "E5", size: "188 MB", status: "healthy", queriesPerMin: 27, p99: 2.8, lastIndex: "12m ago" },
  { name: "internal-wiki", dim: 384, vectors: 5_402, indexType: "HNSW", quantization: "SQ-8bit", provider: "BM25", size: "92 MB", status: "healthy", queriesPerMin: 134, p99: 1.6, lastIndex: "5m ago" },
];

const MOCK_VECTORS = [
  { id: "vec_8h2k3l9d", text: "The capability registry mandates REST and MCP must have identical functionality on the data plane.", coll: "vectorizer-docs", norm: 0.998, dim: 384 },
  { id: "vec_2j8s7d1a", text: "Sequences are global to the WAL file, not per-collection. Recovery validates strict monotonicity.", coll: "vectorizer-docs", norm: 0.991, dim: 384 },
  { id: "vec_x4n9q2k0", text: "AVX-512 downclock on Skylake-X server CPUs trips a ~10% sustained frequency drop.", coll: "vectorizer-docs", norm: 1.000, dim: 384 },
  { id: "vec_p7m3w8r5", text: "Master uses split read/write halves for the live phase so command sender and ACK reader don't serialize.", coll: "vectorizer-docs", norm: 0.987, dim: 384 },
  { id: "vec_l1z6v4b2", text: "QueryCache wraps the serialized JSON response keyed by (collection, query, limit, threshold).", coll: "vectorizer-docs", norm: 0.995, dim: 384 },
  { id: "vec_t9c5h8e3", text: "Batch insert operations include automatic embedding generation for text-based input.", coll: "vectorizer-docs", norm: 0.989, dim: 384 },
  { id: "vec_r2u7f1g6", text: "MMR (Maximal Marginal Relevance) algorithm provides diversification for intelligent search.", coll: "vectorizer-docs", norm: 0.993, dim: 384 },
  { id: "vec_k8b3y4o9", text: "Apple Silicon (M1/M2/M3/M4) does NOT implement SVE — they are NEON-only.", coll: "vectorizer-docs", norm: 0.996, dim: 384 },
];

const MOCK_API_KEYS = [
  { id: "key_01h8z", name: "production-server", masked: "vk_live_••••••••••8a2c", role: "ReadWrite", lastUsed: "12s ago", created: "2026-01-12", calls: 4_812_330 },
  { id: "key_92ks0", name: "ci-cd-pipeline", masked: "vk_live_••••••••••f04b", role: "Admin", lastUsed: "4m ago", created: "2026-02-08", calls: 92_104 },
  { id: "key_44md8", name: "analytics-readonly", masked: "vk_live_••••••••••3d11", role: "ReadOnly", lastUsed: "1h ago", created: "2026-03-22", calls: 1_204_502 },
  { id: "key_71qa3", name: "cursor-mcp-local", masked: "vk_live_••••••••••9c7e", role: "Mcp", lastUsed: "2h ago", created: "2026-04-01", calls: 28_440 },
  { id: "key_18tt9", name: "staging-rotated", masked: "vk_live_••••••••••0a5f", role: "ReadWrite", lastUsed: "3d ago", created: "2025-11-18", calls: 882_001 },
];

const MOCK_MCP_TOOLS = [
  { name: "search_vectors", calls: 184_220, p99: 2.8, status: "ok" },
  { name: "intelligent_search", calls: 92_113, p99: 4.4, status: "ok" },
  { name: "semantic_search", calls: 41_770, p99: 3.1, status: "ok" },
  { name: "contextual_search", calls: 28_104, p99: 5.2, status: "ok" },
  { name: "multi_collection_search", calls: 12_504, p99: 6.8, status: "ok" },
  { name: "list_collections", calls: 8_801, p99: 0.4, status: "ok" },
  { name: "insert_texts", calls: 3_122, p99: 12.1, status: "ok" },
  { name: "batch_insert_texts", calls: 1_204, p99: 38.4, status: "ok" },
  { name: "delete_vectors", calls: 882, p99: 0.8, status: "ok" },
  { name: "get_database_stats", calls: 422, p99: 0.2, status: "ok" },
];

const MOCK_REPLICAS = [
  { id: "replica-eu-west-01", offset: 8_812_004, lag: 0, status: "in-sync", region: "eu-west-1" },
  { id: "replica-us-east-01", offset: 8_811_998, lag: 6, status: "in-sync", region: "us-east-1" },
  { id: "replica-us-east-02", offset: 8_811_840, lag: 164, status: "catching-up", region: "us-east-1" },
  { id: "replica-ap-south-01", offset: 8_811_998, lag: 6, status: "in-sync", region: "ap-south-1" },
];

const MOCK_RECENT_EVENTS = [
  { t: "12:04:38", level: "info", msg: "Collection 'github-issues' reindex started (47338 vectors)" },
  { t: "12:04:21", level: "ok", msg: "WAL checkpoint complete · seq=8811998 · 412 MB freed" },
  { t: "12:03:55", level: "warn", msg: "replica-us-east-02 lag exceeds 100ms (164ms)" },
  { t: "12:03:12", level: "info", msg: "Cache invalidated for collection 'support-tickets-2026'" },
  { t: "12:02:48", level: "ok", msg: "API key 'cursor-mcp-local' authenticated · 28k calls today" },
  { t: "12:01:30", level: "info", msg: "Quantization SQ-8bit applied · saved 1.2 GB" },
  { t: "12:00:11", level: "ok", msg: "Server boot complete · SIMD backend: avx2" },
];

// Sparkline mini-data
const SPARK = (n, base, amp) => Array.from({length: n}, (_, i) => base + Math.sin(i/2) * amp + Math.random() * amp * 0.5);

window.MOCK = {
  collections: MOCK_COLLECTIONS,
  vectors: MOCK_VECTORS,
  apiKeys: MOCK_API_KEYS,
  mcpTools: MOCK_MCP_TOOLS,
  replicas: MOCK_REPLICAS,
  events: MOCK_RECENT_EVENTS,
  spark: SPARK,
};
