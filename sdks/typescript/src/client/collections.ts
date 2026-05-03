/**
 * Collection lifecycle: list / get / create / update / delete.
 *
 * All read paths honour the master/replica routing in `BaseClient`;
 * mutations always go to the master transport.
 */

import { BaseClient } from './_base';
import {
  Collection,
  CollectionInfo,
  CreateCollectionRequest,
  ReadOptions,
  UpdateCollectionRequest,
  validateCollection,
  validateCollectionInfo,
  validateCreateCollectionRequest,
} from '../models';
import type { NativeSnapshotInfo, ReencodeJob, ReindexJob, ReindexParams } from '../models';

export class CollectionsClient extends BaseClient {
  /** List every collection visible to the server. */
  public async listCollections(options?: ReadOptions): Promise<Collection[]> {
    try {
      const transport = this.getReadTransport(options);
      const response = await transport.get<{ collections: Collection[] } | Collection[]>(
        '/collections',
      );
      const collections = Array.isArray(response) ? response : response.collections || [];
      this.logger.debug('Collections listed', { count: collections.length });
      return collections;
    } catch (error) {
      this.logger.error('Failed to list collections', error);
      throw error;
    }
  }

  /** Fetch the full info record for one collection. */
  public async getCollection(
    collectionName: string,
    options?: ReadOptions,
  ): Promise<CollectionInfo> {
    try {
      const transport = this.getReadTransport(options);
      const response = await transport.get<CollectionInfo>(`/collections/${collectionName}`);
      validateCollectionInfo(response);
      this.logger.debug('Collection info retrieved', { collectionName });
      return response;
    } catch (error) {
      this.logger.error('Failed to get collection info', { collectionName, error });
      throw error;
    }
  }

  /** Create a collection. (Master-only.) */
  public async createCollection(request: CreateCollectionRequest): Promise<Collection> {
    try {
      validateCreateCollectionRequest(request);
      const transport = this.getWriteTransport();
      const response = await transport.post<Collection>('/collections', request);
      validateCollection(response);
      this.logger.info('Collection created', { collectionName: request.name });
      return response;
    } catch (error) {
      this.logger.error('Failed to create collection', { request, error });
      throw error;
    }
  }

  /** Update a collection's mutable fields. (Master-only.) */
  public async updateCollection(
    collectionName: string,
    request: UpdateCollectionRequest,
  ): Promise<Collection> {
    try {
      const transport = this.getWriteTransport();
      const response = await transport.put<Collection>(
        `/collections/${collectionName}`,
        request,
      );
      validateCollection(response);
      this.logger.info('Collection updated', { collectionName });
      return response;
    } catch (error) {
      this.logger.error('Failed to update collection', { collectionName, request, error });
      throw error;
    }
  }

  /** Drop a collection. (Master-only.) */
  public async deleteCollection(collectionName: string): Promise<void> {
    try {
      const transport = this.getWriteTransport();
      await transport.delete(`/collections/${collectionName}`);
      this.logger.info('Collection deleted', { collectionName });
    } catch (error) {
      this.logger.error('Failed to delete collection', { collectionName, error });
      throw error;
    }
  }

  /**
   * Re-quantize an existing collection in-place without re-embedding (phase13).
   *
   * Calls `POST /collections/{name}/reencode` with
   * `{"target_encoding": "<encoding>"}`. Valid encoding values:
   * `"sq8"`, `"binary"`, `"fp32"`.
   *
   * The server runs the reencode synchronously and returns
   * `{job_id, collection, state, target_encoding, progress}` on
   * completion. `state` will be `"completed"` on success.
   */
  public async reencodeCollection(
    collectionName: string,
    targetEncoding: string,
  ): Promise<ReencodeJob> {
    try {
      const transport = this.getWriteTransport();
      const response = await transport.post<ReencodeJob>(
        `/collections/${collectionName}/reencode`,
        { target_encoding: targetEncoding },
      );
      this.logger.info('Collection reencode started', { collectionName, targetEncoding, state: response.state });
      return response;
    } catch (error) {
      this.logger.error('Failed to reencode collection', { collectionName, targetEncoding, error });
      throw error;
    }
  }

  /**
   * Set or clear a per-collection TTL (phase13).
   *
   * Calls `POST /collections/{name}/ttl` with `{"ttl_secs": <secs>}`.
   * Pass `null` to clear the collection-level TTL. Existing vectors are NOT
   * retroactively expired; only subsequent insertions that carry `__expires_at`
   * in their payload are affected.
   *
   * For per-vector expiry use `setVectorExpiry` on the vectors surface.
   * Returns `void` (server responds with 204 No Content).
   */
  public async setCollectionTtl(
    collectionName: string,
    ttlSecs: number | null,
  ): Promise<void> {
    try {
      const transport = this.getWriteTransport();
      await transport.post(`/collections/${collectionName}/ttl`, { ttl_secs: ttlSecs });
      this.logger.info('Collection TTL set', { collectionName, ttlSecs });
    } catch (error) {
      this.logger.error('Failed to set collection TTL', { collectionName, ttlSecs, error });
      throw error;
    }
  }

  // ── Phase-14: schema-evolution methods ─────────────────────────────────────

  /**
   * Atomically rename a collection (phase14).
   *
   * Calls `POST /collections/{name}/rename` with `{ new_name }`.
   * The server keeps the old name as an in-memory alias for one minor version.
   */
  public async renameCollection(
    collectionName: string,
    newName: string,
  ): Promise<void> {
    try {
      const transport = this.getWriteTransport();
      await transport.post(`/collections/${collectionName}/rename`, { new_name: newName });
      this.logger.info('Collection renamed', { collectionName, newName });
    } catch (error) {
      this.logger.error('Failed to rename collection', { collectionName, newName, error });
      throw error;
    }
  }

  /**
   * Rebuild the HNSW index with new parameters (phase14).
   *
   * Calls `POST /collections/{name}/reindex` with `{ m, ef_construction, ef_search }`.
   * No re-embedding is required. Returns a {@link ReindexJob} with
   * `state === "completed"` on success.
   */
  public async reindexCollection(
    collectionName: string,
    params: ReindexParams,
  ): Promise<ReindexJob> {
    try {
      const transport = this.getWriteTransport();
      const response = await transport.post<ReindexJob>(
        `/collections/${collectionName}/reindex`,
        { m: params.m, ef_construction: params.ef_construction, ef_search: params.ef_search },
      );
      this.logger.info('Collection reindex completed', { collectionName, state: response.state });
      return response;
    } catch (error) {
      this.logger.error('Failed to reindex collection', { collectionName, params, error });
      throw error;
    }
  }

  /**
   * Create a native per-collection snapshot (phase14).
   *
   * Calls `POST /collections/{name}/snapshot` with an empty body.
   * Returns {@link NativeSnapshotInfo} with the snapshot id, collection,
   * creation timestamp, and compressed size in bytes.
   */
  public async snapshotCollectionNative(
    collectionName: string,
  ): Promise<NativeSnapshotInfo> {
    try {
      const transport = this.getWriteTransport();
      const response = await transport.post<NativeSnapshotInfo>(
        `/collections/${collectionName}/snapshot`,
        {},
      );
      this.logger.info('Native snapshot created', { collectionName, snapshotId: response.id });
      return response;
    } catch (error) {
      this.logger.error('Failed to create native snapshot', { collectionName, error });
      throw error;
    }
  }

  /**
   * List all native snapshots for a collection (phase14).
   *
   * Calls `GET /collections/{name}/snapshots`.
   * Returns snapshots newest-first as reported by the server.
   */
  public async listCollectionSnapshotsNative(
    collectionName: string,
  ): Promise<NativeSnapshotInfo[]> {
    try {
      const transport = this.getReadTransport();
      const response = await transport.get<{ collection: string; snapshots: NativeSnapshotInfo[]; total: number }>(
        `/collections/${collectionName}/snapshots`,
      );
      this.logger.debug('Native snapshots listed', { collectionName, total: response.total });
      return response.snapshots ?? [];
    } catch (error) {
      this.logger.error('Failed to list native snapshots', { collectionName, error });
      throw error;
    }
  }

  /**
   * Restore a collection from a native snapshot (phase14).
   *
   * Calls `POST /collections/{name}/snapshots/{id}/restore` with an empty body.
   * Drops the current in-memory state and replaces it with the snapshot data.
   */
  public async restoreCollectionSnapshotNative(
    collectionName: string,
    snapshotId: string,
  ): Promise<void> {
    try {
      const transport = this.getWriteTransport();
      await transport.post(
        `/collections/${collectionName}/snapshots/${snapshotId}/restore`,
        {},
      );
      this.logger.info('Native snapshot restored', { collectionName, snapshotId });
    } catch (error) {
      this.logger.error('Failed to restore native snapshot', { collectionName, snapshotId, error });
      throw error;
    }
  }
}
