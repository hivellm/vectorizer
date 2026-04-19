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
}
