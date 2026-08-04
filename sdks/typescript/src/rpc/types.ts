/**
 * VectorizerRPC wire types: `VectorizerValue`, `Request`, `Response`.
 *
 * Wire spec § 2 + § 3: `docs/specs/VECTORIZER_RPC.md`. The types are
 * Thunder's (`@hivehub/thunder`) — the family's shared binary RPC package,
 * whose Rust twin `vectorizer-server` runs — so the SDK and the server
 * cannot disagree on the wire. The on-wire encoding is unchanged (v1,
 * frozen): 4-byte little-endian length prefix + MessagePack body, with the
 * externally-tagged value/`Result` layout rmp-serde emits.
 *
 * `VectorizerValue` is an alias for Thunder's `Value`, and the `Value`
 * factory plus the `asX` / `mapGet` accessors below keep this SDK's
 * `null`-returning shape (Thunder's return `undefined`) so the typed
 * command wrappers read exactly as before.
 */

import {
  Value as ThunderValue,
  type Request as ThunderRequest,
  type ResponseResult as ThunderResponseResult,
  type Value as ThunderValueType,
} from '@hivehub/thunder';

/**
 * A response from server to client (wire spec § 2), plus its `ok` / `err`
 * constructors — Thunder's, re-exported so fixtures and test servers build
 * frames the same way the real server does.
 */
export { Response } from '@hivehub/thunder';

/**
 * The dynamically-typed value that crosses the wire — Thunder's
 * eight-variant model (`Null | Bool | Int | Float | Bytes | Str | Array |
 * Map`). Use the {@link Value} factory functions rather than building
 * objects by hand so the on-wire encoding stays consistent.
 *
 * `Int` carries a `bigint` (JavaScript numbers cannot hold the full i64
 * range); the factory also accepts safe-range `number`s.
 */
export type VectorizerValue = ThunderValueType;

/**
 * Factory functions for building {@link VectorizerValue} instances.
 * `null_` keeps its trailing underscore (`null` is a reserved word in the
 * positions this object is used from).
 */
export const Value = {
  null_(): VectorizerValue {
    return ThunderValue.null();
  },
  bool(b: boolean): VectorizerValue {
    return ThunderValue.bool(b);
  },
  int(i: number | bigint): VectorizerValue {
    return ThunderValue.int(i);
  },
  float(f: number): VectorizerValue {
    return ThunderValue.float(f);
  },
  bytes(b: Uint8Array): VectorizerValue {
    return ThunderValue.bytes(b);
  },
  str(s: string): VectorizerValue {
    return ThunderValue.str(s);
  },
  array(items: VectorizerValue[]): VectorizerValue {
    return ThunderValue.array(items);
  },
  map(pairs: Array<[VectorizerValue, VectorizerValue]>): VectorizerValue {
    return ThunderValue.map(pairs);
  },
};

/** Borrow the inner string if `v` is a `Str`, else `null`. */
export function asStr(v: VectorizerValue): string | null {
  return ThunderValue.asStr(v) ?? null;
}

/**
 * Borrow the inner integer if `v` is an `Int`, else `null`. Narrowed to
 * `number` for the SDK's decode helpers — the command catalog carries
 * counts and dimensions, not full-range i64s.
 */
export function asInt(v: VectorizerValue): number | null {
  const i = ThunderValue.asInt(v);
  return i === undefined ? null : Number(i);
}

/** Read as `number` if `Float` (or coerce from `Int`), else `null`. */
export function asFloat(v: VectorizerValue): number | null {
  return ThunderValue.asFloat(v) ?? null;
}

/** Borrow the boolean if `v` is `Bool`, else `null`. */
export function asBool(v: VectorizerValue): boolean | null {
  return ThunderValue.asBool(v) ?? null;
}

/** Borrow the array if `v` is `Array`, else `null`. */
export function asArray(v: VectorizerValue): VectorizerValue[] | null {
  return ThunderValue.asArray(v) ?? null;
}

/** Borrow the map pairs if `v` is `Map`, else `null`. */
export function asMap(
  v: VectorizerValue,
): Array<[VectorizerValue, VectorizerValue]> | null {
  return ThunderValue.asMap(v) ?? null;
}

/**
 * Look up a string-keyed map entry. Returns `null` when `v` is not
 * a `Map` or when the key is missing. Workhorse for decoding HELLO
 * responses and other named-field maps coming back from the server.
 */
export function mapGet(v: VectorizerValue, key: string): VectorizerValue | null {
  return ThunderValue.mapGet(v, key) ?? null;
}

// ── Wire frames ─────────────────────────────────────────────────────────

/**
 * A request from client to server (wire spec § 2), encoded as a 3-element
 * MessagePack array `[id, command, args]`.
 */
export type Request = ThunderRequest;

/**
 * `Response.result` — `{ ok: Value }` or `{ err: string }`, mirroring
 * Rust's `Result<Value, String>`.
 */
export type ResponseResult = ThunderResponseResult;
