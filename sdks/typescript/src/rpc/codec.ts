/**
 * Frame codec for the VectorizerRPC wire protocol.
 *
 * Every frame on the wire is:
 *
 *     [u32 little-endian length][MessagePack body]
 *
 * Length covers the body only. Bodies larger than 64 MiB are rejected
 * to match the server's `MAX_BODY_SIZE` and prevent OOM amplification
 * on malformed inputs.
 *
 * The encode/decode helpers here intentionally know nothing about
 * `Request` or `Response` shape — that lives in `types.ts`. Splitting
 * framing from envelope decoding keeps the wire-spec test vectors
 * trivially round-trippable.
 */

import { encode as msgpackEncode, decode as msgpackDecode } from '@msgpack/msgpack';

/** Wire spec § 1: frame body capped at 64 MiB. */
export const MAX_BODY_SIZE = 64 * 1024 * 1024;

/** Length-prefix header size in bytes (u32 LE). */
export const HEADER_SIZE = 4;

/** Raised when a frame's declared length exceeds {@link MAX_BODY_SIZE}. */
export class FrameTooLargeError extends Error {
  constructor(actual: number) {
    super(`frame body is ${actual} bytes, exceeds 64 MiB cap`);
    this.name = 'FrameTooLargeError';
  }
}

/** Raised when the body bytes are not a valid MessagePack value. */
export class FrameDecodeError extends Error {
  constructor(reason: string) {
    super(`frame body is not valid MessagePack: ${reason}`);
    this.name = 'FrameDecodeError';
  }
}

/**
 * Serialize `value` to a single complete wire frame.
 *
 * Returns `Buffer` of `[u32 LE length][msgpack body]`. The header is
 * written before the body so a single `socket.write()` puts a complete
 * frame on the wire.
 *
 * The encoder uses `@msgpack/msgpack`, which emits MessagePack `bin`
 * for `Uint8Array` — matching `rmp-serde`'s default for
 * `VectorizerValue::Bytes`. Without that, raw bytes would round-trip
 * as `str` and silently corrupt binary payloads.
 */
export function encodeFrame(value: unknown): Buffer {
  const body = msgpackEncode(value);
  if (body.length > MAX_BODY_SIZE) {
    throw new FrameTooLargeError(body.length);
  }
  const frame = Buffer.allocUnsafe(HEADER_SIZE + body.length);
  frame.writeUInt32LE(body.length, 0);
  Buffer.from(body.buffer, body.byteOffset, body.byteLength).copy(frame, HEADER_SIZE);
  return frame;
}

/**
 * Decode the MessagePack body of a frame back to a JavaScript value.
 *
 * Mirrors `encodeFrame`: `Bytes` payloads decode as `Uint8Array`,
 * strings as `string`, ints as `number` (or `bigint` for values
 * outside the safe range).
 */
export function decodeBody(body: Uint8Array): unknown {
  try {
    return msgpackDecode(body);
  } catch (err) {
    throw new FrameDecodeError(err instanceof Error ? err.message : String(err));
  }
}

/**
 * Streaming frame parser for socket data.
 *
 * Append received bytes via {@link push}; pull complete frames via
 * {@link drain}. Buffers partial frames internally so callers don't
 * need to track length-vs-body boundaries themselves.
 */
export class FrameReader {
  private chunks: Buffer[] = [];
  private buffered = 0;

  /** Append a chunk of socket data to the internal buffer. */
  push(chunk: Buffer): void {
    this.chunks.push(chunk);
    this.buffered += chunk.length;
  }

  /**
   * Pull every complete frame currently in the buffer. Returns the
   * decoded MessagePack bodies in order. Stops when fewer bytes
   * remain than needed for the next header or body.
   *
   * Throws {@link FrameTooLargeError} if a frame declares a length
   * over the 64 MiB cap (defensive — the server enforces the same
   * cap, so seeing a larger length on the wire means corruption or a
   * misbehaving peer).
   */
  drain(): unknown[] {
    const out: unknown[] = [];
    const buffer = Buffer.concat(this.chunks, this.buffered);
    let offset = 0;

    while (offset + HEADER_SIZE <= buffer.length) {
      const length = buffer.readUInt32LE(offset);
      if (length > MAX_BODY_SIZE) {
        throw new FrameTooLargeError(length);
      }
      const frameEnd = offset + HEADER_SIZE + length;
      if (frameEnd > buffer.length) {
        break;
      }
      const body = buffer.subarray(offset + HEADER_SIZE, frameEnd);
      out.push(decodeBody(body));
      offset = frameEnd;
    }

    if (offset === buffer.length) {
      this.chunks = [];
      this.buffered = 0;
    } else if (offset > 0) {
      const remainder = buffer.subarray(offset);
      this.chunks = [remainder];
      this.buffered = remainder.length;
    }
    return out;
  }
}
