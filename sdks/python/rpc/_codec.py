"""Frame codec for the VectorizerRPC wire protocol.

Every frame on the wire is::

    [u32 little-endian length][MessagePack body]

Length covers the body only. Bodies larger than 64 MiB are rejected to
match the server's `MAX_BODY_SIZE` and prevent OOM amplification on
malformed inputs.

The two encode/decode helpers here intentionally know nothing about
`Request` or `Response` shape — that lives in `types.py`. Splitting
framing from envelope decoding keeps the wire-spec test vectors
trivially round-trippable: encode a frame, expect specific bytes back.
"""

from __future__ import annotations

import asyncio
import socket
import struct
from typing import Any

import msgpack

# Wire spec § 1: frame body capped at 64 MiB.
MAX_BODY_SIZE: int = 64 * 1024 * 1024

# 4 bytes, little-endian, unsigned.
_LEN_HEADER: struct.Struct = struct.Struct("<I")


class FrameTooLargeError(ValueError):
    """Raised when a frame's declared length exceeds `MAX_BODY_SIZE`.

    The server enforces the same cap; mirroring it client-side avoids
    allocating a 1 GiB buffer just because a malicious peer claimed a
    silly length.
    """


class FrameDecodeError(ValueError):
    """Raised when the body bytes are not a valid MessagePack value."""


def encode_frame(value: Any) -> bytes:
    """Serialize `value` to a single complete wire frame.

    Returns ``length_header + msgpack(value)``. The header is written
    BEFORE the body so a single ``socket.sendall`` (or
    ``StreamWriter.write``) puts a complete frame on the wire.

    `use_bin_type=True` matches `rmp-serde`'s default: ``bytes`` becomes
    MessagePack `bin` (not `str`). Without this flag, raw bytes round-
    trip as `str` and decode back as `str` — which would silently
    corrupt `VectorizerValue::Bytes` payloads.
    """
    body = msgpack.packb(value, use_bin_type=True)
    if len(body) > MAX_BODY_SIZE:
        raise FrameTooLargeError(
            f"frame body is {len(body)} bytes, exceeds 64 MiB cap"
        )
    return _LEN_HEADER.pack(len(body)) + body


def decode_body(body: bytes) -> Any:
    """Decode the MessagePack body of a frame back to a Python value.

    Mirrors `encode_frame`'s `use_bin_type=True`: `raw=False` keeps
    UTF-8 strings as `str` rather than `bytes`. `strict_map_key=False`
    allows non-string keys (the server uses `Map<Value, Value>` for
    HELLO responses, where keys are `VectorizerValue::Str` variants
    that decode to single-key dicts, not bare strings).
    """
    try:
        return msgpack.unpackb(body, raw=False, strict_map_key=False)
    except (msgpack.exceptions.ExtraData, msgpack.exceptions.UnpackException, ValueError) as e:
        raise FrameDecodeError(f"frame body is not valid MessagePack: {e}") from e


def read_frame_sync(sock: socket.socket) -> Any:
    """Blocking read of one frame from `sock`. Returns the decoded body.

    Raises `ConnectionError` (or `OSError`) when the peer closes
    mid-frame. The caller is expected to catch these and either
    reconnect or propagate to the application as a typed error.
    """
    header = _read_exact_sync(sock, 4)
    (length,) = _LEN_HEADER.unpack(header)
    if length > MAX_BODY_SIZE:
        raise FrameTooLargeError(
            f"declared frame length {length} exceeds 64 MiB cap"
        )
    body = _read_exact_sync(sock, length) if length else b""
    return decode_body(body)


def _read_exact_sync(sock: socket.socket, n: int) -> bytes:
    """Read exactly `n` bytes from `sock` or raise ConnectionError."""
    buf = bytearray(n)
    view = memoryview(buf)
    pos = 0
    while pos < n:
        got = sock.recv_into(view[pos:])
        if got == 0:
            # Peer closed cleanly mid-frame. From our PoV this is an
            # error: the server promised `length` bytes and didn't
            # deliver them.
            raise ConnectionError(
                f"connection closed after {pos} of {n} expected body bytes"
            )
        pos += got
    return bytes(buf)


async def read_frame_async(reader: asyncio.StreamReader) -> Any:
    """Async read of one frame from `reader`. Returns the decoded body."""
    header = await reader.readexactly(4)
    (length,) = _LEN_HEADER.unpack(header)
    if length > MAX_BODY_SIZE:
        raise FrameTooLargeError(
            f"declared frame length {length} exceeds 64 MiB cap"
        )
    body = await reader.readexactly(length) if length else b""
    return decode_body(body)
