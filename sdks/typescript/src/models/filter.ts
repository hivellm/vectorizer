/**
 * Typed Qdrant-compatible filter builder for `deleteByFilter` and
 * `bulkUpdateMetadata`.
 *
 * Both tier-control endpoints accept a `filter` body field whose wire shape
 * is a **Qdrant-style** filter with three optional boolean clauses:
 *
 * ```json
 * {
 *   "must":     [ <condition>, ... ],
 *   "should":   [ <condition>, ... ],
 *   "must_not": [ <condition>, ... ]
 * }
 * ```
 *
 * See `docs/users/api/API_REFERENCE.md § Filter shape` for the full reference
 * with all condition types, error responses, and common mistakes.
 *
 * # Quick start
 *
 * ```ts
 * import { filter } from './models/filter';
 *
 * // Match all vectors where topic == "index":
 * const f = filter.must(filter.eq('topic', 'index'));
 *
 * // Match where tier == "hot" AND score >= 0.8:
 * const f2 = filter.combine({
 *   must: [filter.eq('tier', 'hot'), filter.range('score', { gte: 0.8 })],
 * });
 * ```
 */

// ───────────────────────────────────────── condition types ───────────────────

/**
 * Match sub-object — exact value or multi-value membership check.
 *
 * Wire shapes:
 * - Exact: `{ "value": <any> }`
 * - In:    `{ "any": [<any>...] }`
 */
export interface FilterMatch {
  /** Exact value to match against the payload field. */
  value?: unknown;
  /** Array of values — payload field must equal any one of them. */
  any?: unknown[];
}

/**
 * Numeric range bounds (all optional, combined with AND).
 *
 * Wire shape: `{ "gte": <num>?, "lte": <num>? }`
 */
export interface FilterRange {
  /** Greater than or equal. */
  gte?: number;
  /** Less than or equal. */
  lte?: number;
}

/**
 * A single filter condition attached to a payload field key.
 *
 * Exactly one of `match`, `range`, or `filter` should be set:
 * - `match`  — exact value or membership check
 * - `range`  — numeric bounds check
 * - `filter` — nested compound sub-filter (key is ignored by the server)
 *
 * Wire shape:
 * ```json
 * { "key": "<field>", "match": { "value": <any> } }
 * { "key": "<field>", "range": { "gte": <num>?, "lte": <num>? } }
 * { "filter": <QdrantFilter> }
 * ```
 */
export interface FilterCondition {
  /** Payload field path (dot-separated for nested fields). */
  key: string;
  /** Exact value or membership match. */
  match?: FilterMatch;
  /** Numeric range check. */
  range?: FilterRange;
  /** Nested compound sub-filter. */
  filter?: QdrantFilter;
}

// ───────────────────────────────────────── top-level filter ──────────────────

/**
 * Top-level Qdrant-style filter accepted by `deleteByFilter` and
 * `bulkUpdateMetadata`.
 *
 * All three clause arrays are optional; omit any you don't need. At least
 * one clause with at least one condition must be present — the server rejects
 * an all-absent filter with `400 validation_error` ("filter has no
 * conditions").
 *
 * Wire shape:
 * ```json
 * { "must": [...], "should": [...], "must_not": [...] }
 * ```
 */
export interface QdrantFilter {
  /** All conditions must be true (AND semantics). */
  must?: FilterCondition[];
  /** At least one condition must be true (OR semantics). */
  should?: FilterCondition[];
  /** All conditions must be false (NOT semantics). */
  must_not?: FilterCondition[];
}

// ───────────────────────────────────────── validation ────────────────────────

/**
 * Returns true if the filter has at least one non-empty clause.
 *
 * The server rejects filters where all three clause arrays are absent or
 * empty. Use this before calling `deleteByFilter` / `bulkUpdateMetadata`
 * to surface the error client-side.
 */
export function isNonEmptyFilter(f: QdrantFilter): boolean {
  return (
    (f.must != null && f.must.length > 0) ||
    (f.should != null && f.should.length > 0) ||
    (f.must_not != null && f.must_not.length > 0)
  );
}

// ───────────────────────────────────────── builder helpers ───────────────────

/**
 * Typed filter builder helpers.
 *
 * All builder functions return plain objects that satisfy the corresponding
 * interfaces — no class instantiation required.
 *
 * Example:
 * ```ts
 * import { filter } from './models/filter';
 *
 * const f = filter.combine({
 *   must: [filter.eq('tier', 'hot'), filter.range('score', { gte: 0.8 })],
 *   must_not: [filter.eq('archived', true)],
 * });
 * ```
 */
export const filter = {
  /**
   * Build an exact-match condition.
   *
   * Wire: `{ "key": "<key>", "match": { "value": <value> } }`
   */
  eq(key: string, value: unknown): FilterCondition {
    return { key, match: { value } };
  },

  /**
   * Build a multi-value membership condition ("field IN [...]").
   *
   * Wire: `{ "key": "<key>", "match": { "any": [...] } }`
   */
  in(key: string, any: unknown[]): FilterCondition {
    return { key, match: { any } };
  },

  /**
   * Build a numeric range condition.
   *
   * Wire: `{ "key": "<key>", "range": { "gte": <num>?, "lte": <num>? } }`
   */
  range(key: string, range: FilterRange): FilterCondition {
    return { key, range };
  },

  /**
   * Wrap a sub-filter as a nested condition.
   *
   * Wire: `{ "key": "__nested__", "filter": <QdrantFilter> }`
   */
  nested(f: QdrantFilter): FilterCondition {
    return { key: '__nested__', filter: f };
  },

  /**
   * Build a filter requiring ALL given conditions to be true (AND).
   *
   * Accepts a single condition or a spread of conditions.
   *
   * Wire: `{ "must": [...] }`
   */
  must(...conditions: FilterCondition[]): QdrantFilter {
    return { must: conditions };
  },

  /**
   * Build a filter requiring AT LEAST ONE condition to be true (OR).
   *
   * Wire: `{ "should": [...] }`
   */
  should(...conditions: FilterCondition[]): QdrantFilter {
    return { should: conditions };
  },

  /**
   * Build a filter requiring ALL conditions to be false (NOT).
   *
   * Wire: `{ "must_not": [...] }`
   */
  mustNot(...conditions: FilterCondition[]): QdrantFilter {
    return { must_not: conditions };
  },

  /**
   * Assemble a compound filter from partial clause objects.
   *
   * Use this to combine `must`, `should`, and `must_not` in one call:
   * ```ts
   * filter.combine({
   *   must: [filter.eq('status', 'active')],
   *   must_not: [filter.eq('archived', true)],
   * });
   * ```
   */
  combine(parts: Partial<QdrantFilter>): QdrantFilter {
    return parts as QdrantFilter;
  },
} as const;
