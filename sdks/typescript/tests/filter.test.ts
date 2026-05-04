/**
 * Unit tests for the typed QdrantFilter builder (phase23 §4).
 *
 * Verifies:
 * - Wire JSON produced by each builder helper matches the server's expected shape.
 * - Compound must + must_not produces the correct multi-clause object.
 * - Empty-filter guard (isNonEmptyFilter) rejects filters with no conditions.
 * - Builder ergonomics: filter.combine() assembles multi-clause filters.
 *
 * Wire shapes reference: sdks/rust/src/models/filter.rs
 */

import { describe, it, expect } from 'vitest';
import {
  filter,
  isNonEmptyFilter,
  type QdrantFilter,
  type FilterCondition,
} from '../src/models/filter';

// ---------------------------------------------------------------------------
// Single-condition builders → correct wire JSON
// ---------------------------------------------------------------------------

describe('filter.eq — exact match wire shape', () => {
  it('produces {key, match: {value}} for a string value', () => {
    const cond: FilterCondition = filter.eq('topic', 'index');
    expect(cond.key).toBe('topic');
    expect(cond.match).toBeDefined();
    expect(cond.match!.value).toBe('index');
    expect(cond.match!.any).toBeUndefined();
    expect(cond.range).toBeUndefined();
    expect(cond.filter).toBeUndefined();

    // Serialise to JSON and verify the wire shape
    const json = JSON.parse(JSON.stringify(cond));
    expect(json).toEqual({ key: 'topic', match: { value: 'index' } });
  });

  it('produces the correct wire shape for a numeric value', () => {
    const cond = filter.eq('count', 42);
    const json = JSON.parse(JSON.stringify(cond));
    expect(json).toEqual({ key: 'count', match: { value: 42 } });
  });

  it('produces the correct wire shape for a boolean value', () => {
    const cond = filter.eq('active', true);
    const json = JSON.parse(JSON.stringify(cond));
    expect(json).toEqual({ key: 'active', match: { value: true } });
  });
});

describe('filter.in — membership match wire shape', () => {
  it('produces {key, match: {any: [...]}} for an array', () => {
    const cond = filter.in('status', ['hot', 'warm', 'cold']);
    const json = JSON.parse(JSON.stringify(cond));
    expect(json).toEqual({ key: 'status', match: { any: ['hot', 'warm', 'cold'] } });
  });

  it('handles a single-element array', () => {
    const cond = filter.in('tier', ['hot']);
    const json = JSON.parse(JSON.stringify(cond));
    expect(json).toEqual({ key: 'tier', match: { any: ['hot'] } });
  });
});

describe('filter.range — numeric range wire shape', () => {
  it('produces {key, range: {gte, lte}} when both bounds are set', () => {
    const cond = filter.range('score', { gte: 0.5, lte: 0.9 });
    const json = JSON.parse(JSON.stringify(cond));
    expect(json).toEqual({ key: 'score', range: { gte: 0.5, lte: 0.9 } });
  });

  it('omits absent bound keys (gte-only)', () => {
    const cond = filter.range('score', { gte: 0.8 });
    const json = JSON.parse(JSON.stringify(cond));
    expect(json).toEqual({ key: 'score', range: { gte: 0.8 } });
    expect(json.range.lte).toBeUndefined();
  });

  it('omits absent bound keys (lte-only)', () => {
    const cond = filter.range('age', { lte: 100 });
    const json = JSON.parse(JSON.stringify(cond));
    expect(json).toEqual({ key: 'age', range: { lte: 100 } });
    expect(json.range.gte).toBeUndefined();
  });
});

// ---------------------------------------------------------------------------
// Compound filter builders
// ---------------------------------------------------------------------------

describe('filter.must — AND compound filter', () => {
  it('wraps conditions in a must array', () => {
    const f: QdrantFilter = filter.must(filter.eq('tier', 'hot'));
    const json = JSON.parse(JSON.stringify(f));
    expect(json.must).toHaveLength(1);
    expect(json.must[0]).toEqual({ key: 'tier', match: { value: 'hot' } });
    expect(json.should).toBeUndefined();
    expect(json.must_not).toBeUndefined();
  });

  it('accepts multiple conditions as spread args', () => {
    const f = filter.must(filter.eq('tier', 'hot'), filter.range('score', { gte: 0.8 }));
    const json = JSON.parse(JSON.stringify(f));
    expect(json.must).toHaveLength(2);
  });
});

describe('filter.mustNot — NOT compound filter', () => {
  it('wraps conditions in a must_not array', () => {
    const f = filter.mustNot(filter.eq('archived', true));
    const json = JSON.parse(JSON.stringify(f));
    expect(json.must_not).toHaveLength(1);
    expect(json.must).toBeUndefined();
    expect(json.should).toBeUndefined();
  });
});

describe('compound must + must_not — correct wire JSON', () => {
  it('produces both clauses in one filter object', () => {
    const f: QdrantFilter = filter.combine({
      must: [filter.eq('tier', 'hot'), filter.range('score', { gte: 0.8 })],
      must_not: [filter.eq('archived', true)],
    });

    const json = JSON.parse(JSON.stringify(f));
    expect(json.must).toHaveLength(2);
    expect(json.must[0]).toEqual({ key: 'tier', match: { value: 'hot' } });
    expect(json.must[1]).toEqual({ key: 'score', range: { gte: 0.8 } });
    expect(json.must_not).toHaveLength(1);
    expect(json.must_not[0]).toEqual({ key: 'archived', match: { value: true } });
    expect(json.should).toBeUndefined();
  });
});

// ---------------------------------------------------------------------------
// Empty-filter guard
// ---------------------------------------------------------------------------

describe('isNonEmptyFilter — rejects empty filters', () => {
  it('returns false for an object with no clauses', () => {
    expect(isNonEmptyFilter({})).toBe(false);
  });

  it('returns false when all clause arrays are empty', () => {
    expect(isNonEmptyFilter({ must: [], should: [], must_not: [] })).toBe(false);
  });

  it('returns false when arrays are null-ish via cast', () => {
    // Undefined arrays are treated as absent
    expect(isNonEmptyFilter({ must: undefined, should: undefined })).toBe(false);
  });

  it('returns true when must has at least one condition', () => {
    expect(isNonEmptyFilter({ must: [filter.eq('k', 'v')] })).toBe(true);
  });

  it('returns true when should has at least one condition', () => {
    expect(isNonEmptyFilter({ should: [filter.eq('k', 'v')] })).toBe(true);
  });

  it('returns true when must_not has at least one condition', () => {
    expect(isNonEmptyFilter({ must_not: [filter.eq('k', 'v')] })).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// Builder ergonomics — chained combine
// ---------------------------------------------------------------------------

describe('filter.combine — multi-clause assembly', () => {
  it('assembles must + should + must_not in one call', () => {
    const f = filter.combine({
      must: [filter.eq('type', 'article')],
      should: [filter.in('tag', ['ai', 'ml'])],
      must_not: [filter.eq('draft', true)],
    });

    const json = JSON.parse(JSON.stringify(f));
    expect(json.must).toHaveLength(1);
    expect(json.should).toHaveLength(1);
    expect(json.must_not).toHaveLength(1);
    expect(json.should[0]).toEqual({ key: 'tag', match: { any: ['ai', 'ml'] } });
  });
});

// ---------------------------------------------------------------------------
// Nested sub-filter
// ---------------------------------------------------------------------------

describe('filter.nested — nested sub-filter condition', () => {
  it('produces a condition with a filter sub-object', () => {
    const inner = filter.must(filter.eq('inner_key', 'value'));
    const cond = filter.nested(inner);
    const json = JSON.parse(JSON.stringify(cond));
    expect(json.key).toBe('__nested__');
    expect(json.filter).toBeDefined();
    expect(json.filter.must[0]).toEqual({ key: 'inner_key', match: { value: 'value' } });
  });
});
