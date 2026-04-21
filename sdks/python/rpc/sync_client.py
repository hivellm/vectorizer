"""Synchronous ``RpcClient`` over a single TCP connection.

Mirrors ``sdks/rust/src/rpc/client.rs`` but blocking, stdlib-only
(``socket`` + ``threading``). One background reader thread demultiplexes
responses by ``Request.id`` into per-call ``queue.Queue`` mailboxes so
concurrent calls from caller threads on the same client don't block
each other.

Auth is sticky per-connection (wire spec § 4): every connection MUST
issue ``HELLO`` before any data-plane command. The local ``call``
method enforces this client-side so callers see a clear typed error
instead of a server-side string.
"""

from __future__ import annotations

import queue
import socket
import threading
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Sequence

from rpc._codec import encode_frame, read_frame_sync
from rpc.endpoint import Endpoint, parse_endpoint
from rpc.types import Request, Response, VectorizerValue

# Sentinel returned by the reader queue when the reader thread exits.
# A separate object so it can never collide with a real Response.
_READER_DEAD = object()


class RpcClientError(Exception):
    """Base exception for ``RpcClient`` failures.

    Subclassed for the four error conditions the protocol can produce.
    Use ``isinstance`` to discriminate; the string form is also stable
    for logging.
    """


class RpcServerError(RpcClientError):
    """The server returned ``Result::Err(message)`` for the call."""


class RpcConnectionClosed(RpcClientError):
    """The reader thread exited before the response arrived.

    Either the peer closed cleanly (``EOF``) or an I/O error tore down
    the socket. Either way, the client's connection is unusable; build
    a new one.
    """


class RpcNotAuthenticated(RpcClientError):
    """A data-plane command was issued before HELLO succeeded.

    The server would also reject this; the client surfaces it locally
    so the offending caller sees a clear error without burning a
    network round-trip.
    """


@dataclass
class HelloPayload:
    """HELLO request payload — sent as the FIRST frame on a connection.

    At least one of ``token`` / ``api_key`` should be populated when
    the server has auth enabled. When the server runs in single-user
    mode (``auth.enabled: false``), credentials are accepted-but-ignored
    and the connection runs as the implicit local admin.
    """

    client_name: Optional[str] = None
    token: Optional[str] = None
    api_key: Optional[str] = None
    version: int = 1

    def with_token(self, token: str) -> "HelloPayload":
        """Return a copy carrying the given JWT bearer token. Replaces
        any previously set token/api_key."""
        return HelloPayload(
            client_name=self.client_name,
            token=token,
            api_key=None,
            version=self.version,
        )

    def with_api_key(self, api_key: str) -> "HelloPayload":
        """Return a copy carrying the given API key. Replaces any
        previously set token/api_key."""
        return HelloPayload(
            client_name=self.client_name,
            token=None,
            api_key=api_key,
            version=self.version,
        )

    def to_value(self) -> VectorizerValue:
        pairs: List = [
            (VectorizerValue.str_("version"), VectorizerValue.int_(self.version)),
        ]
        if self.token is not None:
            pairs.append((VectorizerValue.str_("token"), VectorizerValue.str_(self.token)))
        if self.api_key is not None:
            pairs.append((VectorizerValue.str_("api_key"), VectorizerValue.str_(self.api_key)))
        if self.client_name is not None:
            pairs.append(
                (VectorizerValue.str_("client_name"), VectorizerValue.str_(self.client_name))
            )
        return VectorizerValue.map(pairs)


@dataclass
class HelloResponse:
    """Decoded HELLO success payload from the server."""

    server_version: str
    protocol_version: int
    authenticated: bool
    admin: bool
    capabilities: List[str] = field(default_factory=list)

    @classmethod
    def parse(cls, value: VectorizerValue) -> "HelloResponse":
        sv = value.map_get("server_version")
        pv = value.map_get("protocol_version")
        au = value.map_get("authenticated")
        ad = value.map_get("admin")
        caps = value.map_get("capabilities")
        return cls(
            server_version=(sv.as_str() if sv is not None else None) or "",
            protocol_version=(pv.as_int() if pv is not None else None) or 0,
            authenticated=(au.as_bool() if au is not None else None) or False,
            admin=(ad.as_bool() if ad is not None else None) or False,
            capabilities=[
                v.as_str() or ""
                for v in ((caps.as_array() if caps is not None else None) or [])
                if v.as_str() is not None
            ],
        )


# Auth-exempt commands per wire spec § 4.
_AUTH_EXEMPT = frozenset({"HELLO", "PING"})


class RpcClient:
    """One synchronous connection to a Vectorizer RPC server.

    Construct with :meth:`connect` (raw ``host:port``) or
    :meth:`connect_url` (``vectorizer://`` URL). Always issue
    :meth:`hello` before any data-plane call.

    Thread-safe: multiple caller threads may call methods concurrently;
    requests serialize on a writer lock and responses are demultiplexed
    by ``id`` into per-call queues.
    """

    def __init__(self, sock: socket.socket) -> None:
        self._sock = sock
        # The writer lock guarantees frames don't interleave on the
        # wire when multiple threads call concurrently.
        self._writer_lock = threading.Lock()
        # Map from request id → mailbox queue (size 1) for the response.
        self._pending: Dict[int, "queue.Queue[object]"] = {}
        self._pending_lock = threading.Lock()
        self._next_id = 1
        self._id_lock = threading.Lock()
        self._authenticated = False
        self._auth_lock = threading.Lock()
        self._closed = False
        self._reader = threading.Thread(
            target=self._read_loop, name="vectorizer-rpc-reader", daemon=True
        )
        self._reader.start()

    # ── construction ─────────────────────────────────────────────────
    @classmethod
    def connect(cls, address: str, timeout: Optional[float] = None) -> "RpcClient":
        """Open a TCP connection to ``address`` (``host:port``).

        Does NOT send HELLO — callers MUST call :meth:`hello` before
        any data-plane command, or the server will reject it.

        ``timeout`` (seconds) controls only the connect step. Once the
        socket is established it switches to blocking mode so the
        reader thread blocks on ``recv`` indefinitely.
        """
        host, port = _split_host_port(address)
        sock = socket.create_connection((host, port), timeout=timeout)
        # Disable Nagle: every RPC frame is a complete request, latency
        # matters more than packing several into one segment.
        sock.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
        sock.settimeout(None)
        return cls(sock)

    @classmethod
    def connect_url(cls, url: str, timeout: Optional[float] = None) -> "RpcClient":
        """Parse a ``vectorizer://host[:port]`` URL and dial it.

        REST URLs (``http(s)://``) are rejected with a clear error
        pointing the caller at the HTTP client; an ``http://`` URL is
        not a transport an RPC client can speak.
        """
        ep = parse_endpoint(url)
        if isinstance(ep, Endpoint.Rpc):
            return cls.connect(f"{ep.host}:{ep.port}", timeout=timeout)
        if isinstance(ep, Endpoint.Rest):
            raise RpcServerError(
                f"RpcClient cannot dial REST URL '{ep.url}'; "
                f"use the HTTP client (VectorizerClient) instead, "
                f"or pass a 'vectorizer://' URL"
            )
        raise RpcServerError(f"unrecognised endpoint shape: {ep!r}")

    # ── handshake + health ───────────────────────────────────────────
    def hello(self, payload: HelloPayload) -> HelloResponse:
        """Issue the HELLO handshake. Must be the first call on a
        fresh connection. Returns the server's capability list and
        auth flags."""
        result = self._raw_call("HELLO", [payload.to_value()])
        parsed = HelloResponse.parse(result)
        if parsed.authenticated:
            with self._auth_lock:
                self._authenticated = True
        return parsed

    def ping(self) -> str:
        """Health check. Auth-exempt per wire spec § 4 — works pre-HELLO."""
        result = self._raw_call("PING", [])
        s = result.as_str()
        if s is None:
            raise RpcServerError("PING returned non-string payload")
        return s

    # ── generic dispatch ─────────────────────────────────────────────
    def call(
        self, command: str, args: Optional[Sequence[VectorizerValue]] = None
    ) -> VectorizerValue:
        """Dispatch a generic command. Most callers should reach for a
        typed wrapper from :mod:`rpc.commands` instead.

        Enforces the local auth gate: data-plane commands raise
        :class:`RpcNotAuthenticated` before sending if HELLO hasn't
        succeeded.
        """
        if command not in _AUTH_EXEMPT:
            with self._auth_lock:
                if not self._authenticated:
                    raise RpcNotAuthenticated(
                        "HELLO must succeed before any data-plane command can be issued"
                    )
        return self._raw_call(command, list(args or []))

    def is_authenticated(self) -> bool:
        with self._auth_lock:
            return self._authenticated

    # ── shutdown ─────────────────────────────────────────────────────
    def close(self) -> None:
        """Close the underlying socket. In-flight calls receive
        :class:`RpcConnectionClosed`."""
        if self._closed:
            return
        self._closed = True
        try:
            # Half-shutdown so the reader thread sees EOF cleanly.
            self._sock.shutdown(socket.SHUT_RDWR)
        except OSError:
            pass
        try:
            self._sock.close()
        except OSError:
            pass
        # Wake any stuck callers.
        self._fail_all_pending()

    def __enter__(self) -> "RpcClient":
        return self

    def __exit__(self, *_exc: object) -> None:
        self.close()

    def __del__(self) -> None:  # pragma: no cover — destructor path
        try:
            self.close()
        except Exception:
            pass

    # ── internals ────────────────────────────────────────────────────
    def _alloc_id(self) -> int:
        # u32 wrap (matches the Rust SDK; collisions on a long-lived
        # connection are vanishingly rare because in-flight is bounded
        # by application backpressure).
        with self._id_lock:
            rid = self._next_id
            self._next_id = (self._next_id + 1) & 0xFFFFFFFF
            if self._next_id == 0:
                self._next_id = 1
        return rid

    def _raw_call(self, command: str, args: List[VectorizerValue]) -> VectorizerValue:
        rid = self._alloc_id()
        mailbox: "queue.Queue[object]" = queue.Queue(maxsize=1)
        with self._pending_lock:
            self._pending[rid] = mailbox

        req = Request(id=rid, command=command, args=args)
        frame = encode_frame(req.to_msgpack())

        try:
            with self._writer_lock:
                self._sock.sendall(frame)
        except OSError as e:
            with self._pending_lock:
                self._pending.pop(rid, None)
            raise RpcConnectionClosed(f"send failed: {e}") from e

        # Block on the mailbox. The reader thread either pushes a
        # Response or pushes _READER_DEAD on shutdown.
        item = mailbox.get()
        if item is _READER_DEAD:
            raise RpcConnectionClosed("connection closed before response")
        assert isinstance(item, Response)
        tag, payload = item.result
        if tag == "Ok":
            assert isinstance(payload, VectorizerValue)
            return payload
        raise RpcServerError(str(payload))

    def _read_loop(self) -> None:
        try:
            while True:
                raw = read_frame_sync(self._sock)
                resp = Response.from_msgpack(raw)
                with self._pending_lock:
                    mailbox = self._pending.pop(resp.id, None)
                if mailbox is not None:
                    try:
                        mailbox.put_nowait(resp)
                    except queue.Full:  # pragma: no cover — mailbox is size 1, only one writer
                        pass
                # else: response with no pending caller — drop silently.
        except (ConnectionError, OSError, ValueError, EOFError):
            # Includes msgpack decode errors (ValueError) and clean EOF
            # (asyncio.IncompleteReadError analog: ConnectionError from
            # _read_exact_sync).
            pass
        finally:
            self._fail_all_pending()

    def _fail_all_pending(self) -> None:
        with self._pending_lock:
            mailboxes = list(self._pending.values())
            self._pending.clear()
        for mb in mailboxes:
            try:
                mb.put_nowait(_READER_DEAD)
            except queue.Full:  # pragma: no cover
                pass


def _split_host_port(address: str) -> tuple[str, int]:
    """Split a ``host:port`` string. IPv6 literals (``[::1]:1234``) are
    handled specially so the colons inside the brackets aren't treated
    as port separators."""
    if address.startswith("["):
        close = address.find("]")
        if close < 0:
            raise ValueError(f"unterminated IPv6 literal in address: {address!r}")
        host = address[1:close]
        rest = address[close + 1 :]
        if not rest.startswith(":"):
            raise ValueError(
                f"expected ':<port>' after IPv6 literal in address: {address!r}"
            )
        port = int(rest[1:])
        return host, port
    if ":" not in address:
        raise ValueError(f"address must include ':<port>', got {address!r}")
    host, _, port_str = address.rpartition(":")
    return host, int(port_str)
