"""VectorizerRPC wire types: ``VectorizerValue``, ``Request``, ``Response``.

Wire spec § 2 + § 3: ``docs/specs/VECTORIZER_RPC.md``. The types are
Thunder's (``hivellm-thunder``, imported as ``thunder_rpc``) — the HiveLLM
family's shared binary RPC package, whose Rust twin ``vectorizer-server``
runs — so the SDK and the server cannot disagree on the wire. The on-wire
encoding is unchanged (v1, frozen): 4-byte little-endian length prefix +
MessagePack body, with the externally-tagged value/``Result`` layout
rmp-serde emits.

:class:`VectorizerValue` subclasses :class:`thunder_rpc.Value` purely to keep
this SDK's trailing-underscore factory names (``str_``, ``int_``, ``float_``,
``bool_``, ``bytes_``), which exist because ``str`` / ``int`` / ``float`` /
``bool`` / ``bytes`` are Python builtins. Thunder's own factories
(``null()``, ``array()``, ``map()``) and every accessor (``as_str()``,
``map_get()``, …) are inherited unchanged. The one visible difference from
the pre-Thunder type: ``kind`` is now lowercase (``"str"``, not ``"Str"``);
the MessagePack tag on the wire is unchanged.
"""

from __future__ import annotations

from typing import Any

from thunder_rpc import Request, Response, Value


class VectorizerValue(Value):
    """A dynamically-typed value that crosses the VectorizerRPC wire.

    Construct via the named factories — ``null()``, ``bool_(b)``,
    ``int_(i)``, ``float_(f)``, ``bytes_(b)``, ``str_(s)``, ``array(items)``,
    ``map(pairs)``. Two values are equal iff both ``kind`` and ``value``
    match, and they are hashable, both inherited from Thunder's frozen
    dataclass.

    The factories return :class:`thunder_rpc.Value` instances (this class
    adds no state), which is what the transport expects.
    """

    __slots__ = ()

    @staticmethod
    def bool_(b: bool) -> Value:
        """A boolean. Coerces truthy/falsy input, as this SDK always has."""
        return Value.bool(bool(b))

    @staticmethod
    def int_(i: int) -> Value:
        """A 64-bit signed integer. Coerces ints-in-disguise (e.g. ``True``,
        ``3.0``) the way this SDK always has."""
        return Value.int(int(i))

    @staticmethod
    def float_(f: float) -> Value:
        """A 64-bit float, always encoded as MessagePack float64."""
        return Value.float(float(f))

    @staticmethod
    def bytes_(b: bytes) -> Value:
        """Raw bytes, emitted as MessagePack ``bin``."""
        return Value.bytes(bytes(b))

    @staticmethod
    def str_(s: Any) -> Value:
        """A UTF-8 string. Coerces non-str input via ``str()``, as this SDK
        always has."""
        return Value.str(str(s))


__all__ = ["Request", "Response", "VectorizerValue"]
