/**
 * Codec + types unit tests.
 *
 * Covers two distinct invariants:
 *
 * 1. **Wire-spec golden vectors** — bytes produced by encoding a
 *    `Request` / `Response` exactly match the hex dumps in
 *    `docs/specs/VECTORIZER_RPC.md` § 11. If these break, the TS SDK
 *    can no longer talk to a Rust server.
 *
 * 2. **Round-trip** for every `VectorizerValue` variant: encode →
 *    decode → equal-value-back.
 */

import { describe, expect, test } from 'vitest';

import {
  FrameTooLargeError,
  MAX_BODY_SIZE,
  decodeBody,
  encodeFrame,
} from '../../src/rpc/codec';
import {
  Value,
  VectorizerValue,
  responseFromMsgpack,
  responseToMsgpack,
  responseOk,
  responseErr,
  requestToMsgpack,
  valueFromMsgpack,
  valueToMsgpack,
  asArray,
  asInt,
  asStr,
} from '../../src/rpc/types';

describe('wire-spec golden vectors', () => {
  test('Request{id=1, command="PING", args=[]} matches spec', () => {
    // Spec § 11: 08 00 00 00  93  01  a4 50 49 4e 47  90
    const frame = encodeFrame(requestToMsgpack({ id: 1, command: 'PING', args: [] }));
    const expected = Buffer.concat([
      Buffer.from('08000000', 'hex'),
      Buffer.from('93', 'hex'),
      Buffer.from('01', 'hex'),
      Buffer.from('a4', 'hex'),
      Buffer.from('PING'),
      Buffer.from('90', 'hex'),
    ]);
    expect(frame.equals(expected)).toBe(true);
  });

  test('Response{id=1, result=Ok(Str("PONG"))} matches spec', () => {
    // Spec § 11: 10 00 00 00  92  01  81 a2 4f 6b  81 a3 53 74 72 a4 50 4f 4e 47
    const frame = encodeFrame(responseToMsgpack(responseOk(1, Value.str('PONG'))));
    const expected = Buffer.concat([
      Buffer.from('10000000', 'hex'),
      Buffer.from('92', 'hex'),
      Buffer.from('01', 'hex'),
      Buffer.from('81', 'hex'),
      Buffer.from('a2', 'hex'),
      Buffer.from('Ok'),
      Buffer.from('81', 'hex'),
      Buffer.from('a3', 'hex'),
      Buffer.from('Str'),
      Buffer.from('a4', 'hex'),
      Buffer.from('PONG'),
    ]);
    expect(frame.equals(expected)).toBe(true);
  });
});

describe('VectorizerValue round-trip', () => {
  const cases: Array<[string, VectorizerValue]> = [
    ['Null', Value.null_()],
    ['Bool(true)', Value.bool(true)],
    ['Bool(false)', Value.bool(false)],
    ['Int(0)', Value.int(0)],
    ['Int(small negative)', Value.int(-42)],
    ['Float(1.5)', Value.float(1.5)],
    ['Float(-3.14)', Value.float(-3.14)],
    ['Bytes', Value.bytes(new Uint8Array([0, 1, 2, 255]))],
    ['Str empty', Value.str('')],
    ['Str ascii', Value.str('hello')],
    ['Str unicode', Value.str('ünïcödé')],
    [
      'Array',
      Value.array([Value.int(1), Value.str('two')]),
    ],
    [
      'Map nested',
      Value.map([
        [Value.str('k'), Value.int(99)],
        [Value.str('nested'), Value.array([Value.bool(true)])],
      ]),
    ],
  ];

  test.each(cases)('%s round-trips through encode/decode', (_label, value) => {
    const frame = encodeFrame(valueToMsgpack(value));
    const body = frame.subarray(4);
    const decoded = valueFromMsgpack(decodeBody(body));
    // For Bytes, Uint8Array equality requires deep-compare.
    if (value.kind === 'Bytes') {
      expect(decoded.kind).toBe('Bytes');
      const got = decoded.value as Uint8Array;
      expect(Array.from(got)).toEqual(Array.from(value.value));
    } else {
      expect(decoded).toEqual(value);
    }
  });

  test('Response.Err round-trips', () => {
    const resp = responseErr(42, 'something went wrong');
    const frame = encodeFrame(responseToMsgpack(resp));
    const decoded = responseFromMsgpack(decodeBody(frame.subarray(4)));
    expect(decoded.id).toBe(42);
    expect(decoded.result).toEqual({ kind: 'Err', message: 'something went wrong' });
  });

  test('Request with mixed args round-trips at the body level', () => {
    const req = {
      id: 99,
      command: 'search.basic',
      args: [Value.str('alpha-docs'), Value.str('q'), Value.int(10), Value.float(0.5)],
    };
    const frame = encodeFrame(requestToMsgpack(req));
    const body = decodeBody(frame.subarray(4)) as unknown[];
    expect(body[0]).toBe(99);
    expect(body[1]).toBe('search.basic');
    const args = body[2] as unknown[];
    expect(args).toHaveLength(4);
    expect(args[0]).toEqual({ Str: 'alpha-docs' });
    expect(args[2]).toEqual({ Int: 10 });
  });
});

describe('frame limits', () => {
  test('encode rejects oversize frame', () => {
    // Build a value bigger than the cap — a giant bytes payload.
    const oversize = Value.bytes(new Uint8Array(MAX_BODY_SIZE + 1));
    expect(() => encodeFrame(valueToMsgpack(oversize))).toThrow(FrameTooLargeError);
  });
});

describe('VectorizerValue accessors', () => {
  test('asStr only matches Str', () => {
    expect(asStr(Value.str('hi'))).toBe('hi');
    expect(asStr(Value.int(7))).toBeNull();
  });

  test('asInt only matches Int', () => {
    expect(asInt(Value.int(42))).toBe(42);
    expect(asInt(Value.str('nope'))).toBeNull();
  });

  test('asArray only matches Array', () => {
    const a = Value.array([Value.int(1)]);
    expect(asArray(a)).toEqual([Value.int(1)]);
    expect(asArray(Value.int(0))).toBeNull();
  });

  test('valueFromMsgpack rejects unknown tag', () => {
    expect(() => valueFromMsgpack({ BogusTag: 1 })).toThrow(/unknown VectorizerValue tag/);
  });

  test('valueFromMsgpack rejects multi-key dict', () => {
    expect(() => valueFromMsgpack({ Int: 1, Str: 'x' })).toThrow(/exactly one key/);
  });
});
