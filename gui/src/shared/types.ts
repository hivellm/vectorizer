// Connection Types
export type ConnectionType = 'local' | 'remote';

export type ConnectionStatus = 'online' | 'offline' | 'connecting';

export interface Connection {
  readonly id: string;
  name: string;
  type: ConnectionType;
  host: string;
  port: number;
  auth?: {
    token: string;
  };
  active: boolean;
  status: ConnectionStatus;
}

// Collection Types
export type IndexingStatus = 'pending' | 'processing' | 'indexing' | 'completed' | 'cached' | 'failed';

export type DistanceMetric = 'cosine' | 'euclidean' | 'dot';

export interface Collection {
  readonly name: string;
  readonly dimension: number;
  readonly metric: DistanceMetric;
  vector_count: number;
  embedding_provider: string;
  indexing_status: {
    status: IndexingStatus;
    progress: number;
    last_updated?: string;
  };
  quantization?: {
    enabled: boolean;
    type?: string;
  };
  size?: {
    total: string;
    total_bytes: number;
  };
}

// Vector Types
export interface VectorPayload {
  content?: string;
  file_path?: string;
  file_extension?: string;
  chunk_index?: number;
  chunk_size?: number;
  metadata?: Record<string, unknown>;
  [key: string]: unknown;
}

export interface Vector {
  readonly id: string;
  readonly vector_id: string;
  score?: number;
  payload?: VectorPayload;
  metadata?: Record<string, unknown>;
  vector?: number[];
  embedding?: number[];
}

// Search Types
export type SearchType = 'search_vectors' | 'discover' | 'intelligent_search' | 'semantic_search';

export interface SearchRequest {
  query: string;
  limit?: number;
  type: SearchType;
  filters?: Record<string, unknown>;
}

export interface SearchResult extends Vector {
  collection?: string;
}

// Workspace Types
export interface WorkspaceDirectory {
  readonly path: string;
  collection_name: string;
  indexed_files: number;
  auto_index: boolean;
}

// Config Types
export interface VectorizerConfig {
  server: {
    host: string;
    port: number;
    auth?: {
      enabled: boolean;
      token?: string;
    };
  };
  storage: {
    data_dir: string;
    cache_size: number;
  };
  embedding: {
    provider: string;
    model: string;
    dimension: number;
  };
  performance: {
    threads: number;
    batch_size: number;
  };
  [key: string]: unknown;
}

// Log Types
export type LogLevel = 'DEBUG' | 'INFO' | 'WARN' | 'ERROR';

export interface LogEntry {
  readonly timestamp: string;
  readonly level: LogLevel;
  readonly message: string;
  readonly source?: string;
}

// Backup Types
export interface Backup {
  readonly id: string;
  readonly name: string;
  readonly date: string;
  readonly size: number;
  readonly collections: string[];
}

// API Response Types
export interface APIResponse<T = unknown> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}

// Indexing Progress Types
export interface IndexingProgress {
  overall_progress: number;
  total_collections: number;
  completed_collections: number;
  processing_collections: number;
  collections: Array<{
    name: string;
    status: IndexingStatus;
    progress: number;
    files_processed?: number;
  }>;
}

// File Watcher Metrics Types
export interface FileWatcherMetrics {
  timing: {
    avg_file_processing_ms: number;
    avg_discovery_ms: number;
    avg_sync_ms: number;
    uptime_seconds: number;
    peak_processing_ms: number;
  };
  files: {
    total_files_processed: number;
    files_processed_success: number;
    files_processed_error: number;
    files_skipped: number;
    files_in_progress: number;
    files_discovered: number;
    files_removed: number;
    files_indexed_realtime: number;
  };
  system: {
    memory_usage_bytes: number;
    cpu_usage_percent: number;
    thread_count: number;
  };
  status: {
    total_errors: number;
    errors_by_type: Record<string, number>;
    current_status: string;
    last_error?: string;
    health_score: number;
    restart_count: number;
  };
  collections: Record<string, {
    name: string;
    total_vectors: number;
    files_indexed: number;
    size_bytes: number;
    avg_indexing_time_ms: number;
  }>;
}

