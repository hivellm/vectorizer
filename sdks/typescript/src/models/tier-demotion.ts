/**
 * Tier-demotion API report types (issue #265).
 *
 * Mirrors the server contract for `POST /batch_delete` and the new
 * `POST /collections/{src}/vectors/move` endpoint. See
 * `docs/specs/api/REST_API_REFERENCE.md` and the Rust SDK's
 * `models::DeleteReport` / `models::MoveReport` for canonical docs.
 */

/** Per-vector outcome status for delete and move operations. */
export type VectorOpStatus =
  | 'ok'
  | 'missing_in_src'
  | 'dst_insert_failed'
  | 'src_delete_failed'
  | 'error';

/** One row in [[DeleteReport.results]] / [[MoveReport.results]]. */
export interface VectorOpResult {
  /** Vector id. May be `null` when the request had a non-string entry. */
  id: string | null;
  /** One of the [[VectorOpStatus]] values. */
  status: VectorOpStatus;
  /** Server-side error message, set when `status !== 'ok'`. */
  error?: string;
  /** Index of this entry in the request's `ids` array (delete only). */
  index?: number;
}

/**
 * Aggregate outcome of a `deleteVectors` call against
 * `POST /batch_delete`. Per-id failures (e.g. not-found) populate
 * `results` without aborting the batch.
 */
export interface DeleteReport {
  /** Source collection name, echoed by the server. */
  collection: string;
  /** Total ids the request asked to delete. */
  count: number;
  /** Successfully deleted ids. */
  deleted: number;
  /** Ids that failed (missing or backend error). */
  failed: number;
  /** Per-id outcomes, in request order. */
  results: VectorOpResult[];
}

/**
 * Aggregate outcome of a `moveToCollection` call against
 * `POST /collections/{src}/vectors/move`.
 *
 * Server invariant: vectors are inserted into `dst` BEFORE being
 * deleted from `src`. A mid-batch failure leaves a recoverable
 * duplicate, never data loss. Per-id failures populate `results`
 * without aborting the batch — operators chasing tier-demotion sweeps
 * want partial progress, not abort-on-first-error.
 */
export interface MoveReport {
  /** Source collection name. */
  src: string;
  /** Destination collection name. */
  dst: string;
  /** Total ids the request asked to move. */
  requested: number;
  /** Ids that fully moved (insert + delete both succeeded). */
  moved: number;
  /** Ids that failed at any step. */
  failed: number;
  /** Per-id outcomes, in request order. */
  results: VectorOpResult[];
}
