/**
 * Discovery surface: orchestrated multi-stage retrieval.
 *
 * `discover` is the headline pipeline (filter → score → expand →
 * search → bullet-summarise); the other three methods expose the
 * individual stages so callers can swap or compose them.
 */

import { BaseClient } from './_base';

export class DiscoveryClient extends BaseClient {
  /** End-to-end discovery pipeline with intelligent search + bullet generation. */
  public async discover(params: {
    query: string;
    include_collections?: string[];
    exclude_collections?: string[];
    max_bullets?: number;
    broad_k?: number;
    focus_k?: number;
  }): Promise<unknown> {
    this.logger.debug('Running discovery pipeline', params);
    return this.transport.post('/discover', params);
  }

  /** Pre-filter collections by name patterns. */
  public async filterCollections(params: {
    query: string;
    include?: string[];
    exclude?: string[];
  }): Promise<unknown> {
    this.logger.debug('Filtering collections', params);
    return this.transport.post('/discovery/filter_collections', params);
  }

  /** Rank collections by relevance to a query. */
  public async scoreCollections(params: {
    query: string;
    name_match_weight?: number;
    term_boost_weight?: number;
    signal_boost_weight?: number;
  }): Promise<unknown> {
    this.logger.debug('Scoring collections', params);
    return this.transport.post('/discovery/score_collections', params);
  }

  /** Generate query variations (definition / features / architecture). */
  public async expandQueries(params: {
    query: string;
    max_expansions?: number;
    include_definition?: boolean;
    include_features?: boolean;
    include_architecture?: boolean;
  }): Promise<unknown> {
    this.logger.debug('Expanding queries', params);
    return this.transport.post('/discovery/expand_queries', params);
  }
}
