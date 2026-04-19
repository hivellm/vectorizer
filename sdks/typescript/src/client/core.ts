/**
 * Server-status surface: health, stats, embeddings.
 *
 * Lives in its own module because it doesn't fit any of the
 * domain-specific surfaces (collections / vectors / search / ...).
 */

import { BaseClient } from './_base';
import {
  DatabaseStats,
  EmbeddingRequest,
  EmbeddingResponse,
  validateDatabaseStats,
  validateEmbeddingRequest,
  validateEmbeddingResponse,
} from '../models';

export class CoreClient extends BaseClient {
  /** Liveness probe. */
  public async healthCheck(): Promise<{ status: string; timestamp: string }> {
    try {
      const response = await this.transport.get<{ status: string; timestamp: string }>('/health');
      this.logger.debug('Health check successful', response);
      return response;
    } catch (error) {
      this.logger.error('Health check failed', error);
      throw error;
    }
  }

  /** Aggregate database statistics. */
  public async getDatabaseStats(): Promise<DatabaseStats> {
    try {
      const response = await this.transport.get<DatabaseStats>('/stats');
      validateDatabaseStats(response);
      this.logger.debug('Database stats retrieved', response);
      return response;
    } catch (error) {
      this.logger.error('Failed to get database stats', error);
      throw error;
    }
  }

  /** Generate embeddings for a single text payload. */
  public async embedText(request: EmbeddingRequest): Promise<EmbeddingResponse> {
    try {
      validateEmbeddingRequest(request);
      const response = await this.transport.post<EmbeddingResponse>('/embed', request);
      validateEmbeddingResponse(response);
      this.logger.debug('Text embedding generated', { text: request.text, model: response.model });
      return response;
    } catch (error) {
      this.logger.error('Failed to generate embedding', { request, error });
      throw error;
    }
  }
}
