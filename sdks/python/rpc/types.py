"""VectorizerRPC wire types: ``VectorizerValue``, ``Request``, ``Response``.

Wire spec § 2 + § 3: ``docs/specs/VECTORIZER_RPC.md``. Mirrors the Rust
SDK at ``sdks/rust/src/rpc/types.rs`` byte-for-byte.

The ``VectorizerValue`` tagged union encodes to MessagePack using
rmp-serde's externally-tagged enum representation:

- Unit variant (``Null``) → bare string ``"Null"``.
- Newtype variant (``Int(42)``) → single-key map ``{"Int": 42}``.

Both ``Result<T, E>`` (used by ``Response.result``) and
``VectorizerValue`` use this same encoding, which means an
``Ok(Str("PONG"))`` round-trips on the wire as **two** nested
single-key maps (``{"Ok": {"Str": "PONG"}}``). The decoder here
unwraps both layers; callers see plain Python values.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, List, Optional, Tuple, Union

# Tag strings — match the Rust enum variant names exactly. Anything
# else returned by the server is a wire-format violation.
_NULL = "Null"
_BOOL = "Bool"
_INT = "Int"
_FLOAT = "Float"
_BYTES = "Bytes"
_STR = "Str"
_ARRAY = "Array"
_MAP = "Map"

_RESULT_OK = "Ok"
_RESULT_ERR = "Err"


@dataclass(frozen=True)
class VectorizerValue:
    """A dynamically-typed value that crosses the VectorizerRPC wire.

    Construct via the named classmethods (``null()``, ``bool_(b)``,
    ``int_(i)``, ``float_(f)``, ``bytes_(b)``, ``str_(s)``,
    ``array(items)``, ``map(pairs)``) — the underscore suffixes avoid
    clashing with Python builtins.

    Two values are equal iff both ``kind`` and ``value`` match. Hashable
    when the inner value is hashable (so values can be used as dict
    keys for assertion purposes).
    """

    kind: str
    value: Any = None

    # ── constructors ─────────────────────────────────────────────────
    @classmethod
    def null(cls) -> "VectorizerValue":
        return cls(_NULL, None)

    @classmethod
    def bool_(cls, b: bool) -> "VectorizerValue":
        return cls(_BOOL, bool(b))

    @classmethod
    def int_(cls, i: int) -> "VectorizerValue":
        return cls(_INT, int(i))

    @classmethod
    def float_(cls, f: float) -> "VectorizerValue":
        return cls(_FLOAT, float(f))

    @classmethod
    def bytes_(cls, b: bytes) -> "VectorizerValue":
        return cls(_BYTES, bytes(b))

    @classmethod
    def str_(cls, s: str) -> "VectorizerValue":
        return cls(_STR, str(s))

    @classmethod
    def array(cls, items: List["VectorizerValue"]) -> "VectorizerValue":
        return cls(_ARRAY, list(items))

    @classmethod
    def map(
        cls, pairs: List[Tuple["VectorizerValue", "VectorizerValue"]]
    ) -> "VectorizerValue":
        return cls(_MAP, list(pairs))

    # ── accessors ────────────────────────────────────────────────────
    def as_str(self) -> Optional[str]:
        return self.value if self.kind == _STR else None

    def as_bytes(self) -> Optional[bytes]:
        if self.kind == _BYTES:
            return self.value
        if self.kind == _STR:
            return self.value.encode("utf-8")
        return None

    def as_int(self) -> Optional[int]:
        return self.value if self.kind == _INT else None

    def as_float(self) -> Optional[float]:
        if self.kind == _FLOAT:
            return self.value
        if self.kind == _INT:
            return float(self.value)
        return None

    def as_bool(self) -> Optional[bool]:
        return self.value if self.kind == _BOOL else None

    def as_array(self) -> Optional[List["VectorizerValue"]]:
        return self.value if self.kind == _ARRAY else None

    def as_map(self) -> Optional[List[Tuple["VectorizerValue", "VectorizerValue"]]]:
        return self.value if self.kind == _MAP else None

    def map_get(self, key: str) -> Optional["VectorizerValue"]:
        """Look up a string-keyed map entry. Returns ``None`` when the
        receiver is not a Map or when the key is missing.

        This is the workhorse for decoding HELLO responses and other
        named-field maps coming back from the server.
        """
        pairs = self.as_map()
        if pairs is None:
            return None
        for k, v in pairs:
            if k.kind == _STR and k.value == key:
                return v
        return None

    # ── codec ────────────────────────────────────────────────────────
    def to_msgpack(self) -> Any:
        """Convert to a Python value ready for ``msgpack.packb``.

        Unit variant becomes a bare tag string; everything else becomes
        a single-key dict mapping the tag to its payload. Nested values
        recurse so an ``Array(Map(...))`` survives the round-trip.
        """
        if self.kind == _NULL:
            return _NULL
        if self.kind == _ARRAY:
            return {_ARRAY: [v.to_msgpack() for v in self.value]}
        if self.kind == _MAP:
            return {
                _MAP: [
                    [k.to_msgpack(), v.to_msgpack()] for (k, v) in self.value
                ]
            }
        return {self.kind: self.value}

    @classmethod
    def from_msgpack(cls, raw: Any) -> "VectorizerValue":
        """Decode an externally-tagged msgpack value back to a typed
        ``VectorizerValue``.

        Raises :class:`ValueError` if the input doesn't match the wire
        format (e.g. a multi-key dict, an unknown tag).
        """
        if isinstance(raw, str):
            if raw == _NULL:
                return cls.null()
            raise ValueError(f"unknown unit-variant tag: {raw!r}")
        if not isinstance(raw, dict):
            raise ValueError(
                f"expected externally-tagged dict or 'Null', got {type(raw).__name__}: {raw!r}"
            )
        if len(raw) != 1:
            raise ValueError(
                f"externally-tagged value must have exactly one key, got {len(raw)}: {raw!r}"
            )
        ((tag, payload),) = raw.items()
        if tag == _BOOL:
            return cls.bool_(payload)
        if tag == _INT:
            return cls.int_(payload)
        if tag == _FLOAT:
            return cls.float_(payload)
        if tag == _BYTES:
            # msgpack with raw=False decodes `bin` as bytes already.
            return cls.bytes_(payload)
        if tag == _STR:
            return cls.str_(payload)
        if tag == _ARRAY:
            return cls.array([cls.from_msgpack(item) for item in payload])
        if tag == _MAP:
            return cls.map(
                [
                    (cls.from_msgpack(k), cls.from_msgpack(v))
                    for (k, v) in payload
                ]
            )
        raise ValueError(f"unknown VectorizerValue tag: {tag!r}")


# ── Wire frames ─────────────────────────────────────────────────────────────


@dataclass
class Request:
    """A request from client to server. Wire spec § 2.

    Encoded on the wire as a 3-element MessagePack array
    ``[id, command, args]`` to match rmp-serde's default struct
    representation.
    """

    id: int
    command: str
    args: List[VectorizerValue] = field(default_factory=list)

    def to_msgpack(self) -> List[Any]:
        return [self.id, self.command, [v.to_msgpack() for v in self.args]]


@dataclass
class Response:
    """A response from server to client. Wire spec § 2.

    ``result`` is a tagged tuple ``("Ok", VectorizerValue)`` or
    ``("Err", "message")`` to mirror Rust's ``Result<Value, String>``.
    """

    id: int
    result: Tuple[str, Union[VectorizerValue, str]]

    @classmethod
    def ok(cls, id: int, value: VectorizerValue) -> "Response":
        return cls(id=id, result=(_RESULT_OK, value))

    @classmethod
    def err(cls, id: int, message: str) -> "Response":
        return cls(id=id, result=(_RESULT_ERR, str(message)))

    def to_msgpack(self) -> List[Any]:
        tag, payload = self.result
        if tag == _RESULT_OK:
            assert isinstance(payload, VectorizerValue)
            inner: Any = payload.to_msgpack()
        else:
            inner = payload
        return [self.id, {tag: inner}]

    @classmethod
    def from_msgpack(cls, raw: Any) -> "Response":
        if not isinstance(raw, (list, tuple)) or len(raw) != 2:
            raise ValueError(
                f"Response wire frame must be a 2-element array, got {raw!r}"
            )
        rid, result_raw = raw
        if not isinstance(result_raw, dict) or len(result_raw) != 1:
            raise ValueError(
                f"Response.result must be a single-key map, got {result_raw!r}"
            )
        ((tag, payload),) = result_raw.items()
        if tag == _RESULT_OK:
            return cls(id=int(rid), result=(_RESULT_OK, VectorizerValue.from_msgpack(payload)))
        if tag == _RESULT_ERR:
            if not isinstance(payload, str):
                raise ValueError(f"Err payload must be a string, got {payload!r}")
            return cls(id=int(rid), result=(_RESULT_ERR, payload))
        raise ValueError(f"unknown Result tag: {tag!r}")

    def is_ok(self) -> bool:
        return self.result[0] == _RESULT_OK

    def unwrap(self) -> VectorizerValue:
        """Return the Ok payload or raise ``RuntimeError`` on Err."""
        tag, payload = self.result
        if tag == _RESULT_OK:
            assert isinstance(payload, VectorizerValue)
            return payload
        raise RuntimeError(f"unwrap on Err response: {payload}")
