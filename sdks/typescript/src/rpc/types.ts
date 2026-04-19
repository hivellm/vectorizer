/**
 * VectorizerRPC wire types: `VectorizerValue`, `Request`, `Response`.
 *
 * Wire spec § 2 + § 3: `docs/specs/VECTORIZER_RPC.md`. Mirrors the
 * Rust SDK at `sdks/rust/src/rpc/types.rs` and the Python SDK at
 * `sdks/python/rpc/types.py` byte-for-byte.
 *
 * The `VectorizerValue` tagged union encodes to MessagePack using
 * rmp-serde's externally-tagged enum representation:
 *
 * - Unit variant (`Null`) → bare string `"Null"`.
 * - Newtype variant (`Int(42)`) → single-key map `{"Int": 42}`.
 *
 * Both `Result<T, E>` (used by `Response.result`) and
 * `VectorizerValue` use this same encoding, which means an
 * `Ok(Str("PONG"))` round-trips on the wire as TWO nested
 * single-key maps (`{"Ok": {"Str": "PONG"}}`). The decoder here
 * unwraps both layers; callers see plain JavaScript values.
 */

/** Tag strings — match the Rust enum variant names exactly. */
const NULL = 'Null';
const BOOL = 'Bool';
const INT = 'Int';
const FLOAT = 'Float';
const BYTES = 'Bytes';
const STR = 'Str';
const ARRAY = 'Array';
const MAP = 'Map';

const RESULT_OK = 'Ok';
const RESULT_ERR = 'Err';

/**
 * Discriminated union for the dynamically-typed value that crosses
 * the wire. Use the {@link Value} factory functions (rather than
 * building objects manually) so the on-wire encoding stays consistent.
 *
 * Equality is structural: two values match when both `kind` and
 * `value` match.
 */
export type VectorizerValue =
  | { kind: 'Null'; value: null }
  | { kind: 'Bool'; value: boolean }
  | { kind: 'Int'; value: number | bigint }
  | { kind: 'Float'; value: number }
  | { kind: 'Bytes'; value: Uint8Array }
  | { kind: 'Str'; value: string }
  | { kind: 'Array'; value: VectorizerValue[] }
  | { kind: 'Map'; value: Array<[VectorizerValue, VectorizerValue]> };

/**
 * Factory functions for building {@link VectorizerValue} instances.
 * Aliased shorthands for the verbose object literal form.
 */
export const Value = {
  null_(): VectorizerValue {
    return { kind: 'Null', value: null };
  },
  bool(b: boolean): VectorizerValue {
    return { kind: 'Bool', value: b };
  },
  int(i: number | bigint): VectorizerValue {
    return { kind: 'Int', value: i };
  },
  float(f: number): VectorizerValue {
    return { kind: 'Float', value: f };
  },
  bytes(b: Uint8Array): VectorizerValue {
    return { kind: 'Bytes', value: b };
  },
  str(s: string): VectorizerValue {
    return { kind: 'Str', value: s };
  },
  array(items: VectorizerValue[]): VectorizerValue {
    return { kind: 'Array', value: items };
  },
  map(pairs: Array<[VectorizerValue, VectorizerValue]>): VectorizerValue {
    return { kind: 'Map', value: pairs };
  },
};

/** Convert a {@link VectorizerValue} to its msgpack-ready shape. */
export function valueToMsgpack(v: VectorizerValue): unknown {
  switch (v.kind) {
    case 'Null':
      return NULL;
    case 'Array':
      return { [ARRAY]: v.value.map(valueToMsgpack) };
    case 'Map':
      return {
        [MAP]: v.value.map(([k, val]) => [valueToMsgpack(k), valueToMsgpack(val)]),
      };
    default:
      return { [v.kind]: v.value };
  }
}

/**
 * Decode an externally-tagged msgpack value back to a typed
 * {@link VectorizerValue}. Throws on shape violations.
 */
export function valueFromMsgpack(raw: unknown): VectorizerValue {
  if (typeof raw === 'string') {
    if (raw === NULL) return Value.null_();
    throw new Error(`unknown unit-variant tag: ${JSON.stringify(raw)}`);
  }
  if (raw === null || typeof raw !== 'object') {
    throw new Error(
      `expected externally-tagged object or 'Null', got ${typeof raw}: ${JSON.stringify(raw)}`,
    );
  }
  // msgpack-javascript decodes maps as Map<K,V> by default for non-string keys
  // but as plain objects when keys are strings (which is our case for the tag).
  let entries: Array<[string, unknown]>;
  if (raw instanceof Map) {
    entries = Array.from(raw.entries()).map(([k, v]) => [String(k), v]);
  } else {
    entries = Object.entries(raw as Record<string, unknown>);
  }
  if (entries.length !== 1) {
    throw new Error(
      `externally-tagged value must have exactly one key, got ${entries.length}`,
    );
  }
  const [tag, payload] = entries[0]!;
  switch (tag) {
    case BOOL:
      return Value.bool(payload as boolean);
    case INT:
      return Value.int(payload as number | bigint);
    case FLOAT:
      return Value.float(payload as number);
    case BYTES:
      return Value.bytes(payload as Uint8Array);
    case STR:
      return Value.str(payload as string);
    case ARRAY: {
      const items = payload as unknown[];
      return Value.array(items.map(valueFromMsgpack));
    }
    case MAP: {
      const pairs = payload as Array<[unknown, unknown]>;
      return Value.map(
        pairs.map(([k, v]) => [valueFromMsgpack(k), valueFromMsgpack(v)] as [
          VectorizerValue,
          VectorizerValue,
        ]),
      );
    }
    default:
      throw new Error(`unknown VectorizerValue tag: ${JSON.stringify(tag)}`);
  }
}

/** Borrow the inner string if `v` is a `Str`, else `null`. */
export function asStr(v: VectorizerValue): string | null {
  return v.kind === 'Str' ? v.value : null;
}

/** Borrow the inner integer (as `number`) if `v` is an `Int`, else `null`. */
export function asInt(v: VectorizerValue): number | null {
  if (v.kind !== 'Int') return null;
  return typeof v.value === 'bigint' ? Number(v.value) : v.value;
}

/** Read as `number` if `Float` (or coerce from `Int`), else `null`. */
export function asFloat(v: VectorizerValue): number | null {
  if (v.kind === 'Float') return v.value;
  if (v.kind === 'Int') {
    return typeof v.value === 'bigint' ? Number(v.value) : v.value;
  }
  return null;
}

/** Borrow the boolean if `v` is `Bool`, else `null`. */
export function asBool(v: VectorizerValue): boolean | null {
  return v.kind === 'Bool' ? v.value : null;
}

/** Borrow the array if `v` is `Array`, else `null`. */
export function asArray(v: VectorizerValue): VectorizerValue[] | null {
  return v.kind === 'Array' ? v.value : null;
}

/** Borrow the map pairs if `v` is `Map`, else `null`. */
export function asMap(
  v: VectorizerValue,
): Array<[VectorizerValue, VectorizerValue]> | null {
  return v.kind === 'Map' ? v.value : null;
}

/**
 * Look up a string-keyed map entry. Returns `null` when `v` is not
 * a `Map` or when the key is missing. Workhorse for decoding HELLO
 * responses and other named-field maps coming back from the server.
 */
export function mapGet(v: VectorizerValue, key: string): VectorizerValue | null {
  const pairs = asMap(v);
  if (pairs === null) return null;
  for (const [k, val] of pairs) {
    if (k.kind === 'Str' && k.value === key) return val;
  }
  return null;
}

// ── Wire frames ─────────────────────────────────────────────────────────

/**
 * A request from client to server. Wire spec § 2.
 *
 * Encoded on the wire as a 3-element MessagePack array
 * `[id, command, args]` to match rmp-serde's default struct
 * representation.
 */
export interface Request {
  id: number;
  command: string;
  args: VectorizerValue[];
}

/** Serialise a Request to its on-wire shape (3-element array). */
export function requestToMsgpack(req: Request): unknown[] {
  return [req.id, req.command, req.args.map(valueToMsgpack)];
}

/**
 * A response from server to client. Wire spec § 2.
 *
 * `result` is a discriminated union mirroring Rust's
 * `Result<Value, String>`.
 */
export type ResponseResult =
  | { kind: 'Ok'; value: VectorizerValue }
  | { kind: 'Err'; message: string };

export interface Response {
  id: number;
  result: ResponseResult;
}

/** Build a successful Response. Used by tests/fixtures. */
export function responseOk(id: number, value: VectorizerValue): Response {
  return { id, result: { kind: RESULT_OK, value } };
}

/** Build an error Response. Used by tests/fixtures. */
export function responseErr(id: number, message: string): Response {
  return { id, result: { kind: RESULT_ERR, message } };
}

/** Serialise a Response to its on-wire shape (2-element array). */
export function responseToMsgpack(resp: Response): unknown[] {
  if (resp.result.kind === RESULT_OK) {
    return [resp.id, { [RESULT_OK]: valueToMsgpack(resp.result.value) }];
  }
  return [resp.id, { [RESULT_ERR]: resp.result.message }];
}

/** Decode a Response from its on-wire shape (2-element array). */
export function responseFromMsgpack(raw: unknown): Response {
  if (!Array.isArray(raw) || raw.length !== 2) {
    throw new Error(`Response wire frame must be a 2-element array`);
  }
  const [rid, resultRaw] = raw;
  if (resultRaw === null || typeof resultRaw !== 'object') {
    throw new Error(`Response.result must be a single-key map`);
  }
  let entries: Array<[string, unknown]>;
  if (resultRaw instanceof Map) {
    entries = Array.from(resultRaw.entries()).map(([k, v]) => [String(k), v]);
  } else {
    entries = Object.entries(resultRaw as Record<string, unknown>);
  }
  if (entries.length !== 1) {
    throw new Error(`Response.result must be a single-key map`);
  }
  const [tag, payload] = entries[0]!;
  if (tag === RESULT_OK) {
    return { id: Number(rid), result: { kind: RESULT_OK, value: valueFromMsgpack(payload) } };
  }
  if (tag === RESULT_ERR) {
    if (typeof payload !== 'string') {
      throw new Error(`Err payload must be a string`);
    }
    return { id: Number(rid), result: { kind: RESULT_ERR, message: payload } };
  }
  throw new Error(`unknown Result tag: ${JSON.stringify(tag)}`);
}
