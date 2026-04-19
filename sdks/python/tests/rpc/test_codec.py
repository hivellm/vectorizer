"""Codec + types unit tests.

Covers two distinct invariants:

1. **Wire-spec golden vectors** — the bytes produced by encoding a
   ``Request`` / ``Response`` exactly match the hex dumps in
   ``docs/specs/VECTORIZER_RPC.md`` § 11. If these break, the Python
   SDK can no longer talk to a Rust server.

2. **Round-trip** for every ``VectorizerValue`` variant: encode →
   decode → equal-value-back.
"""

from __future__ import annotations

import os
import sys

import pytest

_SDK_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
if _SDK_ROOT not in sys.path:
    sys.path.insert(0, _SDK_ROOT)

from rpc._codec import (  # noqa: E402
    MAX_BODY_SIZE,
    FrameTooLargeError,
    decode_body,
    encode_frame,
)
from rpc.types import Request, Response, VectorizerValue  # noqa: E402


class TestWireGoldenVectors:
    """Bit-exact tests against the hex dumps in the wire spec."""

    def test_request_ping_matches_spec(self):
        # Spec § 11: Request{id=1, command="PING", args=[]}
        # 08 00 00 00  93  01  a4 50 49 4e 47  90
        req = Request(id=1, command="PING", args=[])
        frame = encode_frame(req.to_msgpack())
        expected = (
            bytes.fromhex("08000000")
            + bytes.fromhex("93")
            + bytes.fromhex("01")
            + bytes.fromhex("a4")
            + b"PING"
            + bytes.fromhex("90")
        )
        assert frame == expected

    def test_response_ok_pong_matches_spec(self):
        # Spec § 11: Response{id=1, result=Ok(Str("PONG"))}
        # Both Result<T,E> and VectorizerValue use the externally-tagged
        # encoding, so an Ok(Str) produces TWO nested single-key maps.
        # 10 00 00 00  92  01  81 a2 4f 6b  81 a3 53 74 72 a4 50 4f 4e 47
        resp = Response.ok(1, VectorizerValue.str_("PONG"))
        frame = encode_frame(resp.to_msgpack())
        expected = (
            bytes.fromhex("10000000")
            + bytes.fromhex("92")
            + bytes.fromhex("01")
            + bytes.fromhex("81")
            + bytes.fromhex("a2")
            + b"Ok"
            + bytes.fromhex("81")
            + bytes.fromhex("a3")
            + b"Str"
            + bytes.fromhex("a4")
            + b"PONG"
        )
        assert frame == expected


class TestRoundTrip:
    """Each ``VectorizerValue`` variant survives encode → decode."""

    @pytest.mark.parametrize(
        "value",
        [
            VectorizerValue.null(),
            VectorizerValue.bool_(True),
            VectorizerValue.bool_(False),
            VectorizerValue.int_(0),
            VectorizerValue.int_(-(2**63)),  # i64::MIN
            VectorizerValue.int_(2**63 - 1),  # i64::MAX
            VectorizerValue.float_(1.5),
            VectorizerValue.float_(-3.14159),
            VectorizerValue.bytes_(b"\x00\x01\x02\xff"),
            VectorizerValue.str_(""),
            VectorizerValue.str_("hello"),
            VectorizerValue.str_("ünïcödé"),
            VectorizerValue.array(
                [VectorizerValue.int_(1), VectorizerValue.str_("two")]
            ),
            VectorizerValue.map(
                [
                    (VectorizerValue.str_("k"), VectorizerValue.int_(99)),
                    (VectorizerValue.str_("nested"),
                     VectorizerValue.array([VectorizerValue.bool_(True)])),
                ]
            ),
        ],
    )
    def test_value_roundtrip(self, value: VectorizerValue):
        frame = encode_frame(value.to_msgpack())
        decoded = VectorizerValue.from_msgpack(decode_body(frame[4:]))
        assert decoded == value

    def test_response_err_roundtrip(self):
        resp = Response.err(42, "something went wrong")
        frame = encode_frame(resp.to_msgpack())
        decoded = Response.from_msgpack(decode_body(frame[4:]))
        assert decoded.id == 42
        assert decoded.result == ("Err", "something went wrong")
        assert not decoded.is_ok()

    def test_request_with_mixed_args_roundtrip(self):
        req = Request(
            id=99,
            command="search.basic",
            args=[
                VectorizerValue.str_("alpha-docs"),
                VectorizerValue.str_("query text"),
                VectorizerValue.int_(10),
                VectorizerValue.float_(0.5),
            ],
        )
        frame = encode_frame(req.to_msgpack())
        # Decode back — Request doesn't have a from_msgpack helper
        # (clients only encode requests), so decode the raw shape
        # directly to verify field positioning.
        body = decode_body(frame[4:])
        assert body[0] == 99
        assert body[1] == "search.basic"
        assert len(body[2]) == 4
        assert body[2][0] == {"Str": "alpha-docs"}
        assert body[2][2] == {"Int": 10}


class TestFrameLimits:
    """The 64 MiB cap must be enforced on both encode and declared decode."""

    def test_encode_rejects_oversize_frame(self):
        # Build a value bigger than the cap — a giant bytes payload.
        oversize = VectorizerValue.bytes_(b"\x00" * (MAX_BODY_SIZE + 1))
        with pytest.raises(FrameTooLargeError, match="64 MiB"):
            encode_frame(oversize.to_msgpack())


class TestVectorizerValueAccessors:
    """The ``as_*`` and ``map_get`` helpers should match the Rust SDK."""

    def test_as_str_only_for_str_variant(self):
        assert VectorizerValue.str_("hi").as_str() == "hi"
        assert VectorizerValue.int_(7).as_str() is None

    def test_as_float_promotes_int(self):
        assert VectorizerValue.float_(2.5).as_float() == 2.5
        assert VectorizerValue.int_(3).as_float() == 3.0
        assert VectorizerValue.str_("nope").as_float() is None

    def test_map_get_finds_string_keyed_value(self):
        m = VectorizerValue.map(
            [
                (VectorizerValue.str_("name"), VectorizerValue.str_("alpha")),
                (VectorizerValue.str_("count"), VectorizerValue.int_(42)),
            ]
        )
        assert m.map_get("name").as_str() == "alpha"
        assert m.map_get("count").as_int() == 42
        assert m.map_get("missing") is None

    def test_from_msgpack_rejects_unknown_tag(self):
        with pytest.raises(ValueError, match="unknown VectorizerValue tag"):
            VectorizerValue.from_msgpack({"BogusTag": 1})

    def test_from_msgpack_rejects_multi_key_dict(self):
        with pytest.raises(ValueError, match="exactly one key"):
            VectorizerValue.from_msgpack({"Int": 1, "Str": "x"})
