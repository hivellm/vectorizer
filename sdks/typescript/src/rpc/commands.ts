/**
 * Typed wrappers around the v1 RPC command catalog.
 *
 * Each function corresponds to one entry in the wire spec's command
 * catalog (§ 6). Wrapper builds the positional `args` array per the
 * spec, calls {@link RpcClient.call}, and decodes the
 * {@link VectorizerValue} response into a typed JavaScript value.
 *
 * Phase 16 additions mirror the Rust SDK at `sdks/rust/src/rpc/commands.rs`
 * and cover the full server catalog in `rpc_capability_names()`.
 */

import { RpcClient, RpcServerError } from './client';
import { VectorizerValue, Value, asArray, asFloat, asInt, asStr, asBool, mapGet } from './types';

// ── Re-export all response types ─────────────────────────────────────────────

/** Collection metadata returned by `collections.get_info`. */
export interface CollectionInfo {
  name: string;
  vectorCount: number;
  documentCount: number;
  dimension: number;
  metric: string;
  createdAt: string;
  updatedAt: string;
}

/**
 * One result from `search.basic`. `payload` is an optional JSON
 * string — the server stores payloads as `serde_json::Value`; the RPC
 * layer ships them as a string because the wire `VectorizerValue`
 * enum doesn't model JSON directly. Decode with `JSON.parse` if
 * structured access is needed.
 */
export interface SearchHit {
  id: string;
  score: number;
  payload: string | null;
}

/** Response from `collections.create`. */
export interface CreateCollectionResult {
  name: string;
  dimension: number;
  metric: string;
  success: boolean;
}

/** Response from `collections.cleanup_empty`. */
export interface CleanupEmptyResult {
  removed: number;
  dryRun: boolean;
}

/** Response from `vectors.insert` / `vectors.insert_text` / `vectors.update`. */
export interface VectorWriteResult {
  id: string;
  success: boolean;
}

/** Per-item result inside batch responses. */
export interface BatchItemResult {
  index: number;
  id: string | null;
  status: string;
  error: string | null;
}

/** Response from `vectors.batch_insert` / `vectors.batch_insert_texts`. */
export interface BatchInsertResult {
  inserted: number;
  failed: number;
  results: BatchItemResult[];
}

/** Response from `vectors.batch_update`. */
export interface BatchUpdateResult {
  updated: number;
  failed: number;
  results: BatchItemResult[];
}

/** Response from `vectors.batch_delete`. */
export interface BatchDeleteResult {
  deleted: number;
  failed: number;
  results: BatchItemResult[];
}

/** One per-query result from `vectors.batch_search`. */
export interface BatchSearchResult {
  index: number;
  status: string;
  results: SearchHit[];
  error: string | null;
}

/** Response from `vectors.move`. */
export interface MoveRpcResult {
  src: string;
  dst: string;
  moved: number;
  failed: number;
}

/** Response from `vectors.copy`. */
export interface CopyRpcResult {
  src: string;
  dst: string;
  copied: number;
  failed: number;
}

/** Response from `vectors.delete_by_filter`. */
export interface DeleteByFilterRpcResult {
  scanned: number;
  matched: number;
  deleted: number;
}

/** Response from `vectors.bulk_update_metadata`. */
export interface BulkUpdateMetadataRpcResult {
  scanned: number;
  matched: number;
  updated: number;
}

/** Response from `vectors.set_expiry`. */
export interface SetExpiryResult {
  id: string;
  expiresAt: number;
  success: boolean;
}

/** Response from `vectors.embed`. */
export interface EmbedResult {
  embedding: number[];
  model: string;
  dimension: number;
}

/** Response from `vectors.list`. */
export interface VectorListResult {
  items: VectorizerValue[];
  total: number;
  page: number;
  limit: number;
}

/** HNSW traversal trace from `search.explain`. */
export interface SearchTrace {
  visitedNodes: number;
  efSearch: number;
  hnswSearchMs: number;
  totalMs: number;
}

/** Response from `search.explain`. */
export interface SearchExplainResult {
  hits: SearchHit[];
  collection: string;
  k: number;
  trace: SearchTrace;
}

/** Summary response from `discovery.discover`. */
export interface DiscoverResult {
  answerPrompt: string;
  sections: number;
  bullets: number;
  chunks: number;
}

/** One scored collection from `discovery.score_collections`. */
export interface ScoredCollection {
  name: string;
  score: number;
  vectorCount: number;
}

/** Response from `discovery.expand_queries`. */
export interface ExpandQueriesResult {
  originalQuery: string;
  expandedQueries: string[];
  count: number;
}

/** One chunk from `discovery.broad_discovery` / `discovery.semantic_focus`. */
export interface DiscoveryChunk {
  collection: string;
  score: number;
  contentPreview: string;
}

/** One bullet from `discovery.compress_evidence`. */
export interface CompressBullet {
  text: string;
  sourceId: string;
  score: number;
}

/** One section inside an answer plan. */
export interface AnswerPlanSection {
  title: string;
  bulletsCount: number;
}

/** Response from `discovery.build_answer_plan`. */
export interface AnswerPlanResult {
  sections: AnswerPlanSection[];
  totalBullets: number;
}

/** Response from `discovery.render_llm_prompt`. */
export interface RenderPromptResult {
  prompt: string;
  length: number;
  estimatedTokens: number;
}

/** Response from `graph.discovery_status`. */
export interface GraphDiscoveryStatus {
  totalNodes: number;
  nodesWithEdges: number;
  totalEdges: number;
  progressPercentage: number;
}

/** Response from `graph.discover_edges`. */
export interface DiscoverEdgesResult {
  success: boolean;
  totalNodes: number;
  nodesProcessed: number;
  nodesWithEdges: number;
  totalEdgesCreated: number;
}

/** Response from `graph.discover_edges_for_node`. */
export interface DiscoverEdgesForNodeResult {
  success: boolean;
  nodeId: string;
  edgesCreated: number;
}

/** Admin stats response from `admin.stats`. */
export interface AdminStats {
  collectionsCount: number;
  totalVectors: number;
  version: string;
}

/** Admin status response from `admin.status`. */
export interface AdminStatus {
  ready: boolean;
  collectionsCount: number;
  version: string;
}

/** Slow query config from `admin.slow_queries_config`. */
export interface SlowQueryConfigResult {
  thresholdMs: number;
  capacity: number;
  status: string;
}

/** Response from `auth.me`. */
export interface AuthMeResult {
  username: string;
  authenticated: boolean;
}

/** Response from `auth.refresh_token`. */
export interface RefreshTokenResult {
  accessToken: string;
  tokenType: string;
}

/** Response from `auth.validate_password`. */
export interface ValidatePasswordResult {
  valid: boolean;
  errors: string[];
}

/** Response from `auth.api_keys_create` / `auth.api_keys_create_scoped`. */
export interface ApiKeyCreated {
  apiKey: string;
  id: string;
  name: string;
}

/** Response from `auth.api_keys_rotate`. */
export interface RotatedApiKey {
  oldKeyId: string;
  newKeyId: string;
  newToken: string;
  graceUntil: string | null;
}

/** Response from `replication.configure`. */
export interface ReplicationConfigureResult {
  success: boolean;
  role: string;
  message: string;
}

/** Response from `cluster.rebalance_status`. */
export interface RebalanceStatus {
  status: string | null;
  message: string | null;
}

// ── Private decode helpers ────────────────────────────────────────────────────

function needStr(value: VectorizerValue, key: string, command: string): string {
  const v = mapGet(value, key);
  const s = v !== null ? asStr(v) : null;
  if (s === null) {
    throw new RpcServerError(`${command}: missing string field '${key}'`);
  }
  return s;
}

function needInt(value: VectorizerValue, key: string, command: string): number {
  const v = mapGet(value, key);
  const i = v !== null ? asInt(v) : null;
  if (i === null) {
    throw new RpcServerError(`${command}: missing int field '${key}'`);
  }
  return i;
}

function needBool(value: VectorizerValue, key: string, command: string): boolean {
  const v = mapGet(value, key);
  const b = v !== null ? asBool(v) : null;
  if (b === null) {
    throw new RpcServerError(`${command}: missing bool field '${key}'`);
  }
  return b;
}

function getInt(value: VectorizerValue, key: string): number {
  const v = mapGet(value, key);
  return v !== null ? (asInt(v) ?? 0) : 0;
}

function getFloat(value: VectorizerValue, key: string): number {
  const v = mapGet(value, key);
  return v !== null ? (asFloat(v) ?? 0.0) : 0.0;
}

function getStr(value: VectorizerValue, key: string, fallback = ''): string {
  const v = mapGet(value, key);
  return v !== null ? (asStr(v) ?? fallback) : fallback;
}

function getBool(value: VectorizerValue, key: string, fallback = false): boolean {
  const v = mapGet(value, key);
  return v !== null ? (asBool(v) ?? fallback) : fallback;
}

function decodeStringArray(v: VectorizerValue, command: string): string[] {
  const arr = asArray(v);
  if (arr === null) {
    throw new RpcServerError(`${command}: expected Array`);
  }
  const out: string[] = [];
  for (const item of arr) {
    const s = asStr(item);
    if (s !== null) out.push(s);
  }
  return out;
}

function decodeCollectionInfo(v: VectorizerValue): CollectionInfo {
  return {
    name: needStr(v, 'name', 'collections.get_info'),
    vectorCount: needInt(v, 'vector_count', 'collections.get_info'),
    documentCount: needInt(v, 'document_count', 'collections.get_info'),
    dimension: needInt(v, 'dimension', 'collections.get_info'),
    metric: needStr(v, 'metric', 'collections.get_info'),
    createdAt: needStr(v, 'created_at', 'collections.get_info'),
    updatedAt: needStr(v, 'updated_at', 'collections.get_info'),
  };
}

function decodeSearchHits(arr: VectorizerValue[]): SearchHit[] {
  const hits: SearchHit[] = [];
  for (const entry of arr) {
    const idV = mapGet(entry, 'id');
    const id = idV !== null ? asStr(idV) : null;
    if (id === null) continue;
    const scoreV = mapGet(entry, 'score');
    const score = scoreV !== null ? (asFloat(scoreV) ?? 0.0) : 0.0;
    const payloadV = mapGet(entry, 'payload');
    const payload = payloadV !== null ? asStr(payloadV) : null;
    hits.push({ id, score, payload });
  }
  return hits;
}

function decodeBatchItems(arr: VectorizerValue[]): BatchItemResult[] {
  return arr.map((entry) => ({
    index: getInt(entry, 'index'),
    id: (() => { const v = mapGet(entry, 'id'); return v !== null ? asStr(v) : null; })(),
    status: getStr(entry, 'status', 'unknown'),
    error: (() => { const v = mapGet(entry, 'error'); return v !== null ? asStr(v) : null; })(),
  }));
}

function decodeSearchBasic(v: VectorizerValue): SearchHit[] {
  const arr = asArray(v);
  if (arr === null) {
    throw new RpcServerError('search.basic: expected Array');
  }
  const hits: SearchHit[] = [];
  for (const entry of arr) {
    const idV = mapGet(entry, 'id');
    const id = idV !== null ? asStr(idV) : null;
    if (id === null) throw new RpcServerError("search.basic: hit missing 'id'");
    const scoreV = mapGet(entry, 'score');
    const score = scoreV !== null ? asFloat(scoreV) : null;
    if (score === null) throw new RpcServerError("search.basic: hit missing 'score'");
    const payloadV = mapGet(entry, 'payload');
    const payload = payloadV !== null ? asStr(payloadV) : null;
    hits.push({ id, score, payload });
  }
  return hits;
}

// ═══════════════════════════════════════════════════════════════════════════
// Collections
// ═══════════════════════════════════════════════════════════════════════════

/** `collections.list` — return every collection name visible to the principal. */
export async function listCollections(client: RpcClient): Promise<string[]> {
  return decodeStringArray(await client.call('collections.list', []), 'collections.list');
}

/** `collections.get_info` — return metadata for one collection. */
export async function getCollectionInfo(
  client: RpcClient,
  name: string,
): Promise<CollectionInfo> {
  const v = await client.call('collections.get_info', [Value.str(name)]);
  return decodeCollectionInfo(v);
}

/** `collections.create` — create a new collection. `config` is a Map with optional `dimension` and `metric`. */
export async function createCollection(
  client: RpcClient,
  name: string,
  config: VectorizerValue,
): Promise<CreateCollectionResult> {
  const v = await client.call('collections.create', [Value.str(name), config]);
  return {
    name: needStr(v, 'name', 'collections.create'),
    dimension: needInt(v, 'dimension', 'collections.create'),
    metric: needStr(v, 'metric', 'collections.create'),
    success: needBool(v, 'success', 'collections.create'),
  };
}

/** `collections.delete` — delete a collection. Returns success flag. */
export async function deleteCollection(client: RpcClient, name: string): Promise<boolean> {
  const v = await client.call('collections.delete', [Value.str(name)]);
  return needBool(v, 'success', 'collections.delete');
}

/** `collections.list_empty` — list collections containing zero vectors. */
export async function listEmptyCollections(client: RpcClient): Promise<string[]> {
  return decodeStringArray(await client.call('collections.list_empty', []), 'collections.list_empty');
}

/** `collections.cleanup_empty` — remove empty collections. Pass `dryRun: true` to preview. */
export async function cleanupEmptyCollections(
  client: RpcClient,
  dryRun: boolean,
): Promise<CleanupEmptyResult> {
  const config = Value.map([[Value.str('dry_run'), Value.bool(dryRun)]]);
  const v = await client.call('collections.cleanup_empty', [config]);
  return {
    removed: needInt(v, 'removed', 'collections.cleanup_empty'),
    dryRun: needBool(v, 'dry_run', 'collections.cleanup_empty'),
  };
}

/** `collections.force_save` — flush a collection's in-memory state to disk. */
export async function forceSaveCollection(client: RpcClient, name: string): Promise<boolean> {
  const v = await client.call('collections.force_save', [Value.str(name)]);
  return needBool(v, 'success', 'collections.force_save');
}

// ═══════════════════════════════════════════════════════════════════════════
// Vectors
// ═══════════════════════════════════════════════════════════════════════════

/**
 * `vectors.get` — fetch one vector by id. Returns the raw
 * {@link VectorizerValue} so callers can read whichever fields they
 * care about (`id`, `data`, `payload`, `document_id`).
 */
export async function getVector(
  client: RpcClient,
  collection: string,
  vectorId: string,
): Promise<VectorizerValue> {
  return client.call('vectors.get', [Value.str(collection), Value.str(vectorId)]);
}

/** `vectors.insert` — insert one pre-computed vector. `data` must match collection dimension. */
export async function insertVector(
  client: RpcClient,
  collection: string,
  id: string | null,
  data: number[],
  payload: VectorizerValue | null,
): Promise<VectorWriteResult> {
  const idVal = id !== null ? Value.str(id) : Value.null_();
  const dataVal = Value.array(data.map((f) => Value.float(f)));
  const args: VectorizerValue[] = [Value.str(collection), idVal, dataVal];
  if (payload !== null) args.push(payload);
  const v = await client.call('vectors.insert', args);
  return {
    id: needStr(v, 'id', 'vectors.insert'),
    success: needBool(v, 'success', 'vectors.insert'),
  };
}

/** `vectors.insert_text` — embed `text` server-side and insert. Auto-creates collection if absent. */
export async function insertTextVector(
  client: RpcClient,
  collection: string,
  id: string | null,
  text: string,
  payload: VectorizerValue | null,
): Promise<VectorWriteResult> {
  const idVal = id !== null ? Value.str(id) : Value.null_();
  const args: VectorizerValue[] = [Value.str(collection), idVal, Value.str(text)];
  if (payload !== null) args.push(payload);
  const v = await client.call('vectors.insert_text', args);
  return {
    id: needStr(v, 'id', 'vectors.insert_text'),
    success: needBool(v, 'success', 'vectors.insert_text'),
  };
}

/** `vectors.update` — replace a vector's data and/or payload. */
export async function updateVector(
  client: RpcClient,
  collection: string,
  id: string,
  data: number[],
  payload: VectorizerValue | null,
): Promise<VectorWriteResult> {
  const dataVal = Value.array(data.map((f) => Value.float(f)));
  const args: VectorizerValue[] = [Value.str(collection), Value.str(id), dataVal];
  if (payload !== null) args.push(payload);
  const v = await client.call('vectors.update', args);
  return {
    id: needStr(v, 'id', 'vectors.update'),
    success: needBool(v, 'success', 'vectors.update'),
  };
}

/** `vectors.delete` — delete one vector by id. Returns success flag. */
export async function deleteVectorRpc(
  client: RpcClient,
  collection: string,
  id: string,
): Promise<boolean> {
  const v = await client.call('vectors.delete', [Value.str(collection), Value.str(id)]);
  return needBool(v, 'success', 'vectors.delete');
}

/** `vectors.list` — page through vectors in a collection. `page` is zero-based; `limit` capped at 50. */
export async function listVectors(
  client: RpcClient,
  collection: string,
  page: number,
  limit: number,
): Promise<VectorListResult> {
  const v = await client.call('vectors.list', [
    Value.str(collection),
    Value.int(page),
    Value.int(limit),
  ]);
  const itemsV = mapGet(v, 'items');
  const items = itemsV !== null ? (asArray(itemsV) ?? []) : [];
  return {
    items,
    total: getInt(v, 'total'),
    page: getInt(v, 'page'),
    limit: getInt(v, 'limit'),
  };
}

/** `vectors.embed` — embed `text` server-side and return the embedding. */
export async function embedText(
  client: RpcClient,
  text: string,
  model: string | null,
): Promise<EmbedResult> {
  const args: VectorizerValue[] = [Value.str(text)];
  if (model !== null) args.push(Value.str(model));
  const v = await client.call('vectors.embed', args);
  const embeddingV = mapGet(v, 'embedding');
  const embArr = embeddingV !== null ? (asArray(embeddingV) ?? []) : [];
  return {
    embedding: embArr.map((x) => asFloat(x) ?? 0.0),
    model: getStr(v, 'model', 'bm25'),
    dimension: getInt(v, 'dimension'),
  };
}

/** `vectors.batch_insert` — insert multiple pre-computed vectors. Each item is a Map with `data` and optional `id`/`payload`. */
export async function batchInsertVectors(
  client: RpcClient,
  collection: string,
  items: VectorizerValue[],
): Promise<BatchInsertResult> {
  const v = await client.call('vectors.batch_insert', [Value.str(collection), Value.array(items)]);
  const resultsV = mapGet(v, 'results');
  const results = resultsV !== null ? decodeBatchItems(asArray(resultsV) ?? []) : [];
  return {
    inserted: getInt(v, 'inserted'),
    failed: getInt(v, 'failed'),
    results,
  };
}

/** `vectors.batch_insert_texts` — embed and insert multiple text items. Each item is a Map with `text` and optional `id`/`payload`. */
export async function batchInsertTexts(
  client: RpcClient,
  collection: string,
  items: VectorizerValue[],
): Promise<BatchInsertResult> {
  const v = await client.call('vectors.batch_insert_texts', [Value.str(collection), Value.array(items)]);
  const resultsV = mapGet(v, 'results');
  const results = resultsV !== null ? decodeBatchItems(asArray(resultsV) ?? []) : [];
  return {
    inserted: getInt(v, 'inserted'),
    failed: getInt(v, 'failed'),
    results,
  };
}

/** `vectors.batch_search` — run multiple searches in one round-trip. Each request is a Map with `collection`, `query`, and optional `limit`. */
export async function batchSearch(
  client: RpcClient,
  requests: VectorizerValue[],
): Promise<BatchSearchResult[]> {
  const v = await client.call('vectors.batch_search', [Value.array(requests)]);
  const arr = asArray(v);
  if (arr === null) {
    throw new RpcServerError('vectors.batch_search: expected Array');
  }
  return arr.map((entry) => {
    const resultsV = mapGet(entry, 'results');
    const results = resultsV !== null ? decodeSearchHits(asArray(resultsV) ?? []) : [];
    const errV = mapGet(entry, 'error');
    return {
      index: getInt(entry, 'index'),
      status: getStr(entry, 'status', 'unknown'),
      results,
      error: errV !== null ? asStr(errV) : null,
    };
  });
}

/** `vectors.batch_update` — update multiple vectors' data and/or payload. Each item is a Map with `id` and optionally `data`/`payload`. */
export async function batchUpdateVectors(
  client: RpcClient,
  collection: string,
  updates: VectorizerValue[],
): Promise<BatchUpdateResult> {
  const v = await client.call('vectors.batch_update', [Value.str(collection), Value.array(updates)]);
  const resultsV = mapGet(v, 'results');
  const results = resultsV !== null ? decodeBatchItems(asArray(resultsV) ?? []) : [];
  return {
    updated: getInt(v, 'updated'),
    failed: getInt(v, 'failed'),
    results,
  };
}

/** `vectors.batch_delete` — delete multiple vectors by id. */
export async function batchDeleteVectors(
  client: RpcClient,
  collection: string,
  ids: string[],
): Promise<BatchDeleteResult> {
  const idsVal = Value.array(ids.map((id) => Value.str(id)));
  const v = await client.call('vectors.batch_delete', [Value.str(collection), idsVal]);
  const resultsV = mapGet(v, 'results');
  const results = resultsV !== null ? decodeBatchItems(asArray(resultsV) ?? []) : [];
  return {
    deleted: getInt(v, 'deleted'),
    failed: getInt(v, 'failed'),
    results,
  };
}

/** `vectors.move` — move vectors from `src` to `dst` collection. Named `moveVectorsRpc` to avoid collision with REST SDK. */
export async function moveVectorsRpc(
  client: RpcClient,
  src: string,
  dst: string,
  ids: string[],
): Promise<MoveRpcResult> {
  const idsVal = Value.array(ids.map((id) => Value.str(id)));
  const v = await client.call('vectors.move', [Value.str(src), Value.str(dst), idsVal]);
  return {
    src: needStr(v, 'src', 'vectors.move'),
    dst: needStr(v, 'dst', 'vectors.move'),
    moved: getInt(v, 'moved'),
    failed: getInt(v, 'failed'),
  };
}

/** `vectors.copy` — copy vectors from `src` to `dst` without deleting. */
export async function copyVectorsRpc(
  client: RpcClient,
  src: string,
  dst: string,
  ids: string[],
): Promise<CopyRpcResult> {
  const idsVal = Value.array(ids.map((id) => Value.str(id)));
  const v = await client.call('vectors.copy', [Value.str(src), Value.str(dst), idsVal]);
  return {
    src: needStr(v, 'src', 'vectors.copy'),
    dst: needStr(v, 'dst', 'vectors.copy'),
    copied: getInt(v, 'copied'),
    failed: getInt(v, 'failed'),
  };
}

/** `vectors.delete_by_filter` — delete all vectors matching a Qdrant-style filter predicate. */
export async function deleteByFilterRpc(
  client: RpcClient,
  collection: string,
  filter: VectorizerValue,
): Promise<DeleteByFilterRpcResult> {
  const v = await client.call('vectors.delete_by_filter', [Value.str(collection), filter]);
  return {
    scanned: getInt(v, 'scanned'),
    matched: getInt(v, 'matched'),
    deleted: getInt(v, 'deleted'),
  };
}

/** `vectors.bulk_update_metadata` — apply a JSON-merge-patch to all vectors matching `filter`. */
export async function bulkUpdateMetadataRpc(
  client: RpcClient,
  collection: string,
  filter: VectorizerValue,
  patch: VectorizerValue,
): Promise<BulkUpdateMetadataRpcResult> {
  const v = await client.call('vectors.bulk_update_metadata', [Value.str(collection), filter, patch]);
  return {
    scanned: getInt(v, 'scanned'),
    matched: getInt(v, 'matched'),
    updated: getInt(v, 'updated'),
  };
}

/** `vectors.set_expiry` — attach a TTL to one vector. `expiresAt` is a Unix ms timestamp or RFC3339 string. */
export async function setVectorExpiry(
  client: RpcClient,
  collection: string,
  id: string,
  expiresAt: string,
): Promise<SetExpiryResult> {
  const v = await client.call('vectors.set_expiry', [
    Value.str(collection),
    Value.str(id),
    Value.str(expiresAt),
  ]);
  return {
    id: needStr(v, 'id', 'vectors.set_expiry'),
    expiresAt: needInt(v, 'expires_at', 'vectors.set_expiry'),
    success: needBool(v, 'success', 'vectors.set_expiry'),
  };
}

// ═══════════════════════════════════════════════════════════════════════════
// Search
// ═══════════════════════════════════════════════════════════════════════════

/**
 * `search.basic` — search `collection` for `query` and return up to
 * `limit` hits sorted by descending similarity.
 */
export async function searchBasic(
  client: RpcClient,
  collection: string,
  query: string,
  limit = 10,
): Promise<SearchHit[]> {
  const args: VectorizerValue[] = [Value.str(collection), Value.str(query), Value.int(limit)];
  return decodeSearchBasic(await client.call('search.basic', args));
}

/** `search.intelligent` — multi-collection intelligent search. `request` must contain `query` and optionally `collections`, `max_results`, `domain_expansion`. */
export async function searchIntelligent(
  client: RpcClient,
  request: VectorizerValue,
): Promise<VectorizerValue> {
  return client.call('search.intelligent', [request]);
}

/** `search.by_text` — search one collection by text query. */
export async function searchByText(
  client: RpcClient,
  collection: string,
  query: string,
  limit: number,
): Promise<SearchHit[]> {
  const v = await client.call('search.by_text', [
    Value.str(collection),
    Value.str(query),
    Value.int(limit),
  ]);
  const resultsV = mapGet(v, 'results');
  const arr = resultsV !== null ? (asArray(resultsV) ?? []) : [];
  return decodeSearchHits(arr);
}

/** `search.by_file` — file-content-based search. `request` is a Map describing the file query. */
export async function searchByFile(
  client: RpcClient,
  collection: string,
  request: VectorizerValue,
): Promise<SearchHit[]> {
  const v = await client.call('search.by_file', [Value.str(collection), request]);
  const resultsV = mapGet(v, 'results');
  const arr = resultsV !== null ? (asArray(resultsV) ?? []) : [];
  return decodeSearchHits(arr);
}

/** `search.hybrid` — RRF / weighted-combination hybrid dense+sparse search. `request` must contain `query` and optionally `alpha`, `dense_k`, `sparse_k`, `algorithm`. */
export async function searchHybrid(
  client: RpcClient,
  collection: string,
  request: VectorizerValue,
): Promise<SearchHit[]> {
  const v = await client.call('search.hybrid', [Value.str(collection), request]);
  const resultsV = mapGet(v, 'results');
  const arr = resultsV !== null ? (asArray(resultsV) ?? []) : [];
  return decodeSearchHits(arr);
}

/** `search.semantic` — semantic re-ranking search. `request` must contain `query` and `collection`. */
export async function searchSemantic(
  client: RpcClient,
  request: VectorizerValue,
): Promise<VectorizerValue> {
  return client.call('search.semantic', [request]);
}

/** `search.contextual` — context-filtered semantic search. `request` must contain `query` and `collection`. */
export async function searchContextual(
  client: RpcClient,
  request: VectorizerValue,
): Promise<VectorizerValue> {
  return client.call('search.contextual', [request]);
}

/** `search.multi_collection` — fan-out search across multiple collections. `request` must contain `query` and `collections`. */
export async function searchMultiCollection(
  client: RpcClient,
  request: VectorizerValue,
): Promise<VectorizerValue> {
  return client.call('search.multi_collection', [request]);
}

/** `search.explain` — run a vector search and return HNSW traversal trace. `request` must contain `vector` and optionally `k`. */
export async function searchExplain(
  client: RpcClient,
  collection: string,
  request: VectorizerValue,
): Promise<SearchExplainResult> {
  const v = await client.call('search.explain', [Value.str(collection), request]);
  const hitsV = mapGet(v, 'hits');
  const hits = hitsV !== null ? decodeSearchHits(asArray(hitsV) ?? []) : [];
  const traceV = mapGet(v, 'trace');
  const trace: SearchTrace = {
    visitedNodes: traceV !== null ? getInt(traceV, 'visited_nodes') : 0,
    efSearch: traceV !== null ? getInt(traceV, 'ef_search') : 0,
    hnswSearchMs: traceV !== null ? getFloat(traceV, 'hnsw_search_ms') : 0.0,
    totalMs: traceV !== null ? getFloat(traceV, 'total_ms') : 0.0,
  };
  return {
    hits,
    collection: getStr(v, 'collection'),
    k: getInt(v, 'k'),
    trace,
  };
}

// ═══════════════════════════════════════════════════════════════════════════
// Discovery
// ═══════════════════════════════════════════════════════════════════════════

/** `discovery.discover` — full discovery pipeline: embed → search → compress → build plan → render prompt. */
export async function discover(
  client: RpcClient,
  request: VectorizerValue,
): Promise<DiscoverResult> {
  const v = await client.call('discovery.discover', [request]);
  return {
    answerPrompt: needStr(v, 'answer_prompt', 'discovery.discover'),
    sections: getInt(v, 'sections'),
    bullets: getInt(v, 'bullets'),
    chunks: getInt(v, 'chunks'),
  };
}

/** `discovery.filter_collections` — filter collection list by query relevance. `request` must contain `query`. */
export async function filterCollections(
  client: RpcClient,
  request: VectorizerValue,
): Promise<string[]> {
  const v = await client.call('discovery.filter_collections', [request]);
  const filteredV = mapGet(v, 'filtered_collections');
  const arr = filteredV !== null ? (asArray(filteredV) ?? []) : [];
  return arr
    .map((entry) => { const nV = mapGet(entry, 'name'); return nV !== null ? asStr(nV) : null; })
    .filter((n): n is string => n !== null);
}

/** `discovery.score_collections` — score all collections for a query. `request` must contain `query`. */
export async function scoreCollections(
  client: RpcClient,
  request: VectorizerValue,
): Promise<ScoredCollection[]> {
  const v = await client.call('discovery.score_collections', [request]);
  const scoredV = mapGet(v, 'scored_collections');
  const arr = scoredV !== null ? (asArray(scoredV) ?? []) : [];
  return arr.map((entry) => ({
    name: getStr(entry, 'name'),
    score: getFloat(entry, 'score'),
    vectorCount: getInt(entry, 'vector_count'),
  }));
}

/** `discovery.expand_queries` — generate query variants via baseline expansion. `request` must contain `query`. */
export async function expandQueries(
  client: RpcClient,
  request: VectorizerValue,
): Promise<ExpandQueriesResult> {
  const v = await client.call('discovery.expand_queries', [request]);
  const expandedV = mapGet(v, 'expanded_queries');
  const expandedArr = expandedV !== null ? (asArray(expandedV) ?? []) : [];
  return {
    originalQuery: needStr(v, 'original_query', 'discovery.expand_queries'),
    expandedQueries: expandedArr.map((x) => asStr(x) ?? '').filter((s) => s !== ''),
    count: getInt(v, 'count'),
  };
}

/** `discovery.broad_discovery` — multi-query broad search across all collections. `request` must contain `queries`. */
export async function broadDiscovery(
  client: RpcClient,
  request: VectorizerValue,
): Promise<DiscoveryChunk[]> {
  const v = await client.call('discovery.broad_discovery', [request]);
  const chunksV = mapGet(v, 'chunks');
  const arr = chunksV !== null ? (asArray(chunksV) ?? []) : [];
  return arr.map((entry) => ({
    collection: getStr(entry, 'collection'),
    score: getFloat(entry, 'score'),
    contentPreview: getStr(entry, 'content_preview'),
  }));
}

/** `discovery.semantic_focus` — deep semantic search within one collection. `request` must contain `collection` and `queries`. */
export async function semanticFocus(
  client: RpcClient,
  request: VectorizerValue,
): Promise<DiscoveryChunk[]> {
  const v = await client.call('discovery.semantic_focus', [request]);
  const chunksV = mapGet(v, 'chunks');
  const arr = chunksV !== null ? (asArray(chunksV) ?? []) : [];
  return arr.map((entry) => ({
    collection: getStr(entry, 'collection'),
    score: getFloat(entry, 'score'),
    contentPreview: getStr(entry, 'content_preview'),
  }));
}

/** `discovery.promote_readme` — promote README chunks to the top of a chunk set. */
export async function promoteReadme(
  client: RpcClient,
  request: VectorizerValue,
): Promise<VectorizerValue> {
  return client.call('discovery.promote_readme', [request]);
}

/** `discovery.compress_evidence` — compress a chunk set into ranked bullets. `request` must contain `chunks`. */
export async function compressEvidence(
  client: RpcClient,
  request: VectorizerValue,
): Promise<CompressBullet[]> {
  const v = await client.call('discovery.compress_evidence', [request]);
  const bulletsV = mapGet(v, 'bullets');
  const arr = bulletsV !== null ? (asArray(bulletsV) ?? []) : [];
  return arr.map((entry) => ({
    text: getStr(entry, 'text'),
    sourceId: getStr(entry, 'source_id'),
    score: getFloat(entry, 'score'),
  }));
}

/** `discovery.build_answer_plan` — organise bullets into a structured answer plan. */
export async function buildAnswerPlan(
  client: RpcClient,
  request: VectorizerValue,
): Promise<AnswerPlanResult> {
  const v = await client.call('discovery.build_answer_plan', [request]);
  const sectionsV = mapGet(v, 'sections');
  const sectionsArr = sectionsV !== null ? (asArray(sectionsV) ?? []) : [];
  return {
    sections: sectionsArr.map((entry) => ({
      title: getStr(entry, 'title'),
      bulletsCount: getInt(entry, 'bullets_count'),
    })),
    totalBullets: getInt(v, 'total_bullets'),
  };
}

/** `discovery.render_llm_prompt` — render an answer plan into an LLM prompt string. */
export async function renderLlmPrompt(
  client: RpcClient,
  request: VectorizerValue,
): Promise<RenderPromptResult> {
  const v = await client.call('discovery.render_llm_prompt', [request]);
  return {
    prompt: needStr(v, 'prompt', 'discovery.render_llm_prompt'),
    length: getInt(v, 'length'),
    estimatedTokens: getInt(v, 'estimated_tokens'),
  };
}

// ═══════════════════════════════════════════════════════════════════════════
// File ops
// ═══════════════════════════════════════════════════════════════════════════

/** `file.content` — retrieve raw file content stored in a collection. `request` must contain `collection` and `file_path`. */
export async function fileContent(
  client: RpcClient,
  request: VectorizerValue,
): Promise<VectorizerValue> {
  return client.call('file.content', [request]);
}

/** `file.list` — list files indexed in a collection. `request` must contain `collection`. */
export async function fileList(
  client: RpcClient,
  request: VectorizerValue,
): Promise<VectorizerValue> {
  return client.call('file.list', [request]);
}

/** `file.summary` — extractive or structural summary of one file. `request` must contain `collection` and `file_path`. */
export async function fileSummary(
  client: RpcClient,
  request: VectorizerValue,
): Promise<VectorizerValue> {
  return client.call('file.summary', [request]);
}

/** `file.chunks` — retrieve ordered chunks for one file. `request` must contain `collection` and `file_path`. */
export async function fileChunks(
  client: RpcClient,
  request: VectorizerValue,
): Promise<VectorizerValue> {
  return client.call('file.chunks', [request]);
}

/** `file.outline` — directory-tree outline of a collection's files. `request` must contain `collection`. */
export async function fileOutline(
  client: RpcClient,
  request: VectorizerValue,
): Promise<VectorizerValue> {
  return client.call('file.outline', [request]);
}

/** `file.related` — find files semantically related to a given file. `request` must contain `collection` and `file_path`. */
export async function fileRelated(
  client: RpcClient,
  request: VectorizerValue,
): Promise<VectorizerValue> {
  return client.call('file.related', [request]);
}

/** `file.search_by_type` — search within files of specific extension types. `request` must contain `collection`, `query`, and `file_types`. */
export async function fileSearchByType(
  client: RpcClient,
  request: VectorizerValue,
): Promise<VectorizerValue> {
  return client.call('file.search_by_type', [request]);
}

// ═══════════════════════════════════════════════════════════════════════════
// Graph
// ═══════════════════════════════════════════════════════════════════════════

/** `graph.list_nodes` — list all graph nodes in a collection. */
export async function graphListNodes(
  client: RpcClient,
  collection: string,
): Promise<VectorizerValue> {
  return client.call('graph.list_nodes', [Value.str(collection)]);
}

/** `graph.neighbors` — fetch direct neighbors of a graph node. */
export async function graphNeighbors(
  client: RpcClient,
  collection: string,
  nodeId: string,
): Promise<VectorizerValue> {
  return client.call('graph.neighbors', [Value.str(collection), Value.str(nodeId)]);
}

/** `graph.find_related` — find nodes reachable within `maxHops` of a node. */
export async function graphFindRelated(
  client: RpcClient,
  collection: string,
  nodeId: string,
  maxHops: number,
): Promise<VectorizerValue> {
  return client.call('graph.find_related', [
    Value.str(collection),
    Value.str(nodeId),
    Value.int(maxHops),
  ]);
}

/** `graph.find_path` — shortest path between two graph nodes. */
export async function graphFindPath(
  client: RpcClient,
  collection: string,
  from: string,
  to: string,
): Promise<VectorizerValue> {
  return client.call('graph.find_path', [Value.str(collection), Value.str(from), Value.str(to)]);
}

/** `graph.create_edge` — create a directed edge between two nodes. `edge` must contain `source`, `target`, `relationship_type`. */
export async function graphCreateEdge(
  client: RpcClient,
  collection: string,
  edge: VectorizerValue,
): Promise<VectorizerValue> {
  return client.call('graph.create_edge', [Value.str(collection), edge]);
}

/** `graph.delete_edge` — remove an edge by its id. */
export async function graphDeleteEdge(
  client: RpcClient,
  collection: string,
  edgeId: string,
): Promise<VectorizerValue> {
  return client.call('graph.delete_edge', [Value.str(collection), Value.str(edgeId)]);
}

/** `graph.list_edges` — list all edges in a collection's graph. */
export async function graphListEdges(
  client: RpcClient,
  collection: string,
): Promise<VectorizerValue> {
  return client.call('graph.list_edges', [Value.str(collection)]);
}

/** `graph.discover_edges` — auto-discover edges by vector similarity across the whole collection. */
export async function graphDiscoverEdges(
  client: RpcClient,
  collection: string,
  request: VectorizerValue,
): Promise<DiscoverEdgesResult> {
  const v = await client.call('graph.discover_edges', [Value.str(collection), request]);
  return {
    success: getBool(v, 'success'),
    totalNodes: getInt(v, 'total_nodes'),
    nodesProcessed: getInt(v, 'nodes_processed'),
    nodesWithEdges: getInt(v, 'nodes_with_edges'),
    totalEdgesCreated: getInt(v, 'total_edges_created'),
  };
}

/** `graph.discover_edges_for_node` — auto-discover edges for one node. */
export async function graphDiscoverEdgesForNode(
  client: RpcClient,
  collection: string,
  nodeId: string,
  request: VectorizerValue,
): Promise<DiscoverEdgesForNodeResult> {
  const v = await client.call('graph.discover_edges_for_node', [
    Value.str(collection),
    Value.str(nodeId),
    request,
  ]);
  return {
    success: getBool(v, 'success'),
    nodeId: getStr(v, 'node_id', nodeId),
    edgesCreated: getInt(v, 'edges_created'),
  };
}

/** `graph.discovery_status` — percentage of nodes that have edges. */
export async function graphDiscoveryStatus(
  client: RpcClient,
  collection: string,
): Promise<GraphDiscoveryStatus> {
  const v = await client.call('graph.discovery_status', [Value.str(collection)]);
  return {
    totalNodes: getInt(v, 'total_nodes'),
    nodesWithEdges: getInt(v, 'nodes_with_edges'),
    totalEdges: getInt(v, 'total_edges'),
    progressPercentage: getFloat(v, 'progress_percentage'),
  };
}

// ═══════════════════════════════════════════════════════════════════════════
// Admin
// ═══════════════════════════════════════════════════════════════════════════

/** `admin.stats` — aggregate vector/collection counts. */
export async function adminStats(client: RpcClient): Promise<AdminStats> {
  const v = await client.call('admin.stats', []);
  return {
    collectionsCount: getInt(v, 'collections_count'),
    totalVectors: getInt(v, 'total_vectors'),
    version: getStr(v, 'version'),
  };
}

/** `admin.status` — readiness probe and basic counts. */
export async function adminStatus(client: RpcClient): Promise<AdminStatus> {
  const v = await client.call('admin.status', []);
  return {
    ready: getBool(v, 'ready'),
    collectionsCount: getInt(v, 'collections_count'),
    version: getStr(v, 'version'),
  };
}

/** `admin.logs` — in-process log entries. */
export async function adminLogs(
  client: RpcClient,
  request: VectorizerValue | null,
): Promise<VectorizerValue> {
  const args = request !== null ? [request] : [];
  return client.call('admin.logs', args);
}

/** `admin.indexing_progress` — how many collections have been indexed. */
export async function adminIndexingProgress(client: RpcClient): Promise<VectorizerValue> {
  return client.call('admin.indexing_progress', []);
}

/** `admin.config_get` — read the server's config.yml. */
export async function adminConfigGet(client: RpcClient): Promise<VectorizerValue> {
  return client.call('admin.config_get', []);
}

/** `admin.config_update` — write a patch map to config.yml. `patch` is a Map of config keys to new values. */
export async function adminConfigUpdate(
  client: RpcClient,
  patch: VectorizerValue,
): Promise<boolean> {
  const v = await client.call('admin.config_update', [patch]);
  return needBool(v, 'success', 'admin.config_update');
}

/** `admin.backups_list` — list available backup files. */
export async function adminBackupsList(client: RpcClient): Promise<VectorizerValue> {
  return client.call('admin.backups_list', []);
}

/** `admin.backups_create` — create a backup. `request` must contain `name`. */
export async function adminBackupsCreate(
  client: RpcClient,
  request: VectorizerValue,
): Promise<string> {
  const v = await client.call('admin.backups_create', [request]);
  return needStr(v, 'backup_id', 'admin.backups_create');
}

/** `admin.backups_restore` — restore a backup by id. `request` must contain `backup_id`. */
export async function adminBackupsRestore(
  client: RpcClient,
  request: VectorizerValue,
): Promise<boolean> {
  const v = await client.call('admin.backups_restore', [request]);
  return needBool(v, 'success', 'admin.backups_restore');
}

/** `admin.workspaces_list` — list configured workspaces. */
export async function adminWorkspacesList(client: RpcClient): Promise<VectorizerValue> {
  return client.call('admin.workspaces_list', []);
}

/** `admin.workspace_get` — read workspace.yml. */
export async function adminWorkspaceGet(client: RpcClient): Promise<VectorizerValue> {
  return client.call('admin.workspace_get', []);
}

/** `admin.workspace_add` — register a new workspace directory. `request` must contain `path` and `collection_name`. */
export async function adminWorkspaceAdd(
  client: RpcClient,
  request: VectorizerValue,
): Promise<VectorizerValue> {
  return client.call('admin.workspace_add', [request]);
}

/** `admin.workspace_remove` — remove a workspace by name. */
export async function adminWorkspaceRemove(client: RpcClient, name: string): Promise<boolean> {
  const v = await client.call('admin.workspace_remove', [Value.str(name)]);
  return needBool(v, 'success', 'admin.workspace_remove');
}

/** `admin.restart` — schedule a server restart. */
export async function adminRestart(client: RpcClient): Promise<boolean> {
  const v = await client.call('admin.restart', []);
  return needBool(v, 'success', 'admin.restart');
}

/** `admin.slow_queries_list` — retrieve the slow-query ring buffer. */
export async function adminSlowQueriesList(client: RpcClient): Promise<VectorizerValue> {
  return client.call('admin.slow_queries_list', []);
}

/** `admin.slow_queries_config` — configure slow-query threshold and capacity. `config` must contain `threshold_ms`. */
export async function adminSlowQueriesConfig(
  client: RpcClient,
  config: VectorizerValue,
): Promise<SlowQueryConfigResult> {
  const v = await client.call('admin.slow_queries_config', [config]);
  return {
    thresholdMs: getInt(v, 'threshold_ms'),
    capacity: getInt(v, 'capacity'),
    status: getStr(v, 'status', 'ok'),
  };
}

// ═══════════════════════════════════════════════════════════════════════════
// Auth / RBAC
// ═══════════════════════════════════════════════════════════════════════════

/** `auth.me` — return the authenticated principal's identity. */
export async function authMe(client: RpcClient): Promise<AuthMeResult> {
  const v = await client.call('auth.me', []);
  return {
    username: getStr(v, 'username', 'unknown'),
    authenticated: getBool(v, 'authenticated'),
  };
}

/** `auth.logout` — blacklist the supplied JWT so it cannot be reused. */
export async function authLogout(client: RpcClient, token: string): Promise<VectorizerValue> {
  return client.call('auth.logout', [Value.str(token)]);
}

/** `auth.refresh_token` — exchange a valid JWT for a fresh one. */
export async function authRefreshToken(
  client: RpcClient,
  token: string,
): Promise<RefreshTokenResult> {
  const v = await client.call('auth.refresh_token', [Value.str(token)]);
  return {
    accessToken: needStr(v, 'access_token', 'auth.refresh_token'),
    tokenType: getStr(v, 'token_type', 'Bearer'),
  };
}

/** `auth.validate_password` — check a plaintext password against the server's password policy. */
export async function authValidatePassword(
  client: RpcClient,
  password: string,
): Promise<ValidatePasswordResult> {
  const v = await client.call('auth.validate_password', [Value.str(password)]);
  const errorsV = mapGet(v, 'errors');
  const errorsArr = errorsV !== null ? (asArray(errorsV) ?? []) : [];
  return {
    valid: getBool(v, 'valid'),
    errors: errorsArr.map((x) => asStr(x) ?? '').filter((s) => s !== ''),
  };
}

/** `auth.api_keys_create` — create a new API key. `request` must contain `name`. */
export async function authApiKeysCreate(
  client: RpcClient,
  request: VectorizerValue,
): Promise<ApiKeyCreated> {
  const v = await client.call('auth.api_keys_create', [request]);
  return {
    apiKey: needStr(v, 'api_key', 'auth.api_keys_create'),
    id: needStr(v, 'id', 'auth.api_keys_create'),
    name: needStr(v, 'name', 'auth.api_keys_create'),
  };
}

/** `auth.api_keys_list` — list API keys for the current principal. */
export async function authApiKeysList(client: RpcClient): Promise<VectorizerValue> {
  return client.call('auth.api_keys_list', []);
}

/** `auth.api_keys_revoke` — permanently revoke an API key by id. */
export async function authApiKeysRevoke(client: RpcClient, keyId: string): Promise<boolean> {
  const v = await client.call('auth.api_keys_revoke', [Value.str(keyId)]);
  return needBool(v, 'success', 'auth.api_keys_revoke');
}

/**
 * `auth.api_keys_rotate` — rotate an API key (5-minute grace period).
 * Named `rotateApiKeyRpc` to avoid collision with the REST SDK's `rotateApiKey`.
 */
export async function rotateApiKeyRpc(client: RpcClient, keyId: string): Promise<RotatedApiKey> {
  const v = await client.call('auth.api_keys_rotate', [Value.str(keyId)]);
  const graceV = mapGet(v, 'grace_until');
  return {
    oldKeyId: needStr(v, 'old_key_id', 'auth.api_keys_rotate'),
    newKeyId: needStr(v, 'new_key_id', 'auth.api_keys_rotate'),
    newToken: needStr(v, 'new_token', 'auth.api_keys_rotate'),
    graceUntil: graceV !== null ? asStr(graceV) : null,
  };
}

/** `auth.api_keys_create_scoped` — create a collection-scoped API key. `request` must contain `name`. */
export async function authApiKeysCreateScoped(
  client: RpcClient,
  request: VectorizerValue,
): Promise<ApiKeyCreated> {
  const v = await client.call('auth.api_keys_create_scoped', [request]);
  return {
    apiKey: needStr(v, 'api_key', 'auth.api_keys_create_scoped'),
    id: needStr(v, 'id', 'auth.api_keys_create_scoped'),
    name: needStr(v, 'name', 'auth.api_keys_create_scoped'),
  };
}

/** `auth.introspect` — inspect a token's claims and blacklist status. */
export async function authIntrospect(client: RpcClient, token: string): Promise<VectorizerValue> {
  return client.call('auth.introspect', [Value.str(token)]);
}

/** `auth.audit` — query the auth audit log. `request` is a Map with optional `from`, `to`, `actor`, `action`, `limit`. */
export async function authAudit(
  client: RpcClient,
  request: VectorizerValue,
): Promise<VectorizerValue> {
  return client.call('auth.audit', [request]);
}

// ═══════════════════════════════════════════════════════════════════════════
// Replication
// ═══════════════════════════════════════════════════════════════════════════

/** `replication.status` — current replication role and replica list. */
export async function replicationStatus(client: RpcClient): Promise<VectorizerValue> {
  return client.call('replication.status', []);
}

/** `replication.configure` — set the replication role for this node. `config` must contain `role`. */
export async function replicationConfigure(
  client: RpcClient,
  config: VectorizerValue,
): Promise<ReplicationConfigureResult> {
  const v = await client.call('replication.configure', [config]);
  return {
    success: needBool(v, 'success', 'replication.configure'),
    role: needStr(v, 'role', 'replication.configure'),
    message: getStr(v, 'message'),
  };
}

/** `replication.stats` — replication throughput and lag statistics. */
export async function replicationStats(client: RpcClient): Promise<VectorizerValue> {
  return client.call('replication.stats', []);
}

/** `replication.replicas_list` — list connected replicas (master only). */
export async function replicationReplicasList(client: RpcClient): Promise<VectorizerValue> {
  return client.call('replication.replicas_list', []);
}

// ═══════════════════════════════════════════════════════════════════════════
// Cluster
// ═══════════════════════════════════════════════════════════════════════════

/** `cluster.failover` — promote a replica to master. */
export async function clusterFailover(
  client: RpcClient,
  replicaId: string,
): Promise<VectorizerValue> {
  return client.call('cluster.failover', [Value.str(replicaId)]);
}

/** `cluster.replica_resync` — force a replica to resync from master. */
export async function clusterReplicaResync(
  client: RpcClient,
  replicaId: string,
): Promise<VectorizerValue> {
  return client.call('cluster.replica_resync', [Value.str(replicaId)]);
}

/** `cluster.peer_add` — add a new peer to the cluster. `request` must contain `address`. */
export async function clusterPeerAdd(
  client: RpcClient,
  request: VectorizerValue,
): Promise<VectorizerValue> {
  return client.call('cluster.peer_add', [request]);
}

/** `cluster.rebalance` — trigger a shard rebalance across peers. */
export async function clusterRebalance(client: RpcClient): Promise<VectorizerValue> {
  return client.call('cluster.rebalance', []);
}

/** `cluster.rebalance_status` — check the status of an in-progress rebalance (or confirm idle). */
export async function clusterRebalanceStatus(client: RpcClient): Promise<RebalanceStatus> {
  const v = await client.call('cluster.rebalance_status', []);
  const statusV = mapGet(v, 'status');
  const messageV = mapGet(v, 'message');
  return {
    status: statusV !== null ? asStr(statusV) : null,
    message: messageV !== null ? asStr(messageV) : null,
  };
}

// ═══════════════════════════════════════════════════════════════════════════
// Module augmentation — attach all wrappers as methods on RpcClient
// ═══════════════════════════════════════════════════════════════════════════

declare module './client' {
  interface RpcClient {
    // Collections
    listCollections(): Promise<string[]>;
    getCollectionInfo(name: string): Promise<CollectionInfo>;
    createCollection(name: string, config: VectorizerValue): Promise<CreateCollectionResult>;
    deleteCollection(name: string): Promise<boolean>;
    listEmptyCollections(): Promise<string[]>;
    cleanupEmptyCollections(dryRun: boolean): Promise<CleanupEmptyResult>;
    forceSaveCollection(name: string): Promise<boolean>;
    // Vectors
    getVector(collection: string, vectorId: string): Promise<VectorizerValue>;
    insertVector(collection: string, id: string | null, data: number[], payload: VectorizerValue | null): Promise<VectorWriteResult>;
    insertTextVector(collection: string, id: string | null, text: string, payload: VectorizerValue | null): Promise<VectorWriteResult>;
    updateVector(collection: string, id: string, data: number[], payload: VectorizerValue | null): Promise<VectorWriteResult>;
    deleteVectorRpc(collection: string, id: string): Promise<boolean>;
    listVectors(collection: string, page: number, limit: number): Promise<VectorListResult>;
    embedText(text: string, model: string | null): Promise<EmbedResult>;
    batchInsertVectors(collection: string, items: VectorizerValue[]): Promise<BatchInsertResult>;
    batchInsertTexts(collection: string, items: VectorizerValue[]): Promise<BatchInsertResult>;
    batchSearch(requests: VectorizerValue[]): Promise<BatchSearchResult[]>;
    batchUpdateVectors(collection: string, updates: VectorizerValue[]): Promise<BatchUpdateResult>;
    batchDeleteVectors(collection: string, ids: string[]): Promise<BatchDeleteResult>;
    moveVectorsRpc(src: string, dst: string, ids: string[]): Promise<MoveRpcResult>;
    copyVectorsRpc(src: string, dst: string, ids: string[]): Promise<CopyRpcResult>;
    deleteByFilterRpc(collection: string, filter: VectorizerValue): Promise<DeleteByFilterRpcResult>;
    bulkUpdateMetadataRpc(collection: string, filter: VectorizerValue, patch: VectorizerValue): Promise<BulkUpdateMetadataRpcResult>;
    setVectorExpiry(collection: string, id: string, expiresAt: string): Promise<SetExpiryResult>;
    // Search
    searchBasic(collection: string, query: string, limit?: number): Promise<SearchHit[]>;
    searchIntelligent(request: VectorizerValue): Promise<VectorizerValue>;
    searchByText(collection: string, query: string, limit: number): Promise<SearchHit[]>;
    searchByFile(collection: string, request: VectorizerValue): Promise<SearchHit[]>;
    searchHybrid(collection: string, request: VectorizerValue): Promise<SearchHit[]>;
    searchSemantic(request: VectorizerValue): Promise<VectorizerValue>;
    searchContextual(request: VectorizerValue): Promise<VectorizerValue>;
    searchMultiCollection(request: VectorizerValue): Promise<VectorizerValue>;
    searchExplain(collection: string, request: VectorizerValue): Promise<SearchExplainResult>;
    // Discovery
    discover(request: VectorizerValue): Promise<DiscoverResult>;
    filterCollections(request: VectorizerValue): Promise<string[]>;
    scoreCollections(request: VectorizerValue): Promise<ScoredCollection[]>;
    expandQueries(request: VectorizerValue): Promise<ExpandQueriesResult>;
    broadDiscovery(request: VectorizerValue): Promise<DiscoveryChunk[]>;
    semanticFocus(request: VectorizerValue): Promise<DiscoveryChunk[]>;
    promoteReadme(request: VectorizerValue): Promise<VectorizerValue>;
    compressEvidence(request: VectorizerValue): Promise<CompressBullet[]>;
    buildAnswerPlan(request: VectorizerValue): Promise<AnswerPlanResult>;
    renderLlmPrompt(request: VectorizerValue): Promise<RenderPromptResult>;
    // File ops
    fileContent(request: VectorizerValue): Promise<VectorizerValue>;
    fileList(request: VectorizerValue): Promise<VectorizerValue>;
    fileSummary(request: VectorizerValue): Promise<VectorizerValue>;
    fileChunks(request: VectorizerValue): Promise<VectorizerValue>;
    fileOutline(request: VectorizerValue): Promise<VectorizerValue>;
    fileRelated(request: VectorizerValue): Promise<VectorizerValue>;
    fileSearchByType(request: VectorizerValue): Promise<VectorizerValue>;
    // Graph
    graphListNodes(collection: string): Promise<VectorizerValue>;
    graphNeighbors(collection: string, nodeId: string): Promise<VectorizerValue>;
    graphFindRelated(collection: string, nodeId: string, maxHops: number): Promise<VectorizerValue>;
    graphFindPath(collection: string, from: string, to: string): Promise<VectorizerValue>;
    graphCreateEdge(collection: string, edge: VectorizerValue): Promise<VectorizerValue>;
    graphDeleteEdge(collection: string, edgeId: string): Promise<VectorizerValue>;
    graphListEdges(collection: string): Promise<VectorizerValue>;
    graphDiscoverEdges(collection: string, request: VectorizerValue): Promise<DiscoverEdgesResult>;
    graphDiscoverEdgesForNode(collection: string, nodeId: string, request: VectorizerValue): Promise<DiscoverEdgesForNodeResult>;
    graphDiscoveryStatus(collection: string): Promise<GraphDiscoveryStatus>;
    // Admin
    adminStats(): Promise<AdminStats>;
    adminStatus(): Promise<AdminStatus>;
    adminLogs(request: VectorizerValue | null): Promise<VectorizerValue>;
    adminIndexingProgress(): Promise<VectorizerValue>;
    adminConfigGet(): Promise<VectorizerValue>;
    adminConfigUpdate(patch: VectorizerValue): Promise<boolean>;
    adminBackupsList(): Promise<VectorizerValue>;
    adminBackupsCreate(request: VectorizerValue): Promise<string>;
    adminBackupsRestore(request: VectorizerValue): Promise<boolean>;
    adminWorkspacesList(): Promise<VectorizerValue>;
    adminWorkspaceGet(): Promise<VectorizerValue>;
    adminWorkspaceAdd(request: VectorizerValue): Promise<VectorizerValue>;
    adminWorkspaceRemove(name: string): Promise<boolean>;
    adminRestart(): Promise<boolean>;
    adminSlowQueriesList(): Promise<VectorizerValue>;
    adminSlowQueriesConfig(config: VectorizerValue): Promise<SlowQueryConfigResult>;
    // Auth
    authMe(): Promise<AuthMeResult>;
    authLogout(token: string): Promise<VectorizerValue>;
    authRefreshToken(token: string): Promise<RefreshTokenResult>;
    authValidatePassword(password: string): Promise<ValidatePasswordResult>;
    authApiKeysCreate(request: VectorizerValue): Promise<ApiKeyCreated>;
    authApiKeysList(): Promise<VectorizerValue>;
    authApiKeysRevoke(keyId: string): Promise<boolean>;
    rotateApiKeyRpc(keyId: string): Promise<RotatedApiKey>;
    authApiKeysCreateScoped(request: VectorizerValue): Promise<ApiKeyCreated>;
    authIntrospect(token: string): Promise<VectorizerValue>;
    authAudit(request: VectorizerValue): Promise<VectorizerValue>;
    // Replication
    replicationStatus(): Promise<VectorizerValue>;
    replicationConfigure(config: VectorizerValue): Promise<ReplicationConfigureResult>;
    replicationStats(): Promise<VectorizerValue>;
    replicationReplicasList(): Promise<VectorizerValue>;
    // Cluster
    clusterFailover(replicaId: string): Promise<VectorizerValue>;
    clusterReplicaResync(replicaId: string): Promise<VectorizerValue>;
    clusterPeerAdd(request: VectorizerValue): Promise<VectorizerValue>;
    clusterRebalance(): Promise<VectorizerValue>;
    clusterRebalanceStatus(): Promise<RebalanceStatus>;
  }
}

// ── Prototype bindings ────────────────────────────────────────────────────────

// Collections
RpcClient.prototype.listCollections = function (this: RpcClient) { return listCollections(this); };
RpcClient.prototype.getCollectionInfo = function (this: RpcClient, name: string) { return getCollectionInfo(this, name); };
RpcClient.prototype.createCollection = function (this: RpcClient, name: string, config: VectorizerValue) { return createCollection(this, name, config); };
RpcClient.prototype.deleteCollection = function (this: RpcClient, name: string) { return deleteCollection(this, name); };
RpcClient.prototype.listEmptyCollections = function (this: RpcClient) { return listEmptyCollections(this); };
RpcClient.prototype.cleanupEmptyCollections = function (this: RpcClient, dryRun: boolean) { return cleanupEmptyCollections(this, dryRun); };
RpcClient.prototype.forceSaveCollection = function (this: RpcClient, name: string) { return forceSaveCollection(this, name); };

// Vectors
RpcClient.prototype.getVector = function (this: RpcClient, collection: string, vectorId: string) { return getVector(this, collection, vectorId); };
RpcClient.prototype.insertVector = function (this: RpcClient, collection: string, id: string | null, data: number[], payload: VectorizerValue | null) { return insertVector(this, collection, id, data, payload); };
RpcClient.prototype.insertTextVector = function (this: RpcClient, collection: string, id: string | null, text: string, payload: VectorizerValue | null) { return insertTextVector(this, collection, id, text, payload); };
RpcClient.prototype.updateVector = function (this: RpcClient, collection: string, id: string, data: number[], payload: VectorizerValue | null) { return updateVector(this, collection, id, data, payload); };
RpcClient.prototype.deleteVectorRpc = function (this: RpcClient, collection: string, id: string) { return deleteVectorRpc(this, collection, id); };
RpcClient.prototype.listVectors = function (this: RpcClient, collection: string, page: number, limit: number) { return listVectors(this, collection, page, limit); };
RpcClient.prototype.embedText = function (this: RpcClient, text: string, model: string | null) { return embedText(this, text, model); };
RpcClient.prototype.batchInsertVectors = function (this: RpcClient, collection: string, items: VectorizerValue[]) { return batchInsertVectors(this, collection, items); };
RpcClient.prototype.batchInsertTexts = function (this: RpcClient, collection: string, items: VectorizerValue[]) { return batchInsertTexts(this, collection, items); };
RpcClient.prototype.batchSearch = function (this: RpcClient, requests: VectorizerValue[]) { return batchSearch(this, requests); };
RpcClient.prototype.batchUpdateVectors = function (this: RpcClient, collection: string, updates: VectorizerValue[]) { return batchUpdateVectors(this, collection, updates); };
RpcClient.prototype.batchDeleteVectors = function (this: RpcClient, collection: string, ids: string[]) { return batchDeleteVectors(this, collection, ids); };
RpcClient.prototype.moveVectorsRpc = function (this: RpcClient, src: string, dst: string, ids: string[]) { return moveVectorsRpc(this, src, dst, ids); };
RpcClient.prototype.copyVectorsRpc = function (this: RpcClient, src: string, dst: string, ids: string[]) { return copyVectorsRpc(this, src, dst, ids); };
RpcClient.prototype.deleteByFilterRpc = function (this: RpcClient, collection: string, filter: VectorizerValue) { return deleteByFilterRpc(this, collection, filter); };
RpcClient.prototype.bulkUpdateMetadataRpc = function (this: RpcClient, collection: string, filter: VectorizerValue, patch: VectorizerValue) { return bulkUpdateMetadataRpc(this, collection, filter, patch); };
RpcClient.prototype.setVectorExpiry = function (this: RpcClient, collection: string, id: string, expiresAt: string) { return setVectorExpiry(this, collection, id, expiresAt); };

// Search
RpcClient.prototype.searchBasic = function (this: RpcClient, collection: string, query: string, limit?: number) { return searchBasic(this, collection, query, limit); };
RpcClient.prototype.searchIntelligent = function (this: RpcClient, request: VectorizerValue) { return searchIntelligent(this, request); };
RpcClient.prototype.searchByText = function (this: RpcClient, collection: string, query: string, limit: number) { return searchByText(this, collection, query, limit); };
RpcClient.prototype.searchByFile = function (this: RpcClient, collection: string, request: VectorizerValue) { return searchByFile(this, collection, request); };
RpcClient.prototype.searchHybrid = function (this: RpcClient, collection: string, request: VectorizerValue) { return searchHybrid(this, collection, request); };
RpcClient.prototype.searchSemantic = function (this: RpcClient, request: VectorizerValue) { return searchSemantic(this, request); };
RpcClient.prototype.searchContextual = function (this: RpcClient, request: VectorizerValue) { return searchContextual(this, request); };
RpcClient.prototype.searchMultiCollection = function (this: RpcClient, request: VectorizerValue) { return searchMultiCollection(this, request); };
RpcClient.prototype.searchExplain = function (this: RpcClient, collection: string, request: VectorizerValue) { return searchExplain(this, collection, request); };

// Discovery
RpcClient.prototype.discover = function (this: RpcClient, request: VectorizerValue) { return discover(this, request); };
RpcClient.prototype.filterCollections = function (this: RpcClient, request: VectorizerValue) { return filterCollections(this, request); };
RpcClient.prototype.scoreCollections = function (this: RpcClient, request: VectorizerValue) { return scoreCollections(this, request); };
RpcClient.prototype.expandQueries = function (this: RpcClient, request: VectorizerValue) { return expandQueries(this, request); };
RpcClient.prototype.broadDiscovery = function (this: RpcClient, request: VectorizerValue) { return broadDiscovery(this, request); };
RpcClient.prototype.semanticFocus = function (this: RpcClient, request: VectorizerValue) { return semanticFocus(this, request); };
RpcClient.prototype.promoteReadme = function (this: RpcClient, request: VectorizerValue) { return promoteReadme(this, request); };
RpcClient.prototype.compressEvidence = function (this: RpcClient, request: VectorizerValue) { return compressEvidence(this, request); };
RpcClient.prototype.buildAnswerPlan = function (this: RpcClient, request: VectorizerValue) { return buildAnswerPlan(this, request); };
RpcClient.prototype.renderLlmPrompt = function (this: RpcClient, request: VectorizerValue) { return renderLlmPrompt(this, request); };

// File ops
RpcClient.prototype.fileContent = function (this: RpcClient, request: VectorizerValue) { return fileContent(this, request); };
RpcClient.prototype.fileList = function (this: RpcClient, request: VectorizerValue) { return fileList(this, request); };
RpcClient.prototype.fileSummary = function (this: RpcClient, request: VectorizerValue) { return fileSummary(this, request); };
RpcClient.prototype.fileChunks = function (this: RpcClient, request: VectorizerValue) { return fileChunks(this, request); };
RpcClient.prototype.fileOutline = function (this: RpcClient, request: VectorizerValue) { return fileOutline(this, request); };
RpcClient.prototype.fileRelated = function (this: RpcClient, request: VectorizerValue) { return fileRelated(this, request); };
RpcClient.prototype.fileSearchByType = function (this: RpcClient, request: VectorizerValue) { return fileSearchByType(this, request); };

// Graph
RpcClient.prototype.graphListNodes = function (this: RpcClient, collection: string) { return graphListNodes(this, collection); };
RpcClient.prototype.graphNeighbors = function (this: RpcClient, collection: string, nodeId: string) { return graphNeighbors(this, collection, nodeId); };
RpcClient.prototype.graphFindRelated = function (this: RpcClient, collection: string, nodeId: string, maxHops: number) { return graphFindRelated(this, collection, nodeId, maxHops); };
RpcClient.prototype.graphFindPath = function (this: RpcClient, collection: string, from: string, to: string) { return graphFindPath(this, collection, from, to); };
RpcClient.prototype.graphCreateEdge = function (this: RpcClient, collection: string, edge: VectorizerValue) { return graphCreateEdge(this, collection, edge); };
RpcClient.prototype.graphDeleteEdge = function (this: RpcClient, collection: string, edgeId: string) { return graphDeleteEdge(this, collection, edgeId); };
RpcClient.prototype.graphListEdges = function (this: RpcClient, collection: string) { return graphListEdges(this, collection); };
RpcClient.prototype.graphDiscoverEdges = function (this: RpcClient, collection: string, request: VectorizerValue) { return graphDiscoverEdges(this, collection, request); };
RpcClient.prototype.graphDiscoverEdgesForNode = function (this: RpcClient, collection: string, nodeId: string, request: VectorizerValue) { return graphDiscoverEdgesForNode(this, collection, nodeId, request); };
RpcClient.prototype.graphDiscoveryStatus = function (this: RpcClient, collection: string) { return graphDiscoveryStatus(this, collection); };

// Admin
RpcClient.prototype.adminStats = function (this: RpcClient) { return adminStats(this); };
RpcClient.prototype.adminStatus = function (this: RpcClient) { return adminStatus(this); };
RpcClient.prototype.adminLogs = function (this: RpcClient, request: VectorizerValue | null) { return adminLogs(this, request); };
RpcClient.prototype.adminIndexingProgress = function (this: RpcClient) { return adminIndexingProgress(this); };
RpcClient.prototype.adminConfigGet = function (this: RpcClient) { return adminConfigGet(this); };
RpcClient.prototype.adminConfigUpdate = function (this: RpcClient, patch: VectorizerValue) { return adminConfigUpdate(this, patch); };
RpcClient.prototype.adminBackupsList = function (this: RpcClient) { return adminBackupsList(this); };
RpcClient.prototype.adminBackupsCreate = function (this: RpcClient, request: VectorizerValue) { return adminBackupsCreate(this, request); };
RpcClient.prototype.adminBackupsRestore = function (this: RpcClient, request: VectorizerValue) { return adminBackupsRestore(this, request); };
RpcClient.prototype.adminWorkspacesList = function (this: RpcClient) { return adminWorkspacesList(this); };
RpcClient.prototype.adminWorkspaceGet = function (this: RpcClient) { return adminWorkspaceGet(this); };
RpcClient.prototype.adminWorkspaceAdd = function (this: RpcClient, request: VectorizerValue) { return adminWorkspaceAdd(this, request); };
RpcClient.prototype.adminWorkspaceRemove = function (this: RpcClient, name: string) { return adminWorkspaceRemove(this, name); };
RpcClient.prototype.adminRestart = function (this: RpcClient) { return adminRestart(this); };
RpcClient.prototype.adminSlowQueriesList = function (this: RpcClient) { return adminSlowQueriesList(this); };
RpcClient.prototype.adminSlowQueriesConfig = function (this: RpcClient, config: VectorizerValue) { return adminSlowQueriesConfig(this, config); };

// Auth
RpcClient.prototype.authMe = function (this: RpcClient) { return authMe(this); };
RpcClient.prototype.authLogout = function (this: RpcClient, token: string) { return authLogout(this, token); };
RpcClient.prototype.authRefreshToken = function (this: RpcClient, token: string) { return authRefreshToken(this, token); };
RpcClient.prototype.authValidatePassword = function (this: RpcClient, password: string) { return authValidatePassword(this, password); };
RpcClient.prototype.authApiKeysCreate = function (this: RpcClient, request: VectorizerValue) { return authApiKeysCreate(this, request); };
RpcClient.prototype.authApiKeysList = function (this: RpcClient) { return authApiKeysList(this); };
RpcClient.prototype.authApiKeysRevoke = function (this: RpcClient, keyId: string) { return authApiKeysRevoke(this, keyId); };
RpcClient.prototype.rotateApiKeyRpc = function (this: RpcClient, keyId: string) { return rotateApiKeyRpc(this, keyId); };
RpcClient.prototype.authApiKeysCreateScoped = function (this: RpcClient, request: VectorizerValue) { return authApiKeysCreateScoped(this, request); };
RpcClient.prototype.authIntrospect = function (this: RpcClient, token: string) { return authIntrospect(this, token); };
RpcClient.prototype.authAudit = function (this: RpcClient, request: VectorizerValue) { return authAudit(this, request); };

// Replication
RpcClient.prototype.replicationStatus = function (this: RpcClient) { return replicationStatus(this); };
RpcClient.prototype.replicationConfigure = function (this: RpcClient, config: VectorizerValue) { return replicationConfigure(this, config); };
RpcClient.prototype.replicationStats = function (this: RpcClient) { return replicationStats(this); };
RpcClient.prototype.replicationReplicasList = function (this: RpcClient) { return replicationReplicasList(this); };

// Cluster
RpcClient.prototype.clusterFailover = function (this: RpcClient, replicaId: string) { return clusterFailover(this, replicaId); };
RpcClient.prototype.clusterReplicaResync = function (this: RpcClient, replicaId: string) { return clusterReplicaResync(this, replicaId); };
RpcClient.prototype.clusterPeerAdd = function (this: RpcClient, request: VectorizerValue) { return clusterPeerAdd(this, request); };
RpcClient.prototype.clusterRebalance = function (this: RpcClient) { return clusterRebalance(this); };
RpcClient.prototype.clusterRebalanceStatus = function (this: RpcClient) { return clusterRebalanceStatus(this); };
