/**
 * Summarization models for the Hive Vectorizer TypeScript SDK.
 * 
 * This module contains all the data models used for text and context summarization.
 */

export interface SummarizeTextRequest {
  /** Text to summarize */
  text: string;
  /** Summarization method (extractive, keyword, sentence, abstractive) */
  method?: string;
  /** Maximum summary length (optional) */
  max_length?: number;
  /** Compression ratio (optional) */
  compression_ratio?: number;
  /** Language code (optional) */
  language?: string;
  /** Additional metadata (optional) */
  metadata?: Record<string, string>;
}

export interface SummarizeTextResponse {
  /** Summary ID */
  summary_id: string;
  /** Original text */
  original_text: string;
  /** Generated summary */
  summary: string;
  /** Method used */
  method: string;
  /** Original text length */
  original_length: number;
  /** Summary length */
  summary_length: number;
  /** Compression ratio */
  compression_ratio: number;
  /** Language */
  language: string;
  /** Status */
  status: string;
  /** Message */
  message: string;
  /** Metadata */
  metadata: Record<string, string>;
}

export interface SummarizeContextRequest {
  /** Context to summarize */
  context: string;
  /** Summarization method (extractive, keyword, sentence, abstractive) */
  method?: string;
  /** Maximum summary length (optional) */
  max_length?: number;
  /** Compression ratio (optional) */
  compression_ratio?: number;
  /** Language code (optional) */
  language?: string;
  /** Additional metadata (optional) */
  metadata?: Record<string, string>;
}

export interface SummarizeContextResponse {
  /** Summary ID */
  summary_id: string;
  /** Original context */
  original_context: string;
  /** Generated summary */
  summary: string;
  /** Method used */
  method: string;
  /** Original context length */
  original_length: number;
  /** Summary length */
  summary_length: number;
  /** Compression ratio */
  compression_ratio: number;
  /** Language */
  language: string;
  /** Status */
  status: string;
  /** Message */
  message: string;
  /** Metadata */
  metadata: Record<string, string>;
}

export interface GetSummaryResponse {
  /** Summary ID */
  summary_id: string;
  /** Original text */
  original_text: string;
  /** Generated summary */
  summary: string;
  /** Method used */
  method: string;
  /** Original text length */
  original_length: number;
  /** Summary length */
  summary_length: number;
  /** Compression ratio */
  compression_ratio: number;
  /** Language */
  language: string;
  /** Creation timestamp */
  created_at: string;
  /** Metadata */
  metadata: Record<string, string>;
  /** Status */
  status: string;
}

export interface SummaryInfo {
  /** Summary ID */
  summary_id: string;
  /** Method used */
  method: string;
  /** Language */
  language: string;
  /** Original text length */
  original_length: number;
  /** Summary length */
  summary_length: number;
  /** Compression ratio */
  compression_ratio: number;
  /** Creation timestamp */
  created_at: string;
  /** Metadata */
  metadata: Record<string, string>;
}

export interface ListSummariesResponse {
  /** List of summaries */
  summaries: SummaryInfo[];
  /** Total count */
  total_count: number;
  /** Status */
  status: string;
}

export interface ListSummariesQuery {
  /** Filter by method (optional) */
  method?: string;
  /** Filter by language (optional) */
  language?: string;
  /** Maximum number of summaries to return (optional) */
  limit?: number;
  /** Offset for pagination (optional) */
  offset?: number;
}
