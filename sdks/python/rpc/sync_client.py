"""Synchronous ``RpcClient`` over a single TCP connection.

The transport is Thunder's (:class:`thunder_rpc.Client`): one connection per
client, responses demultiplexed by frame id, bounded in-flight, connect and
per-call timeouts, lazy re-dial and typed errors. What lives here is
Vectorizer's shape on top of it — the ``vectorizer://`` protocol config, the
HELLO payload/response types, and the exception hierarchy the typed wrappers
in :mod:`rpc.commands` raise.

Auth is sticky per-connection (wire spec § 4), and Thunder carries credentials
in the connection handshake (``AUTH``) rather than in a command.
:meth:`RpcClient.hello` therefore re-dials when its payload carries a token or
an API key, so the credentials reach the session later commands run under; the
HELLO command itself still runs, because the server answers it with the
capability list and auth flags this client surfaces.

Thread-safe: multiple caller threads may call methods concurrently.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import List, Optional, Sequence

from thunder_rpc import (
    AuthError,
    Client,
    ClientConfig,
    Config,
    Credentials,
    ErrorConvention,
    Handshake,
    HelloStyle,
    PushPolicy,
    ServerError,
    ThunderError,
)

# Thunder's `ConnectionError` and `TimeoutError` shadow the builtins of the
# same name; alias them so `isinstance` checks below cannot silently test the
# wrong class.
from thunder_rpc import ConnectionError as ThunderConnectionError
from thunder_rpc import TimeoutError as ThunderTimeoutError

from rpc.endpoint import DEFAULT_RPC_PORT, Endpoint, parse_endpoint
from rpc.types import VectorizerValue

#: Frame-body cap, matching the server's listener so neither end rejects a
#: frame the other is willing to send.
MAX_FRAME_BYTES = 512 * 1024 * 1024


def protocol_config() -> Config:
    """How Vectorizer uses the Thunder wire.

    The client half of the server's ``vectorizer_config()``: the
    ``vectorizer`` scheme, an ``AUTH``-command handshake, no HELLO
    negotiation (the ``HELLO`` *command* is Vectorizer's own), no server
    push, and RESP3-style error prefixes. Declared here rather than imported
    so the SDK depends only on published packages.
    """
    return Config(
        scheme="vectorizer",
        default_port=DEFAULT_RPC_PORT,
        handshake=Handshake.AUTH_COMMAND,
        hello_style=HelloStyle.NOT_USED,
        push=PushPolicy.RESERVED,
        error_codes=ErrorConvention.RESP3_PREFIXES,
        max_frame_bytes=MAX_FRAME_BYTES,
    )


class RpcClientError(Exception):
    """Base exception for ``RpcClient`` failures.

    Subclassed for the error conditions the protocol can produce. Use
    ``isinstance`` to discriminate; the string form is also stable for
    logging.
    """


class RpcServerError(RpcClientError):
    """The server returned ``Result::Err(message)`` for the call."""


class RpcConnectionClosed(RpcClientError):
    """The connection failed: the dial was refused, the write failed, or the
    peer went away while the call was pending.

    Thunder re-dials lazily on the next call; a dial that cannot be
    re-established keeps raising this.
    """


class RpcNotAuthenticated(RpcClientError):
    """The server refused the session's credentials.

    Raised for ``NOAUTH`` (no ``AUTH`` sent, or HELLO issued without
    credentials against an auth-enabled server), ``WRONGPASS``, and
    ``NOPERM`` on an admin-only command.
    """


class RpcTimeout(RpcClientError):
    """The connect or per-call timeout elapsed."""


class RpcProtocolError(RpcClientError):
    """The peer sent a malformed or oversized frame; the connection is
    poisoned and the next call re-dials."""


def _to_rpc_error(exc: ThunderError) -> RpcClientError:
    """Map a typed Thunder error onto this SDK's exception hierarchy."""
    if isinstance(exc, AuthError):
        return RpcNotAuthenticated(str(exc))
    if isinstance(exc, ServerError):
        return RpcServerError(str(exc))
    if isinstance(exc, ThunderTimeoutError):
        return RpcTimeout(str(exc))
    if isinstance(exc, ThunderConnectionError):
        return RpcConnectionClosed(str(exc))
    return RpcProtocolError(str(exc))


@dataclass
class HelloPayload:
    """HELLO request payload.

    At least one of ``token`` / ``api_key`` should be populated when the
    server has auth enabled: those credentials travel in the connection
    handshake, so passing them to :meth:`RpcClient.hello` is what
    authenticates the session. When the server runs in single-user mode
    (``auth.enabled: false``) the listener is open, credentials are
    accepted-but-ignored, and the connection runs as the implicit local
    admin.
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

    def credentials(self) -> Optional[Credentials]:
        """The handshake credentials this payload carries, if any."""
        if self.token is not None:
            return Credentials.token(self.token)
        if self.api_key is not None:
            return Credentials.api_key(self.api_key)
        return None

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


class RpcClient:
    """One synchronous connection to a Vectorizer RPC server.

    Construct with :meth:`connect` (raw ``host:port``) or
    :meth:`connect_url` (``vectorizer://`` URL). Issue :meth:`hello` with
    credentials when the server enforces auth.
    """

    def __init__(self, client: Client, endpoint: str, client_config: ClientConfig) -> None:
        self._client = client
        self._endpoint = endpoint
        self._client_config = client_config
        self._closed = False

    # ── construction ─────────────────────────────────────────────────
    @classmethod
    def connect(cls, address: str, timeout: Optional[float] = None) -> "RpcClient":
        """Dial ``address`` — ``host:port``, or any form
        :func:`thunder_rpc.parse_endpoint` accepts.

        Does NOT authenticate: pass credentials to :meth:`hello`, which
        re-dials with them in the handshake.

        ``timeout`` (seconds) sets both the connect and the per-call
        timeout; ``None`` keeps Thunder's defaults (10 s / 30 s).
        """
        client_config = ClientConfig(client_name="vectorizer-python-sdk")
        if timeout is not None:
            client_config = ClientConfig(
                connect_timeout=timeout,
                call_timeout=timeout,
                client_name="vectorizer-python-sdk",
            )
        return cls(cls._dial(address, client_config), address, client_config)

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

    @staticmethod
    def _dial(endpoint: str, client_config: ClientConfig) -> Client:
        try:
            return Client.connect(endpoint, protocol_config(), client_config)
        except ThunderError as exc:
            raise _to_rpc_error(exc) from exc

    # ── handshake + health ───────────────────────────────────────────
    def hello(self, payload: HelloPayload) -> HelloResponse:
        """Issue the HELLO handshake and return the server's capability
        list and auth flags.

        When ``payload`` carries a token or an API key, the connection is
        re-dialed so those credentials travel in Thunder's ``AUTH``
        handshake — that is what authenticates the session every later
        command runs under. A credential-free payload reuses the existing
        connection.
        """
        credentials = payload.credentials()
        if credentials is not None:
            client_config = ClientConfig(
                connect_timeout=self._client_config.connect_timeout,
                call_timeout=self._client_config.call_timeout,
                credentials=credentials,
                client_name=payload.client_name or self._client_config.client_name,
            )
            fresh = self._dial(self._endpoint, client_config)
            previous = self._client
            self._client = fresh
            self._client_config = client_config
            previous.close()
        return HelloResponse.parse(self.call("HELLO", [payload.to_value()]))

    def ping(self) -> str:
        """Health check. Auth-exempt per wire spec § 4 — works pre-HELLO."""
        s = self.call("PING", []).as_str()
        if s is None:
            raise RpcServerError("PING returned non-string payload")
        return s

    # ── generic dispatch ─────────────────────────────────────────────
    def call(
        self, command: str, args: Optional[Sequence[VectorizerValue]] = None
    ) -> VectorizerValue:
        """Dispatch a generic command. Most callers should reach for a
        typed wrapper from :mod:`rpc.commands` instead.

        The server gates un-authenticated sessions, so a data-plane command
        on a session that never authenticated raises
        :class:`RpcNotAuthenticated`.
        """
        try:
            return self._client.call(command, list(args or []))
        except ThunderError as exc:
            raise _to_rpc_error(exc) from exc

    def is_authenticated(self) -> bool:
        """``True`` once the connection's handshake authenticated. Always
        ``False`` against an open (single-user) server, which authenticates
        nobody because it gates nothing."""
        return self._client.is_authenticated()

    # ── shutdown ─────────────────────────────────────────────────────
    def close(self) -> None:
        """Close the connection. In-flight calls receive
        :class:`RpcConnectionClosed`."""
        if self._closed:
            return
        self._closed = True
        self._client.close()

    def __enter__(self) -> "RpcClient":
        return self

    def __exit__(self, *_exc: object) -> None:
        self.close()

    def __del__(self) -> None:  # pragma: no cover — destructor path
        try:
            self.close()
        except Exception:
            pass


__all__ = [
    "MAX_FRAME_BYTES",
    "HelloPayload",
    "HelloResponse",
    "RpcClient",
    "RpcClientError",
    "RpcConnectionClosed",
    "RpcNotAuthenticated",
    "RpcProtocolError",
    "RpcServerError",
    "RpcTimeout",
    "protocol_config",
]
