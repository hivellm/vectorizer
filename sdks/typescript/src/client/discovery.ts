/**
 * Discovery surface: orchestrated multi-stage retrieval.
 *
 * `discover` is the headline pipeline (filter → score → expand →
 * search → bullet-summarise); the other methods expose individual
 * stages and the phase12 pipeline steps:
 * broadDiscovery, semanticFocus, promoteReadme, compressEvidence,
 * buildAnswerPlan, renderLlmPrompt.
 */

import { BaseClient } from './_base';
import type {
  AnswerPlan,
  AnswerPlanRequest,
  BroadDiscoveryRequest,
  BroadDiscoveryResponse,
  CompressEvidenceRequest,
  CompressEvidenceResponse,
  LlmPrompt,
  PromoteReadmeRequest,
  PromoteReadmeResponse,
  RenderPromptRequest,
  SemanticFocusRequest,
  SemanticFocusResponse,
} from '../models';

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

  /**
   * Broad multi-query search across all collections.
   * Calls `POST /discovery/broad_discovery` with `{queries, k?}`.
   */
  public async broadDiscovery(request: BroadDiscoveryRequest): Promise<BroadDiscoveryResponse> {
    this.logger.debug('Running broad discovery', { queryCount: request.queries.length });
    return this.transport.post<BroadDiscoveryResponse>('/discovery/broad_discovery', {
      queries: request.queries,
      k: request.k ?? 50,
    });
  }

  /**
   * Focused semantic search within a single collection.
   * Calls `POST /discovery/semantic_focus` with `{collection, queries, k?}`.
   */
  public async semanticFocus(request: SemanticFocusRequest): Promise<SemanticFocusResponse> {
    this.logger.debug('Running semantic focus', {
      collection: request.collection,
      queryCount: request.queries.length,
    });
    return this.transport.post<SemanticFocusResponse>('/discovery/semantic_focus', {
      collection: request.collection,
      queries: request.queries,
      k: request.k ?? 15,
    });
  }

  /**
   * Promote README-quality chunks to the top of a result set.
   * Calls `POST /discovery/promote_readme` with `{chunks}`.
   */
  public async promoteReadme(request: PromoteReadmeRequest): Promise<PromoteReadmeResponse> {
    this.logger.debug('Promoting README chunks', { chunkCount: request.chunks.length });
    return this.transport.post<PromoteReadmeResponse>('/discovery/promote_readme', {
      chunks: request.chunks,
    });
  }

  /**
   * Compress a chunk set into a concise bullet list.
   * Calls `POST /discovery/compress_evidence` with `{chunks, max_bullets?, max_per_doc?}`.
   */
  public async compressEvidence(
    request: CompressEvidenceRequest,
  ): Promise<CompressEvidenceResponse> {
    this.logger.debug('Compressing evidence', { chunkCount: request.chunks.length });
    const payload: Record<string, unknown> = { chunks: request.chunks };
    if (request.max_bullets !== undefined) payload['max_bullets'] = request.max_bullets;
    if (request.max_per_doc !== undefined) payload['max_per_doc'] = request.max_per_doc;
    return this.transport.post<CompressEvidenceResponse>('/discovery/compress_evidence', payload);
  }

  /**
   * Organise bullets into a structured answer plan.
   * Calls `POST /discovery/build_answer_plan` with `{bullets}`.
   */
  public async buildAnswerPlan(request: AnswerPlanRequest): Promise<AnswerPlan> {
    this.logger.debug('Building answer plan', { bulletCount: request.bullets.length });
    return this.transport.post<AnswerPlan>('/discovery/build_answer_plan', {
      bullets: request.bullets,
    });
  }

  /**
   * Render an answer plan into a final LLM prompt string.
   * Calls `POST /discovery/render_llm_prompt` with `{plan}`.
   */
  public async renderLlmPrompt(request: RenderPromptRequest): Promise<LlmPrompt> {
    this.logger.debug('Rendering LLM prompt');
    return this.transport.post<LlmPrompt>('/discovery/render_llm_prompt', {
      plan: request.plan,
    });
  }
}
