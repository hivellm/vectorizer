/**
 * Discovery pipeline models (phase12).
 *
 * Types for the multi-stage discovery pipeline steps:
 * broad_discovery, semantic_focus, promote_readme,
 * compress_evidence, build_answer_plan, render_llm_prompt.
 */

/** Request for `broadDiscovery` (`POST /discovery/broad_discovery`). */
export interface BroadDiscoveryRequest {
  /** Expanded query strings to search across all collections. */
  queries: string[];
  /** Number of top chunks to retrieve per query (default 50). */
  k?: number;
}

/** Response from `broadDiscovery`. */
export interface BroadDiscoveryResponse {
  /** Retrieved chunk summaries. */
  chunks: Record<string, unknown>[];
  /** Total chunks returned. */
  count: number;
}

/** Request for `semanticFocus` (`POST /discovery/semantic_focus`). */
export interface SemanticFocusRequest {
  /** Target collection name. */
  collection: string;
  /** Queries to focus within the collection. */
  queries: string[];
  /** Number of top results per query (default 15). */
  k?: number;
}

/** Response from `semanticFocus`. */
export interface SemanticFocusResponse {
  /** Retrieved chunk summaries. */
  chunks: Record<string, unknown>[];
  /** Total chunks returned. */
  count: number;
}

/** Request for `promoteReadme` (`POST /discovery/promote_readme`). */
export interface PromoteReadmeRequest {
  /** Chunks to evaluate for README promotion. */
  chunks: Record<string, unknown>[];
}

/** Response from `promoteReadme`. */
export interface PromoteReadmeResponse {
  /** Chunks identified as README-quality. */
  promoted_chunks: Record<string, unknown>[];
  /** Count of promoted chunks. */
  count: number;
}

/** Request for `compressEvidence` (`POST /discovery/compress_evidence`). */
export interface CompressEvidenceRequest {
  /** Input chunks to compress into bullets. */
  chunks: Record<string, unknown>[];
  /** Maximum number of bullets to emit (default 20). */
  max_bullets?: number;
  /** Maximum bullets per source document (default 3). */
  max_per_doc?: number;
}

/** Response from `compressEvidence`. */
export interface CompressEvidenceResponse {
  /** Compressed bullet points. */
  bullets: Record<string, unknown>[];
  /** Count of bullets. */
  count: number;
}

/** Request for `buildAnswerPlan` (`POST /discovery/build_answer_plan`). */
export interface AnswerPlanRequest {
  /** Bullets to organize into a plan. */
  bullets: Record<string, unknown>[];
}

/** Structured answer plan returned by `buildAnswerPlan`. */
export interface AnswerPlan {
  /** Organized sections. */
  sections: Record<string, unknown>[];
  /** Total bullet count across all sections. */
  total_bullets: number;
  /** Source collection names referenced. */
  sources: string[];
}

/** Request for `renderLlmPrompt` (`POST /discovery/render_llm_prompt`). */
export interface RenderPromptRequest {
  /** The answer plan to render. */
  plan: AnswerPlan;
}

/** Response from `renderLlmPrompt`. */
export interface LlmPrompt {
  /** The rendered prompt string. */
  prompt: string;
  /** Byte length of the prompt. */
  length: number;
  /** Rough token estimate (length / 4). */
  estimated_tokens: number;
}
