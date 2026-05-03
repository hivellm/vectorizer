/**
 * Extended vector operation models (phase12).
 *
 * Types for insertText (single), listVectors, getVectorByPath,
 * searchByFile, and the associated request/response shapes.
 */

/** Paginated vector listing returned by `GET /collections/{name}/vectors`. */
export interface VectorPage {
  /** Vectors on this page — each entry carries `id`, `vector`, and `payload`. */
  vectors: Record<string, unknown>[];
  /** Total vector count in the collection (unfiltered). */
  total: number;
  /** The effective `limit` applied by the server. */
  limit: number;
  /** The byte-offset applied by the server. */
  offset: number;
  /** Optional human-readable pagination hint from the server. */
  message?: string;
}

/** Request for `searchByFile` (`POST /collections/{name}/search/file`). */
export interface SearchByFileRequest {
  /** File path to search within. */
  file_path: string;
  /** Max results (default 10). */
  limit?: number;
}
