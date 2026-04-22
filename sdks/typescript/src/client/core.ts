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

/**
 * Response from `POST /auth/login`. Full shape includes `token_type`,
 * `expires_in`, and `user` — this typing keeps the minimum every
 * caller needs (`accessToken`) while leaving the rest available as
 * optional fields for those that want the expiry.
 */
export interface LoginResponse {
  accessToken: string;
  tokenType?: string;
  expiresIn?: number;
  user?: { userId: string; username: string; roles: string[] };
}

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

  /**
   * Exchange `(username, password)` for a JWT via `POST /auth/login`.
   * The returned `accessToken` is NOT stored on `this` — to use it
   * for subsequent requests, construct a new client with
   * `apiKey: token.accessToken`. The HTTP transport sniffs the
   * three-segment JWT shape and sends it as `Authorization: Bearer …`
   * rather than `X-API-Key`.
   *
   * When the server runs with `auth.enabled: false` this endpoint
   * returns 404 — no login needed against a no-auth dev server.
   */
  public async login(username: string, password: string): Promise<LoginResponse> {
    try {
      const raw = await this.transport.post<{
        access_token: string;
        token_type?: string;
        expires_in?: number;
        user?: { user_id: string; username: string; roles: string[] };
      }>('/auth/login', { username, password });
      // `exactOptionalPropertyTypes` in this project rejects explicit
      // `undefined` on optional fields — build the response shape
      // conditionally so each optional is either present with a
      // defined value or omitted entirely.
      const response: LoginResponse = { accessToken: raw.access_token };
      if (raw.token_type !== undefined) {
        response.tokenType = raw.token_type;
      }
      if (raw.expires_in !== undefined) {
        response.expiresIn = raw.expires_in;
      }
      if (raw.user !== undefined) {
        response.user = {
          userId: raw.user.user_id,
          username: raw.user.username,
          roles: raw.user.roles,
        };
      }
      return response;
    } catch (error) {
      this.logger.error('Login failed', { username, error });
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
