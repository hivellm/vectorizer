/**
 * Wire-shape tests for phase-16 RPC typed wrappers.
 *
 * Each `describe` block covers one domain group. Tests build a
 * `VectorizerValue` map/array that matches the server's wire response,
 * call the standalone decode function (or build the result directly from
 * the same helpers used by the wrapper), and assert field correctness.
 *
 * No live server is required — these are pure unit tests on the decode
 * layer.
 */

import { describe, expect, test } from 'vitest';

import {
  Value,
  VectorizerValue,
  asInt,
  asStr,
  asBool,
  asFloat,
  asArray,
  mapGet,
} from '../src/rpc/types';

import type {
  CollectionInfo,
  CreateCollectionResult,
  CleanupEmptyResult,
  VectorWriteResult,
  BatchItemResult,
  BatchInsertResult,
  BatchSearchResult,
  MoveRpcResult,
  CopyRpcResult,
  DeleteByFilterRpcResult,
  BulkUpdateMetadataRpcResult,
  SetExpiryResult,
  EmbedResult,
  VectorListResult,
  SearchTrace,
  SearchExplainResult,
  SearchHit,
  DiscoverResult,
  ScoredCollection,
  ExpandQueriesResult,
  DiscoveryChunk,
  CompressBullet,
  AnswerPlanResult,
  RenderPromptResult,
  GraphDiscoveryStatus,
  DiscoverEdgesResult,
  DiscoverEdgesForNodeResult,
  AdminStats,
  AdminStatus,
  SlowQueryConfigResult,
  AuthMeResult,
  RefreshTokenResult,
  ValidatePasswordResult,
  ApiKeyCreated,
  RotatedApiKey,
  ReplicationConfigureResult,
  RebalanceStatus,
} from '../src/rpc/commands';

// ── Shared helpers (mirrors commands.ts private helpers) ──────────────────────

function needStr(v: VectorizerValue, key: string): string {
  const val = mapGet(v, key);
  const s = val !== null ? asStr(val) : null;
  if (s === null) throw new Error(`missing string field '${key}'`);
  return s;
}

function needInt(v: VectorizerValue, key: string): number {
  const val = mapGet(v, key);
  const i = val !== null ? asInt(val) : null;
  if (i === null) throw new Error(`missing int field '${key}'`);
  return i;
}

function needBool(v: VectorizerValue, key: string): boolean {
  const val = mapGet(v, key);
  const b = val !== null ? asBool(val) : null;
  if (b === null) throw new Error(`missing bool field '${key}'`);
  return b;
}

function getInt(v: VectorizerValue, key: string): number {
  const val = mapGet(v, key);
  return val !== null ? (asInt(val) ?? 0) : 0;
}

function getFloat(v: VectorizerValue, key: string): number {
  const val = mapGet(v, key);
  return val !== null ? (asFloat(val) ?? 0.0) : 0.0;
}

function getStr(v: VectorizerValue, key: string, fallback = ''): string {
  const val = mapGet(v, key);
  return val !== null ? (asStr(val) ?? fallback) : fallback;
}

function getBool(v: VectorizerValue, key: string, fallback = false): boolean {
  const val = mapGet(v, key);
  return val !== null ? (asBool(val) ?? fallback) : fallback;
}

function decodeSearchHits(arr: VectorizerValue[]): SearchHit[] {
  return arr.flatMap((entry) => {
    const idV = mapGet(entry, 'id');
    const id = idV !== null ? asStr(idV) : null;
    if (id === null) return [];
    const scoreV = mapGet(entry, 'score');
    const score = scoreV !== null ? (asFloat(scoreV) ?? 0.0) : 0.0;
    const payloadV = mapGet(entry, 'payload');
    const payload = payloadV !== null ? asStr(payloadV) : null;
    return [{ id, score, payload }];
  });
}

function decodeBatchItems(arr: VectorizerValue[]): BatchItemResult[] {
  return arr.map((entry) => ({
    index: getInt(entry, 'index'),
    id: (() => { const v = mapGet(entry, 'id'); return v !== null ? asStr(v) : null; })(),
    status: getStr(entry, 'status', 'unknown'),
    error: (() => { const v = mapGet(entry, 'error'); return v !== null ? asStr(v) : null; })(),
  }));
}

// ══════════════════════════════════════════════════════════════════════════════
// Collections
// ══════════════════════════════════════════════════════════════════════════════

describe('collections — wire shapes', () => {
  test('get_info decodes all seven fields', () => {
    const wire = Value.map([
      [Value.str('name'), Value.str('my-col')],
      [Value.str('vector_count'), Value.int(100)],
      [Value.str('document_count'), Value.int(50)],
      [Value.str('dimension'), Value.int(384)],
      [Value.str('metric'), Value.str('Cosine')],
      [Value.str('created_at'), Value.str('2026-01-01T00:00:00Z')],
      [Value.str('updated_at'), Value.str('2026-01-02T00:00:00Z')],
    ]);
    const info: CollectionInfo = {
      name: needStr(wire, 'name'),
      vectorCount: needInt(wire, 'vector_count'),
      documentCount: needInt(wire, 'document_count'),
      dimension: needInt(wire, 'dimension'),
      metric: needStr(wire, 'metric'),
      createdAt: needStr(wire, 'created_at'),
      updatedAt: needStr(wire, 'updated_at'),
    };
    expect(info.name).toBe('my-col');
    expect(info.vectorCount).toBe(100);
    expect(info.dimension).toBe(384);
    expect(info.metric).toBe('Cosine');
    expect(info.createdAt).toBe('2026-01-01T00:00:00Z');
  });

  test('create decodes name/dimension/metric/success', () => {
    const wire = Value.map([
      [Value.str('name'), Value.str('new-col')],
      [Value.str('dimension'), Value.int(512)],
      [Value.str('metric'), Value.str('euclidean')],
      [Value.str('success'), Value.bool(true)],
    ]);
    const r: CreateCollectionResult = {
      name: needStr(wire, 'name'),
      dimension: needInt(wire, 'dimension'),
      metric: needStr(wire, 'metric'),
      success: needBool(wire, 'success'),
    };
    expect(r.name).toBe('new-col');
    expect(r.dimension).toBe(512);
    expect(r.success).toBe(true);
  });

  test('cleanup_empty decodes removed/dryRun', () => {
    const wire = Value.map([
      [Value.str('removed'), Value.int(3)],
      [Value.str('dry_run'), Value.bool(false)],
    ]);
    const r: CleanupEmptyResult = {
      removed: needInt(wire, 'removed'),
      dryRun: needBool(wire, 'dry_run'),
    };
    expect(r.removed).toBe(3);
    expect(r.dryRun).toBe(false);
  });
});

// ══════════════════════════════════════════════════════════════════════════════
// Vectors
// ══════════════════════════════════════════════════════════════════════════════

describe('vectors — wire shapes', () => {
  test('insert/update result decodes id/success', () => {
    const wire = Value.map([
      [Value.str('id'), Value.str('abc-123')],
      [Value.str('success'), Value.bool(true)],
    ]);
    const r: VectorWriteResult = {
      id: needStr(wire, 'id'),
      success: needBool(wire, 'success'),
    };
    expect(r.id).toBe('abc-123');
    expect(r.success).toBe(true);
  });

  test('batch_insert result decodes inserted/failed/results', () => {
    const itemWire = Value.map([
      [Value.str('index'), Value.int(0)],
      [Value.str('id'), Value.str('x1')],
      [Value.str('status'), Value.str('ok')],
    ]);
    const wire = Value.map([
      [Value.str('inserted'), Value.int(1)],
      [Value.str('failed'), Value.int(0)],
      [Value.str('results'), Value.array([itemWire])],
    ]);
    const resultsV = mapGet(wire, 'results');
    const items = resultsV !== null ? decodeBatchItems(asArray(resultsV) ?? []) : [];
    const r: BatchInsertResult = {
      inserted: getInt(wire, 'inserted'),
      failed: getInt(wire, 'failed'),
      results: items,
    };
    expect(r.inserted).toBe(1);
    expect(r.results).toHaveLength(1);
    expect(r.results[0]!.id).toBe('x1');
    expect(r.results[0]!.status).toBe('ok');
  });

  test('batch_search result decodes per-query results', () => {
    const hitWire = Value.map([
      [Value.str('id'), Value.str('v1')],
      [Value.str('score'), Value.float(0.88)],
    ]);
    const entryWire = Value.map([
      [Value.str('index'), Value.int(0)],
      [Value.str('status'), Value.str('ok')],
      [Value.str('results'), Value.array([hitWire])],
    ]);
    const outerArr = asArray(Value.array([entryWire])) ?? [];
    const decoded: BatchSearchResult[] = outerArr.map((entry) => {
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
    expect(decoded).toHaveLength(1);
    expect(decoded[0]!.status).toBe('ok');
    expect(decoded[0]!.results[0]!.id).toBe('v1');
    expect(decoded[0]!.results[0]!.score).toBeCloseTo(0.88, 9);
  });

  test('vectors.move decodes src/dst/moved/failed', () => {
    const wire = Value.map([
      [Value.str('src'), Value.str('col-a')],
      [Value.str('dst'), Value.str('col-b')],
      [Value.str('moved'), Value.int(5)],
      [Value.str('failed'), Value.int(1)],
    ]);
    const r: MoveRpcResult = {
      src: needStr(wire, 'src'),
      dst: needStr(wire, 'dst'),
      moved: getInt(wire, 'moved'),
      failed: getInt(wire, 'failed'),
    };
    expect(r.src).toBe('col-a');
    expect(r.dst).toBe('col-b');
    expect(r.moved).toBe(5);
    expect(r.failed).toBe(1);
  });

  test('vectors.copy decodes src/dst/copied/failed', () => {
    const wire = Value.map([
      [Value.str('src'), Value.str('col-x')],
      [Value.str('dst'), Value.str('col-y')],
      [Value.str('copied'), Value.int(3)],
      [Value.str('failed'), Value.int(0)],
    ]);
    const r: CopyRpcResult = {
      src: needStr(wire, 'src'),
      dst: needStr(wire, 'dst'),
      copied: getInt(wire, 'copied'),
      failed: getInt(wire, 'failed'),
    };
    expect(r.copied).toBe(3);
  });

  test('delete_by_filter decodes scanned/matched/deleted', () => {
    const wire = Value.map([
      [Value.str('scanned'), Value.int(100)],
      [Value.str('matched'), Value.int(20)],
      [Value.str('deleted'), Value.int(20)],
    ]);
    const r: DeleteByFilterRpcResult = {
      scanned: getInt(wire, 'scanned'),
      matched: getInt(wire, 'matched'),
      deleted: getInt(wire, 'deleted'),
    };
    expect(r.scanned).toBe(100);
    expect(r.matched).toBe(20);
    expect(r.deleted).toBe(20);
  });

  test('bulk_update_metadata decodes scanned/matched/updated', () => {
    const wire = Value.map([
      [Value.str('scanned'), Value.int(50)],
      [Value.str('matched'), Value.int(10)],
      [Value.str('updated'), Value.int(10)],
    ]);
    const r: BulkUpdateMetadataRpcResult = {
      scanned: getInt(wire, 'scanned'),
      matched: getInt(wire, 'matched'),
      updated: getInt(wire, 'updated'),
    };
    expect(r.updated).toBe(10);
  });

  test('set_expiry decodes id/expiresAt/success', () => {
    const wire = Value.map([
      [Value.str('id'), Value.str('v99')],
      [Value.str('expires_at'), Value.int(9_999_999)],
      [Value.str('success'), Value.bool(true)],
    ]);
    const r: SetExpiryResult = {
      id: needStr(wire, 'id'),
      expiresAt: needInt(wire, 'expires_at'),
      success: needBool(wire, 'success'),
    };
    expect(r.id).toBe('v99');
    expect(r.expiresAt).toBe(9_999_999);
    expect(r.success).toBe(true);
  });

  test('embed decodes embedding/model/dimension', () => {
    const wire = Value.map([
      [Value.str('embedding'), Value.array([Value.float(0.1), Value.float(0.2), Value.float(0.3)])],
      [Value.str('model'), Value.str('minilm')],
      [Value.str('dimension'), Value.int(3)],
    ]);
    const embV = mapGet(wire, 'embedding');
    const embArr = embV !== null ? (asArray(embV) ?? []) : [];
    const r: EmbedResult = {
      embedding: embArr.map((x) => asFloat(x) ?? 0.0),
      model: getStr(wire, 'model', 'bm25'),
      dimension: getInt(wire, 'dimension'),
    };
    expect(r.embedding).toHaveLength(3);
    expect(r.embedding[0]).toBeCloseTo(0.1, 9);
    expect(r.model).toBe('minilm');
    expect(r.dimension).toBe(3);
  });

  test('vectors.list decodes items/total/page/limit', () => {
    const itemWire = Value.map([[Value.str('id'), Value.str('v1')]]);
    const wire = Value.map([
      [Value.str('items'), Value.array([itemWire])],
      [Value.str('total'), Value.int(1)],
      [Value.str('page'), Value.int(0)],
      [Value.str('limit'), Value.int(10)],
    ]);
    const itemsV = mapGet(wire, 'items');
    const r: VectorListResult = {
      items: itemsV !== null ? (asArray(itemsV) ?? []) : [],
      total: getInt(wire, 'total'),
      page: getInt(wire, 'page'),
      limit: getInt(wire, 'limit'),
    };
    expect(r.items).toHaveLength(1);
    expect(r.total).toBe(1);
    expect(r.page).toBe(0);
    expect(r.limit).toBe(10);
  });
});

// ══════════════════════════════════════════════════════════════════════════════
// Search
// ══════════════════════════════════════════════════════════════════════════════

describe('search — wire shapes', () => {
  test('search.explain decodes hits + trace', () => {
    const hitWire = Value.map([
      [Value.str('id'), Value.str('vec-0')],
      [Value.str('score'), Value.float(0.95)],
    ]);
    const traceWire = Value.map([
      [Value.str('visited_nodes'), Value.int(50)],
      [Value.str('ef_search'), Value.int(100)],
      [Value.str('hnsw_search_ms'), Value.float(1.5)],
      [Value.str('total_ms'), Value.float(2.0)],
    ]);
    const wire = Value.map([
      [Value.str('hits'), Value.array([hitWire])],
      [Value.str('collection'), Value.str('docs')],
      [Value.str('k'), Value.int(10)],
      [Value.str('trace'), traceWire],
    ]);
    const hitsV = mapGet(wire, 'hits');
    const hits = hitsV !== null ? decodeSearchHits(asArray(hitsV) ?? []) : [];
    const traceV = mapGet(wire, 'trace');
    const trace: SearchTrace = {
      visitedNodes: traceV !== null ? getInt(traceV, 'visited_nodes') : 0,
      efSearch: traceV !== null ? getInt(traceV, 'ef_search') : 0,
      hnswSearchMs: traceV !== null ? getFloat(traceV, 'hnsw_search_ms') : 0.0,
      totalMs: traceV !== null ? getFloat(traceV, 'total_ms') : 0.0,
    };
    const r: SearchExplainResult = {
      hits,
      collection: getStr(wire, 'collection'),
      k: getInt(wire, 'k'),
      trace,
    };
    expect(r.hits).toHaveLength(1);
    expect(r.hits[0]!.id).toBe('vec-0');
    expect(r.collection).toBe('docs');
    expect(r.trace.visitedNodes).toBe(50);
    expect(r.trace.hnswSearchMs).toBeCloseTo(1.5, 9);
  });
});

// ══════════════════════════════════════════════════════════════════════════════
// Discovery
// ══════════════════════════════════════════════════════════════════════════════

describe('discovery — wire shapes', () => {
  test('discover result decodes answerPrompt/sections/bullets/chunks', () => {
    const wire = Value.map([
      [Value.str('answer_prompt'), Value.str('Here is your answer...')],
      [Value.str('sections'), Value.int(3)],
      [Value.str('bullets'), Value.int(12)],
      [Value.str('chunks'), Value.int(8)],
    ]);
    const r: DiscoverResult = {
      answerPrompt: needStr(wire, 'answer_prompt'),
      sections: getInt(wire, 'sections'),
      bullets: getInt(wire, 'bullets'),
      chunks: getInt(wire, 'chunks'),
    };
    expect(r.answerPrompt).toBe('Here is your answer...');
    expect(r.bullets).toBe(12);
  });

  test('score_collections decodes name/score/vectorCount', () => {
    const entryWire = Value.map([
      [Value.str('name'), Value.str('code')],
      [Value.str('score'), Value.float(0.87)],
      [Value.str('vector_count'), Value.int(200)],
    ]);
    const r: ScoredCollection = {
      name: getStr(entryWire, 'name'),
      score: getFloat(entryWire, 'score'),
      vectorCount: getInt(entryWire, 'vector_count'),
    };
    expect(r.name).toBe('code');
    expect(r.score).toBeCloseTo(0.87, 9);
    expect(r.vectorCount).toBe(200);
  });

  test('expand_queries decodes originalQuery/expandedQueries/count', () => {
    const wire = Value.map([
      [Value.str('original_query'), Value.str('rust')],
      [Value.str('expanded_queries'), Value.array([Value.str('rust programming'), Value.str('rust lang')])],
      [Value.str('count'), Value.int(2)],
    ]);
    const expandedV = mapGet(wire, 'expanded_queries');
    const expandedArr = expandedV !== null ? (asArray(expandedV) ?? []) : [];
    const r: ExpandQueriesResult = {
      originalQuery: needStr(wire, 'original_query'),
      expandedQueries: expandedArr.map((x) => asStr(x) ?? '').filter((s) => s !== ''),
      count: getInt(wire, 'count'),
    };
    expect(r.originalQuery).toBe('rust');
    expect(r.expandedQueries).toHaveLength(2);
    expect(r.count).toBe(2);
  });

  test('broad_discovery decodes chunks with collection/score/contentPreview', () => {
    const entryWire = Value.map([
      [Value.str('collection'), Value.str('docs')],
      [Value.str('score'), Value.float(0.72)],
      [Value.str('content_preview'), Value.str('Some text...')],
    ]);
    const r: DiscoveryChunk = {
      collection: getStr(entryWire, 'collection'),
      score: getFloat(entryWire, 'score'),
      contentPreview: getStr(entryWire, 'content_preview'),
    };
    expect(r.collection).toBe('docs');
    expect(r.score).toBeCloseTo(0.72, 9);
    expect(r.contentPreview).toBe('Some text...');
  });

  test('compress_evidence decodes bullets with text/sourceId/score', () => {
    const entryWire = Value.map([
      [Value.str('text'), Value.str('Important finding.')],
      [Value.str('source_id'), Value.str('doc-1')],
      [Value.str('score'), Value.float(0.9)],
    ]);
    const r: CompressBullet = {
      text: getStr(entryWire, 'text'),
      sourceId: getStr(entryWire, 'source_id'),
      score: getFloat(entryWire, 'score'),
    };
    expect(r.text).toBe('Important finding.');
    expect(r.sourceId).toBe('doc-1');
    expect(r.score).toBeCloseTo(0.9, 9);
  });

  test('build_answer_plan decodes sections/totalBullets', () => {
    const sectionWire = Value.map([
      [Value.str('title'), Value.str('Overview')],
      [Value.str('bullets_count'), Value.int(4)],
    ]);
    const wire = Value.map([
      [Value.str('sections'), Value.array([sectionWire])],
      [Value.str('total_bullets'), Value.int(4)],
    ]);
    const sectionsV = mapGet(wire, 'sections');
    const sectionsArr = sectionsV !== null ? (asArray(sectionsV) ?? []) : [];
    const r: AnswerPlanResult = {
      sections: sectionsArr.map((entry) => ({
        title: getStr(entry, 'title'),
        bulletsCount: getInt(entry, 'bullets_count'),
      })),
      totalBullets: getInt(wire, 'total_bullets'),
    };
    expect(r.sections).toHaveLength(1);
    expect(r.sections[0]!.title).toBe('Overview');
    expect(r.totalBullets).toBe(4);
  });

  test('render_llm_prompt decodes prompt/length/estimatedTokens', () => {
    const wire = Value.map([
      [Value.str('prompt'), Value.str('You are an AI...')],
      [Value.str('length'), Value.int(1024)],
      [Value.str('estimated_tokens'), Value.int(256)],
    ]);
    const r: RenderPromptResult = {
      prompt: needStr(wire, 'prompt'),
      length: getInt(wire, 'length'),
      estimatedTokens: getInt(wire, 'estimated_tokens'),
    };
    expect(r.prompt).toBe('You are an AI...');
    expect(r.length).toBe(1024);
    expect(r.estimatedTokens).toBe(256);
  });
});

// ══════════════════════════════════════════════════════════════════════════════
// File ops
// ══════════════════════════════════════════════════════════════════════════════

describe('file ops — wire shapes', () => {
  test('file ops return raw VectorizerValue (pass-through)', () => {
    // File ops return raw VectorizerValue — validate the shape is a Map
    const wire = Value.map([
      [Value.str('content'), Value.str('file content here')],
      [Value.str('size'), Value.int(18)],
    ]);
    // Verify it is a Map kind (what the server returns)
    expect(wire.kind).toBe('Map');
    const contentV = mapGet(wire, 'content');
    expect(contentV).not.toBeNull();
    expect(asStr(contentV!)).toBe('file content here');
  });
});

// ══════════════════════════════════════════════════════════════════════════════
// Graph
// ══════════════════════════════════════════════════════════════════════════════

describe('graph — wire shapes', () => {
  test('graph.discovery_status decodes totalNodes/nodesWithEdges/totalEdges/progressPercentage', () => {
    const wire = Value.map([
      [Value.str('total_nodes'), Value.int(100)],
      [Value.str('nodes_with_edges'), Value.int(75)],
      [Value.str('total_edges'), Value.int(200)],
      [Value.str('progress_percentage'), Value.float(75.0)],
    ]);
    const r: GraphDiscoveryStatus = {
      totalNodes: getInt(wire, 'total_nodes'),
      nodesWithEdges: getInt(wire, 'nodes_with_edges'),
      totalEdges: getInt(wire, 'total_edges'),
      progressPercentage: getFloat(wire, 'progress_percentage'),
    };
    expect(r.totalNodes).toBe(100);
    expect(r.nodesWithEdges).toBe(75);
    expect(r.progressPercentage).toBeCloseTo(75.0, 9);
  });

  test('graph.discover_edges decodes all five fields', () => {
    const wire = Value.map([
      [Value.str('success'), Value.bool(true)],
      [Value.str('total_nodes'), Value.int(50)],
      [Value.str('nodes_processed'), Value.int(50)],
      [Value.str('nodes_with_edges'), Value.int(40)],
      [Value.str('total_edges_created'), Value.int(120)],
    ]);
    const r: DiscoverEdgesResult = {
      success: getBool(wire, 'success'),
      totalNodes: getInt(wire, 'total_nodes'),
      nodesProcessed: getInt(wire, 'nodes_processed'),
      nodesWithEdges: getInt(wire, 'nodes_with_edges'),
      totalEdgesCreated: getInt(wire, 'total_edges_created'),
    };
    expect(r.success).toBe(true);
    expect(r.totalEdgesCreated).toBe(120);
  });

  test('graph.discover_edges_for_node decodes success/nodeId/edgesCreated', () => {
    const wire = Value.map([
      [Value.str('success'), Value.bool(true)],
      [Value.str('node_id'), Value.str('node-42')],
      [Value.str('edges_created'), Value.int(7)],
    ]);
    const r: DiscoverEdgesForNodeResult = {
      success: getBool(wire, 'success'),
      nodeId: getStr(wire, 'node_id', 'unknown'),
      edgesCreated: getInt(wire, 'edges_created'),
    };
    expect(r.nodeId).toBe('node-42');
    expect(r.edgesCreated).toBe(7);
  });
});

// ══════════════════════════════════════════════════════════════════════════════
// Admin
// ══════════════════════════════════════════════════════════════════════════════

describe('admin — wire shapes', () => {
  test('admin.stats decodes collectionsCount/totalVectors/version', () => {
    const wire = Value.map([
      [Value.str('collections_count'), Value.int(5)],
      [Value.str('total_vectors'), Value.int(1000)],
      [Value.str('version'), Value.str('3.8.0')],
    ]);
    const r: AdminStats = {
      collectionsCount: getInt(wire, 'collections_count'),
      totalVectors: getInt(wire, 'total_vectors'),
      version: getStr(wire, 'version'),
    };
    expect(r.collectionsCount).toBe(5);
    expect(r.totalVectors).toBe(1000);
    expect(r.version).toBe('3.8.0');
  });

  test('admin.status decodes ready/collectionsCount/version', () => {
    const wire = Value.map([
      [Value.str('ready'), Value.bool(true)],
      [Value.str('collections_count'), Value.int(3)],
      [Value.str('version'), Value.str('3.8.0')],
    ]);
    const r: AdminStatus = {
      ready: getBool(wire, 'ready'),
      collectionsCount: getInt(wire, 'collections_count'),
      version: getStr(wire, 'version'),
    };
    expect(r.ready).toBe(true);
    expect(r.collectionsCount).toBe(3);
  });

  test('admin.slow_queries_config decodes thresholdMs/capacity/status', () => {
    const wire = Value.map([
      [Value.str('threshold_ms'), Value.int(200)],
      [Value.str('capacity'), Value.int(100)],
      [Value.str('status'), Value.str('ok')],
    ]);
    const r: SlowQueryConfigResult = {
      thresholdMs: getInt(wire, 'threshold_ms'),
      capacity: getInt(wire, 'capacity'),
      status: getStr(wire, 'status', 'ok'),
    };
    expect(r.thresholdMs).toBe(200);
    expect(r.capacity).toBe(100);
    expect(r.status).toBe('ok');
  });
});

// ══════════════════════════════════════════════════════════════════════════════
// Auth
// ══════════════════════════════════════════════════════════════════════════════

describe('auth — wire shapes', () => {
  test('auth.me decodes username/authenticated', () => {
    const wire = Value.map([
      [Value.str('username'), Value.str('alice')],
      [Value.str('authenticated'), Value.bool(true)],
    ]);
    const r: AuthMeResult = {
      username: getStr(wire, 'username', 'unknown'),
      authenticated: getBool(wire, 'authenticated'),
    };
    expect(r.username).toBe('alice');
    expect(r.authenticated).toBe(true);
  });

  test('auth.refresh_token decodes accessToken/tokenType', () => {
    const wire = Value.map([
      [Value.str('access_token'), Value.str('eyJhbGci...')],
      [Value.str('token_type'), Value.str('Bearer')],
    ]);
    const r: RefreshTokenResult = {
      accessToken: needStr(wire, 'access_token'),
      tokenType: getStr(wire, 'token_type', 'Bearer'),
    };
    expect(r.accessToken).toBe('eyJhbGci...');
    expect(r.tokenType).toBe('Bearer');
  });

  test('auth.validate_password decodes valid/errors', () => {
    const wire = Value.map([
      [Value.str('valid'), Value.bool(false)],
      [Value.str('errors'), Value.array([Value.str('too short'), Value.str('no uppercase')])],
    ]);
    const errorsV = mapGet(wire, 'errors');
    const errorsArr = errorsV !== null ? (asArray(errorsV) ?? []) : [];
    const r: ValidatePasswordResult = {
      valid: getBool(wire, 'valid'),
      errors: errorsArr.map((x) => asStr(x) ?? '').filter((s) => s !== ''),
    };
    expect(r.valid).toBe(false);
    expect(r.errors).toHaveLength(2);
    expect(r.errors[0]).toBe('too short');
  });

  test('auth.api_keys_create decodes apiKey/id/name', () => {
    const wire = Value.map([
      [Value.str('api_key'), Value.str('vk-secret-token')],
      [Value.str('id'), Value.str('key-001')],
      [Value.str('name'), Value.str('my-key')],
    ]);
    const r: ApiKeyCreated = {
      apiKey: needStr(wire, 'api_key'),
      id: needStr(wire, 'id'),
      name: needStr(wire, 'name'),
    };
    expect(r.apiKey).toBe('vk-secret-token');
    expect(r.id).toBe('key-001');
    expect(r.name).toBe('my-key');
  });

  test('auth.api_keys_rotate decodes oldKeyId/newKeyId/newToken/graceUntil', () => {
    const wire = Value.map([
      [Value.str('old_key_id'), Value.str('key-old')],
      [Value.str('new_key_id'), Value.str('key-new')],
      [Value.str('new_token'), Value.str('vk-new-secret')],
      [Value.str('grace_until'), Value.str('2026-05-02T12:00:00Z')],
    ]);
    const graceV = mapGet(wire, 'grace_until');
    const r: RotatedApiKey = {
      oldKeyId: needStr(wire, 'old_key_id'),
      newKeyId: needStr(wire, 'new_key_id'),
      newToken: needStr(wire, 'new_token'),
      graceUntil: graceV !== null ? asStr(graceV) : null,
    };
    expect(r.oldKeyId).toBe('key-old');
    expect(r.newKeyId).toBe('key-new');
    expect(r.graceUntil).toBe('2026-05-02T12:00:00Z');
  });
});

// ══════════════════════════════════════════════════════════════════════════════
// Replication
// ══════════════════════════════════════════════════════════════════════════════

describe('replication — wire shapes', () => {
  test('replication.configure decodes success/role/message', () => {
    const wire = Value.map([
      [Value.str('success'), Value.bool(true)],
      [Value.str('role'), Value.str('master')],
      [Value.str('message'), Value.str('Replication configured; restart required.')],
    ]);
    const r: ReplicationConfigureResult = {
      success: needBool(wire, 'success'),
      role: needStr(wire, 'role'),
      message: getStr(wire, 'message'),
    };
    expect(r.success).toBe(true);
    expect(r.role).toBe('master');
    expect(r.message).toContain('restart');
  });
});

// ══════════════════════════════════════════════════════════════════════════════
// Cluster
// ══════════════════════════════════════════════════════════════════════════════

describe('cluster — wire shapes', () => {
  test('cluster.rebalance_status decodes status/message (idle case)', () => {
    const wire = Value.map([
      [Value.str('status'), Value.str('idle')],
      [Value.str('message'), Value.str('No rebalance in progress.')],
    ]);
    const statusV = mapGet(wire, 'status');
    const messageV = mapGet(wire, 'message');
    const r: RebalanceStatus = {
      status: statusV !== null ? asStr(statusV) : null,
      message: messageV !== null ? asStr(messageV) : null,
    };
    expect(r.status).toBe('idle');
    expect(r.message).toBe('No rebalance in progress.');
  });

  test('cluster.rebalance_status handles null fields gracefully', () => {
    // Server may omit status/message when no rebalance is running
    const wire = Value.map([]);
    const statusV = mapGet(wire, 'status');
    const messageV = mapGet(wire, 'message');
    const r: RebalanceStatus = {
      status: statusV !== null ? asStr(statusV) : null,
      message: messageV !== null ? asStr(messageV) : null,
    };
    expect(r.status).toBeNull();
    expect(r.message).toBeNull();
  });
});
