/**
 * Vector and batch operations.
 *
 * `insertVectors` writes through the Qdrant-compatible upsert endpoint
 * to share the optimised batch path with the Qdrant surface; the
 * remaining single-vector operations use the dedicated REST routes.
 */

import { BaseClient } from './_base';
import {
  BatchDeleteRequest,
  BatchInsertRequest,
  BatchResponse,
  BatchSearchRequest,
  BatchSearchResponse,
  BatchUpdateRequest,
  CreateVectorRequest,
  ReadOptions,
  UpdateVectorRequest,
  Vector,
  validateCreateVectorRequest,
  validateVector,
} from '../models';

export class VectorsClient extends BaseClient {
  /** Insert vectors via the Qdrant-compatible point upsert. (Master-only.) */
  public async insertVectors(
    collectionName: string,
    vectors: CreateVectorRequest[],
    publicKey?: string,
  ): Promise<{ inserted: number }> {
    try {
      vectors.forEach(validateCreateVectorRequest);
      const transport = this.getWriteTransport();
      const points = vectors.map((v, idx) => ({
        id: v.id ?? `${Date.now()}-${idx}`,
        vector: v.data,
        payload: v.metadata ?? {},
      }));
      const payload: Record<string, unknown> = { points };
      const effectivePublicKey = publicKey || vectors.find((v) => v.publicKey)?.publicKey;
      if (effectivePublicKey) {
        payload['public_key'] = effectivePublicKey;
      }
      await transport.put(`/qdrant/collections/${collectionName}/points`, payload);
      this.logger.info('Vectors inserted', {
        collectionName,
        count: vectors.length,
        encrypted: !!effectivePublicKey,
      });
      return { inserted: vectors.length };
    } catch (error) {
      this.logger.error('Failed to insert vectors', {
        collectionName,
        count: vectors.length,
        error,
      });
      throw error;
    }
  }

  /** Fetch a single vector by ID. */
  public async getVector(
    collectionName: string,
    vectorId: string,
    options?: ReadOptions,
  ): Promise<Vector> {
    try {
      const transport = this.getReadTransport(options);
      const response = await transport.get<Vector>(
        `/collections/${collectionName}/vectors/${vectorId}`,
      );
      validateVector(response);
      this.logger.debug('Vector retrieved', { collectionName, vectorId });
      return response;
    } catch (error) {
      this.logger.error('Failed to get vector', { collectionName, vectorId, error });
      throw error;
    }
  }

  /** Update a vector's payload or data. (Master-only.) */
  public async updateVector(
    collectionName: string,
    vectorId: string,
    request: UpdateVectorRequest,
  ): Promise<Vector> {
    try {
      const transport = this.getWriteTransport();
      const payload: Record<string, unknown> = { ...request };
      if (request.publicKey) {
        payload['public_key'] = request.publicKey;
        delete payload['publicKey'];
      }
      const response = await transport.put<Vector>(
        `/collections/${collectionName}/vectors/${vectorId}`,
        payload,
      );
      validateVector(response);
      this.logger.info('Vector updated', {
        collectionName,
        vectorId,
        encrypted: !!request.publicKey,
      });
      return response;
    } catch (error) {
      this.logger.error('Failed to update vector', { collectionName, vectorId, request, error });
      throw error;
    }
  }

  /** Delete a single vector. (Master-only.) */
  public async deleteVector(collectionName: string, vectorId: string): Promise<void> {
    try {
      const transport = this.getWriteTransport();
      await transport.delete(`/collections/${collectionName}/vectors/${vectorId}`);
      this.logger.info('Vector deleted', { collectionName, vectorId });
    } catch (error) {
      this.logger.error('Failed to delete vector', { collectionName, vectorId, error });
      throw error;
    }
  }

  /** Bulk-delete vectors by ID. (Master-only.) */
  public async deleteVectors(
    collectionName: string,
    vectorIds: string[],
  ): Promise<{ deleted: number }> {
    try {
      const transport = this.getWriteTransport();
      const response = await transport.post<{ deleted: number }>(
        `/collections/${collectionName}/vectors/delete`,
        { vector_ids: vectorIds },
      );
      this.logger.info('Vectors deleted', { collectionName, count: vectorIds.length });
      return response;
    } catch (error) {
      this.logger.error('Failed to delete vectors', {
        collectionName,
        count: vectorIds.length,
        error,
      });
      throw error;
    }
  }

  /** Batch-insert pre-tokenised text payloads. (Master-only.) */
  public async batchInsertTexts(
    collection: string,
    request: BatchInsertRequest,
  ): Promise<BatchResponse> {
    this.logger.debug('Batch inserting texts', { collection, count: request.texts.length });

    try {
      const response = await this.transport.post<BatchResponse>('/batch_insert', request);
      this.logger.info('Batch insert completed', {
        collection,
        successful: response.successful_operations,
        failed: response.failed_operations,
        duration: response.duration_ms,
      });
      return response;
    } catch (error) {
      this.logger.error('Batch insert failed', { collection, error });
      throw error;
    }
  }

  /** Batch search with multiple queries against one collection. */
  public async batchSearchVectors(
    collection: string,
    request: BatchSearchRequest,
  ): Promise<BatchSearchResponse> {
    this.logger.debug('Batch searching vectors', {
      collection,
      queries: request.queries.length,
    });

    try {
      const response = await this.transport.post<BatchSearchResponse>('/batch_search', request);
      this.logger.info('Batch search completed', {
        collection,
        successful: response.successful_queries,
        failed: response.failed_queries,
        duration: response.duration_ms,
      });
      return response;
    } catch (error) {
      this.logger.error('Batch search failed', { collection, error });
      throw error;
    }
  }

  /** Batch update vectors by ID. (Master-only.) */
  public async batchUpdateVectors(
    collection: string,
    request: BatchUpdateRequest,
  ): Promise<BatchResponse> {
    this.logger.debug('Batch updating vectors', { collection, count: request.updates.length });

    try {
      const response = await this.transport.post<BatchResponse>('/batch_update', request);
      this.logger.info('Batch update completed', {
        collection,
        successful: response.successful_operations,
        failed: response.failed_operations,
        duration: response.duration_ms,
      });
      return response;
    } catch (error) {
      this.logger.error('Batch update failed', { collection, error });
      throw error;
    }
  }

  /** Batch delete vectors by ID. (Master-only.) */
  public async batchDeleteVectors(
    collection: string,
    request: BatchDeleteRequest,
  ): Promise<BatchResponse> {
    this.logger.debug('Batch deleting vectors', {
      collection,
      count: request.vector_ids.length,
    });

    try {
      const response = await this.transport.post<BatchResponse>('/batch_delete', request);
      this.logger.info('Batch delete completed', {
        collection,
        successful: response.successful_operations,
        failed: response.failed_operations,
        duration: response.duration_ms,
      });
      return response;
    } catch (error) {
      this.logger.error('Batch delete failed', { collection, error });
      throw error;
    }
  }
}
