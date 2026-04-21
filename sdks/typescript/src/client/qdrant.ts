/**
 * Qdrant compatibility surface.
 *
 * Mirrors the Qdrant 1.14 REST API verbatim so existing Qdrant clients
 * can talk to Vectorizer with no code changes. Methods are loosely
 * typed (`unknown`) because Qdrant payloads are open-ended; callers
 * cast to the schema they expect.
 */

import { BaseClient } from './_base';
import { ReadOptions } from '../models';

export class QdrantClient extends BaseClient {
  /** List collections via Qdrant API. */
  public async qdrantListCollections(options?: ReadOptions): Promise<unknown> {
    try {
      const transport = this.getReadTransport(options);
      return await transport.get('/qdrant/collections');
    } catch (error) {
      this.logger.error('Failed to list Qdrant collections', { error });
      throw error;
    }
  }

  /** Get one collection via Qdrant API. */
  public async qdrantGetCollection(name: string, options?: ReadOptions): Promise<unknown> {
    try {
      const transport = this.getReadTransport(options);
      return await transport.get(`/qdrant/collections/${name}`);
    } catch (error) {
      this.logger.error('Failed to get Qdrant collection', { name, error });
      throw error;
    }
  }

  /** Create collection via Qdrant API. (Master-only.) */
  public async qdrantCreateCollection(name: string, config: unknown): Promise<unknown> {
    try {
      const transport = this.getWriteTransport();
      return await transport.put(`/qdrant/collections/${name}`, { config });
    } catch (error) {
      this.logger.error('Failed to create Qdrant collection', { name, error });
      throw error;
    }
  }

  /** Upsert points via Qdrant API. (Master-only.) */
  public async qdrantUpsertPoints(
    collection: string,
    points: unknown[],
    wait: boolean = false,
  ): Promise<unknown> {
    try {
      const transport = this.getWriteTransport();
      return await transport.put(`/qdrant/collections/${collection}/points`, { points, wait });
    } catch (error) {
      this.logger.error('Failed to upsert Qdrant points', { collection, error });
      throw error;
    }
  }

  /** Search points via Qdrant API. */
  public async qdrantSearchPoints(
    collection: string,
    vector: number[],
    limit: number = 10,
    filter?: unknown,
    withPayload: boolean = true,
    withVector: boolean = false,
    options?: ReadOptions,
  ): Promise<unknown> {
    try {
      const transport = this.getReadTransport(options);
      const payload: Record<string, unknown> = {
        vector,
        limit,
        with_payload: withPayload,
        with_vector: withVector,
      };
      if (filter) {
        payload['filter'] = filter;
      }
      return await transport.post(`/qdrant/collections/${collection}/points/search`, payload);
    } catch (error) {
      this.logger.error('Failed to search Qdrant points', { collection, error });
      throw error;
    }
  }

  /** Delete points via Qdrant API. (Master-only.) */
  public async qdrantDeletePoints(
    collection: string,
    pointIds: (string | number)[],
    wait: boolean = false,
  ): Promise<unknown> {
    try {
      const transport = this.getWriteTransport();
      return await transport.post(`/qdrant/collections/${collection}/points/delete`, {
        points: pointIds,
        wait,
      });
    } catch (error) {
      this.logger.error('Failed to delete Qdrant points', { collection, error });
      throw error;
    }
  }

  /** Retrieve points by ID via Qdrant API. */
  public async qdrantRetrievePoints(
    collection: string,
    pointIds: (string | number)[],
    withPayload: boolean = true,
    withVector: boolean = false,
    options?: ReadOptions,
  ): Promise<unknown> {
    try {
      const transport = this.getReadTransport(options);
      const params = [
        `ids=${encodeURIComponent(pointIds.join(','))}`,
        `with_payload=${String(withPayload)}`,
        `with_vector=${String(withVector)}`,
      ].join('&');
      return await transport.get(`/qdrant/collections/${collection}/points?${params}`);
    } catch (error) {
      this.logger.error('Failed to retrieve Qdrant points', { collection, error });
      throw error;
    }
  }

  /** Count points via Qdrant API. */
  public async qdrantCountPoints(
    collection: string,
    filter?: unknown,
    options?: ReadOptions,
  ): Promise<unknown> {
    try {
      const transport = this.getReadTransport(options);
      const payload: Record<string, unknown> = {};
      if (filter) {
        payload['filter'] = filter;
      }
      return await transport.post(`/qdrant/collections/${collection}/points/count`, payload);
    } catch (error) {
      this.logger.error('Failed to count Qdrant points', { collection, error });
      throw error;
    }
  }

  // -- Snapshots ----------------------------------------------------------

  public async qdrantListCollectionSnapshots(collection: string): Promise<unknown> {
    try {
      return await this.transport.get(`/qdrant/collections/${collection}/snapshots`);
    } catch (error) {
      this.logger.error('Failed to list collection snapshots', { collection, error });
      throw error;
    }
  }

  public async qdrantCreateCollectionSnapshot(collection: string): Promise<unknown> {
    try {
      return await this.transport.post(`/qdrant/collections/${collection}/snapshots`, {});
    } catch (error) {
      this.logger.error('Failed to create collection snapshot', { collection, error });
      throw error;
    }
  }

  public async qdrantDeleteCollectionSnapshot(
    collection: string,
    snapshotName: string,
  ): Promise<unknown> {
    try {
      return await this.transport.delete(
        `/qdrant/collections/${collection}/snapshots/${snapshotName}`,
      );
    } catch (error) {
      this.logger.error('Failed to delete collection snapshot', {
        collection,
        snapshotName,
        error,
      });
      throw error;
    }
  }

  public async qdrantRecoverCollectionSnapshot(
    collection: string,
    location: string,
  ): Promise<unknown> {
    try {
      return await this.transport.post(
        `/qdrant/collections/${collection}/snapshots/recover`,
        { location },
      );
    } catch (error) {
      this.logger.error('Failed to recover collection snapshot', { collection, location, error });
      throw error;
    }
  }

  public async qdrantListAllSnapshots(): Promise<unknown> {
    try {
      return await this.transport.get('/qdrant/snapshots');
    } catch (error) {
      this.logger.error('Failed to list all snapshots', { error });
      throw error;
    }
  }

  public async qdrantCreateFullSnapshot(): Promise<unknown> {
    try {
      return await this.transport.post('/qdrant/snapshots', {});
    } catch (error) {
      this.logger.error('Failed to create full snapshot', { error });
      throw error;
    }
  }

  // -- Sharding ------------------------------------------------------------

  public async qdrantListShardKeys(collection: string): Promise<unknown> {
    try {
      return await this.transport.get(`/qdrant/collections/${collection}/shards`);
    } catch (error) {
      this.logger.error('Failed to list shard keys', { collection, error });
      throw error;
    }
  }

  public async qdrantCreateShardKey(collection: string, shardKey: unknown): Promise<unknown> {
    try {
      return await this.transport.put(`/qdrant/collections/${collection}/shards`, {
        shard_key: shardKey,
      });
    } catch (error) {
      this.logger.error('Failed to create shard key', { collection, error });
      throw error;
    }
  }

  public async qdrantDeleteShardKey(collection: string, shardKey: unknown): Promise<unknown> {
    try {
      return await this.transport.post(`/qdrant/collections/${collection}/shards/delete`, {
        shard_key: shardKey,
      });
    } catch (error) {
      this.logger.error('Failed to delete shard key', { collection, error });
      throw error;
    }
  }

  // -- Cluster -------------------------------------------------------------

  public async qdrantGetClusterStatus(): Promise<unknown> {
    try {
      return await this.transport.get('/qdrant/cluster');
    } catch (error) {
      this.logger.error('Failed to get cluster status', { error });
      throw error;
    }
  }

  public async qdrantClusterRecover(): Promise<unknown> {
    try {
      return await this.transport.post('/qdrant/cluster/recover', {});
    } catch (error) {
      this.logger.error('Failed to recover cluster', { error });
      throw error;
    }
  }

  public async qdrantRemovePeer(peerId: string): Promise<unknown> {
    try {
      return await this.transport.delete(`/qdrant/cluster/peer/${peerId}`);
    } catch (error) {
      this.logger.error('Failed to remove peer', { peerId, error });
      throw error;
    }
  }

  public async qdrantListMetadataKeys(): Promise<unknown> {
    try {
      return await this.transport.get('/qdrant/cluster/metadata/keys');
    } catch (error) {
      this.logger.error('Failed to list metadata keys', { error });
      throw error;
    }
  }

  public async qdrantGetMetadataKey(key: string): Promise<unknown> {
    try {
      return await this.transport.get(`/qdrant/cluster/metadata/keys/${key}`);
    } catch (error) {
      this.logger.error('Failed to get metadata key', { key, error });
      throw error;
    }
  }

  public async qdrantUpdateMetadataKey(key: string, value: unknown): Promise<unknown> {
    try {
      return await this.transport.put(`/qdrant/cluster/metadata/keys/${key}`, { value });
    } catch (error) {
      this.logger.error('Failed to update metadata key', { key, error });
      throw error;
    }
  }

  // -- Advanced query / matrix --------------------------------------------

  public async qdrantQueryPoints(collection: string, request: unknown): Promise<unknown> {
    try {
      return await this.transport.post(
        `/qdrant/collections/${collection}/points/query`,
        request,
      );
    } catch (error) {
      this.logger.error('Failed to query points', { collection, error });
      throw error;
    }
  }

  public async qdrantBatchQueryPoints(collection: string, request: unknown): Promise<unknown> {
    try {
      return await this.transport.post(
        `/qdrant/collections/${collection}/points/query/batch`,
        request,
      );
    } catch (error) {
      this.logger.error('Failed to batch query points', { collection, error });
      throw error;
    }
  }

  public async qdrantQueryPointsGroups(collection: string, request: unknown): Promise<unknown> {
    try {
      return await this.transport.post(
        `/qdrant/collections/${collection}/points/query/groups`,
        request,
      );
    } catch (error) {
      this.logger.error('Failed to query points groups', { collection, error });
      throw error;
    }
  }

  public async qdrantSearchPointsGroups(collection: string, request: unknown): Promise<unknown> {
    try {
      return await this.transport.post(
        `/qdrant/collections/${collection}/points/search/groups`,
        request,
      );
    } catch (error) {
      this.logger.error('Failed to search points groups', { collection, error });
      throw error;
    }
  }

  public async qdrantSearchMatrixPairs(collection: string, request: unknown): Promise<unknown> {
    try {
      return await this.transport.post(
        `/qdrant/collections/${collection}/points/search/matrix/pairs`,
        request,
      );
    } catch (error) {
      this.logger.error('Failed to search matrix pairs', { collection, error });
      throw error;
    }
  }

  public async qdrantSearchMatrixOffsets(
    collection: string,
    request: unknown,
  ): Promise<unknown> {
    try {
      return await this.transport.post(
        `/qdrant/collections/${collection}/points/search/matrix/offsets`,
        request,
      );
    } catch (error) {
      this.logger.error('Failed to search matrix offsets', { collection, error });
      throw error;
    }
  }
}
