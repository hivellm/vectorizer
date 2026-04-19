/**
 * Typed wrappers around the v1 RPC command catalog.
 *
 * Each function corresponds to one entry in the wire spec's command
 * catalog (§ 6). Wrapper builds the positional `args` array per the
 * spec, calls {@link RpcClient.call}, and decodes the
 * {@link VectorizerValue} response into a typed JavaScript value.
 */

import { RpcClient, RpcServerError } from './client';
import { VectorizerValue, Value, asArray, asFloat, asInt, asStr, mapGet } from './types';

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

function decodeCollectionsList(v: VectorizerValue): string[] {
  const arr = asArray(v);
  if (arr === null) {
    throw new RpcServerError('collections.list: expected Array');
  }
  const out: string[] = [];
  for (const item of arr) {
    const s = asStr(item);
    if (s !== null) out.push(s);
  }
  return out;
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

/** `collections.list` — return every collection name visible to the principal. */
export async function listCollections(client: RpcClient): Promise<string[]> {
  return decodeCollectionsList(await client.call('collections.list', []));
}

/** `collections.get_info` — return metadata for one collection. */
export async function getCollectionInfo(
  client: RpcClient,
  name: string,
): Promise<CollectionInfo> {
  const v = await client.call('collections.get_info', [Value.str(name)]);
  return decodeCollectionInfo(v);
}

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

// Attach as methods on RpcClient so callers can write
// `await client.listCollections()` instead of
// `await listCollections(client)`. Mirrors the Rust SDK where every
// wrapper is an `impl RpcClient` method.
declare module './client' {
  interface RpcClient {
    listCollections(): Promise<string[]>;
    getCollectionInfo(name: string): Promise<CollectionInfo>;
    getVector(collection: string, vectorId: string): Promise<VectorizerValue>;
    searchBasic(collection: string, query: string, limit?: number): Promise<SearchHit[]>;
  }
}

RpcClient.prototype.listCollections = function (this: RpcClient): Promise<string[]> {
  return listCollections(this);
};
RpcClient.prototype.getCollectionInfo = function (
  this: RpcClient,
  name: string,
): Promise<CollectionInfo> {
  return getCollectionInfo(this, name);
};
RpcClient.prototype.getVector = function (
  this: RpcClient,
  collection: string,
  vectorId: string,
): Promise<VectorizerValue> {
  return getVector(this, collection, vectorId);
};
RpcClient.prototype.searchBasic = function (
  this: RpcClient,
  collection: string,
  query: string,
  limit?: number,
): Promise<SearchHit[]> {
  return searchBasic(this, collection, query, limit);
};
