"""End-to-end integration tests for ``RpcClient`` / ``AsyncRpcClient``.

Spins up an in-test server on ``127.0.0.1:0`` that speaks the
VectorizerRPC wire format using the SDK's own codec + types (because
the production server isn't available as a Python dependency), and
drives it from both the sync and async clients to prove:

- HELLO handshake produces the expected :class:`HelloResponse` shape.
- ``PING`` works pre-HELLO (auth-exempt per wire spec § 4).
- A data-plane command before HELLO returns
  :class:`RpcNotAuthenticated` from the local gate.
- Concurrent calls on the same connection are demultiplexed by
  ``Request.id`` correctly.
- Typed wrappers (``list_collections``, ``get_collection_info``,
  ``search_basic``) round-trip through the codec.
- ``connect_url`` accepts the canonical ``vectorizer://`` form and
  rejects REST URLs with a clear error.
"""

from __future__ import annotations

import asyncio
import os
import socket
import sys
import threading
import time
from typing import List, Optional

import pytest

_SDK_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
if _SDK_ROOT not in sys.path:
    sys.path.insert(0, _SDK_ROOT)

from rpc import (  # noqa: E402
    AsyncRpcClient,
    HelloPayload,
    RpcClient,
    RpcNotAuthenticated,
    RpcServerError,
)
from rpc._codec import encode_frame, read_frame_async, read_frame_sync  # noqa: E402
from rpc.types import Request, Response, VectorizerValue  # noqa: E402


# ─────────────────────────────────────────────────────────────────────────────
# In-test fake-server fixture
# ─────────────────────────────────────────────────────────────────────────────


def _build_hello_response(rid: int) -> Response:
    return Response.ok(
        rid,
        VectorizerValue.map(
            [
                (VectorizerValue.str_("server_version"),
                 VectorizerValue.str_("test-fixture/0.0.0")),
                (VectorizerValue.str_("protocol_version"),
                 VectorizerValue.int_(1)),
                (VectorizerValue.str_("authenticated"),
                 VectorizerValue.bool_(True)),
                (VectorizerValue.str_("admin"),
                 VectorizerValue.bool_(True)),
                (VectorizerValue.str_("capabilities"),
                 VectorizerValue.array(
                    [
                        VectorizerValue.str_("PING"),
                        VectorizerValue.str_("collections.list"),
                        VectorizerValue.str_("collections.get_info"),
                        VectorizerValue.str_("vectors.get"),
                        VectorizerValue.str_("search.basic"),
                    ]
                 )),
            ]
        ),
    )


def _build_collection_info_response(rid: int, name: str) -> Response:
    return Response.ok(
        rid,
        VectorizerValue.map(
            [
                (VectorizerValue.str_("name"), VectorizerValue.str_(name)),
                (VectorizerValue.str_("vector_count"), VectorizerValue.int_(42)),
                (VectorizerValue.str_("document_count"), VectorizerValue.int_(10)),
                (VectorizerValue.str_("dimension"), VectorizerValue.int_(384)),
                (VectorizerValue.str_("metric"), VectorizerValue.str_("Cosine")),
                (VectorizerValue.str_("created_at"),
                 VectorizerValue.str_("2026-04-19T00:00:00Z")),
                (VectorizerValue.str_("updated_at"),
                 VectorizerValue.str_("2026-04-19T00:00:00Z")),
            ]
        ),
    )


def _build_search_basic_response(rid: int) -> Response:
    return Response.ok(
        rid,
        VectorizerValue.array(
            [
                VectorizerValue.map(
                    [
                        (VectorizerValue.str_("id"), VectorizerValue.str_("vec-0")),
                        (VectorizerValue.str_("score"), VectorizerValue.float_(0.95)),
                        (VectorizerValue.str_("payload"),
                         VectorizerValue.str_('{"title":"hit one"}')),
                    ]
                ),
                VectorizerValue.map(
                    [
                        (VectorizerValue.str_("id"), VectorizerValue.str_("vec-1")),
                        (VectorizerValue.str_("score"), VectorizerValue.float_(0.81)),
                    ]
                ),
            ]
        ),
    )


def _dispatch(req: Request, authenticated: List[bool]) -> Response:
    """Handle one request the way the production dispatcher would.

    ``authenticated`` is a one-element list used as a mutable cell
    (so the closure-style state survives across calls without a class
    wrapper).
    """
    cmd = req.command
    if cmd == "HELLO":
        authenticated[0] = True
        return _build_hello_response(req.id)
    if cmd == "PING":
        return Response.ok(req.id, VectorizerValue.str_("PONG"))
    if not authenticated[0]:
        return Response.err(
            req.id, f"authentication required: send HELLO first ({cmd})"
        )
    if cmd == "collections.list":
        return Response.ok(
            req.id,
            VectorizerValue.array(
                [
                    VectorizerValue.str_("alpha-docs"),
                    VectorizerValue.str_("beta-source"),
                ]
            ),
        )
    if cmd == "collections.get_info":
        name = "unknown"
        if req.args:
            s = req.args[0].as_str()
            if s is not None:
                name = s
        return _build_collection_info_response(req.id, name)
    if cmd == "search.basic":
        return _build_search_basic_response(req.id)
    return Response.err(req.id, f"unknown command '{cmd}'")


def _serve_sync_connection(sock: socket.socket) -> None:
    authenticated = [False]
    try:
        while True:
            raw = read_frame_sync(sock)
            req_body = raw  # raw is the decoded msgpack body — a list
            assert isinstance(req_body, list)
            req = Request(
                id=int(req_body[0]),
                command=str(req_body[1]),
                args=[VectorizerValue.from_msgpack(a) for a in req_body[2]],
            )
            resp = _dispatch(req, authenticated)
            sock.sendall(encode_frame(resp.to_msgpack()))
    except (ConnectionError, OSError, ValueError):
        pass
    finally:
        try:
            sock.close()
        except OSError:
            pass


def _spawn_fake_server() -> tuple[str, threading.Event]:
    """Start a fake server on an ephemeral port and return its address.

    Returns ``(host_port, stop_event)``. The caller can ``stop_event.set()``
    to end the accept loop, but tests usually leak the daemon thread —
    process exit reaps it.
    """
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    listener.bind(("127.0.0.1", 0))
    listener.listen(8)
    listener.settimeout(0.5)
    host, port = listener.getsockname()
    address = f"{host}:{port}"
    stop = threading.Event()

    def accept_loop() -> None:
        while not stop.is_set():
            try:
                conn, _ = listener.accept()
            except (socket.timeout, OSError):
                continue
            t = threading.Thread(
                target=_serve_sync_connection, args=(conn,), daemon=True
            )
            t.start()
        try:
            listener.close()
        except OSError:
            pass

    threading.Thread(target=accept_loop, daemon=True).start()
    # Tiny grace period so the listener is definitely accepting.
    time.sleep(0.02)
    return address, stop


@pytest.fixture
def fake_server() -> str:
    address, _stop = _spawn_fake_server()
    return address


# ─────────────────────────────────────────────────────────────────────────────
# Sync client tests
# ─────────────────────────────────────────────────────────────────────────────


class TestSyncRpcClient:
    def test_hello_then_ping_then_typed_commands(self, fake_server: str) -> None:
        with RpcClient.connect(fake_server) as client:
            # PING is auth-exempt per wire spec § 4.
            assert client.ping() == "PONG"

            hello = client.hello(HelloPayload(client_name="rpc-integration-test"))
            assert hello.authenticated is True
            assert hello.admin is True
            assert hello.protocol_version == 1
            assert hello.server_version == "test-fixture/0.0.0"
            assert "collections.list" in hello.capabilities

            cols = client.list_collections()
            assert cols == ["alpha-docs", "beta-source"]

            info = client.get_collection_info("alpha-docs")
            assert info.name == "alpha-docs"
            assert info.vector_count == 42
            assert info.dimension == 384
            assert info.metric == "Cosine"

            hits = client.search_basic("alpha-docs", "anything", 10)
            assert len(hits) == 2
            assert hits[0].id == "vec-0"
            assert abs(hits[0].score - 0.95) < 1e-9
            assert hits[0].payload == '{"title":"hit one"}'
            assert hits[1].id == "vec-1"
            assert hits[1].payload is None

    def test_data_plane_call_before_hello_is_rejected_locally(
        self, fake_server: str
    ) -> None:
        with RpcClient.connect(fake_server) as client:
            with pytest.raises(RpcNotAuthenticated):
                client.list_collections()

    def test_concurrent_calls_on_one_connection_are_demultiplexed_by_id(
        self, fake_server: str
    ) -> None:
        # Fire 16 list_collections from different threads on the same
        # client. If demuxing were broken, calls would either hang or
        # deliver the wrong payload.
        with RpcClient.connect(fake_server) as client:
            client.hello(HelloPayload(client_name="concurrent-test"))

            results: List[Optional[List[str]]] = [None] * 16
            errors: List[Optional[BaseException]] = [None] * 16

            def worker(idx: int) -> None:
                try:
                    results[idx] = client.list_collections()
                except BaseException as e:  # noqa: BLE001 — propagate to assertion
                    errors[idx] = e

            threads = [
                threading.Thread(target=worker, args=(i,)) for i in range(16)
            ]
            for t in threads:
                t.start()
            for t in threads:
                t.join(timeout=5.0)

            for i, err in enumerate(errors):
                assert err is None, f"thread {i} raised {err!r}"
            for i, cols in enumerate(results):
                assert cols == ["alpha-docs", "beta-source"], (
                    f"thread {i} got {cols!r}"
                )

    def test_connect_url_accepts_vectorizer_scheme(
        self, fake_server: str
    ) -> None:
        url = f"vectorizer://{fake_server}"
        with RpcClient.connect_url(url) as client:
            assert client.ping() == "PONG"

    def test_connect_url_rejects_http_scheme_with_clear_error(self) -> None:
        with pytest.raises(RpcServerError) as exc_info:
            RpcClient.connect_url("http://localhost:15002")
        msg = str(exc_info.value)
        assert "REST URL" in msg
        assert "HTTP client" in msg


# ─────────────────────────────────────────────────────────────────────────────
# Async client tests
# ─────────────────────────────────────────────────────────────────────────────


class TestAsyncRpcClient:
    @pytest.mark.asyncio
    async def test_hello_then_ping_then_typed_commands(self, fake_server: str) -> None:
        async with await AsyncRpcClient.connect(fake_server) as client:
            assert await client.ping() == "PONG"

            hello = await client.hello(
                HelloPayload(client_name="async-rpc-integration-test")
            )
            assert hello.authenticated is True
            assert hello.admin is True
            assert hello.protocol_version == 1
            assert "collections.list" in hello.capabilities

            cols = await client.list_collections()
            assert cols == ["alpha-docs", "beta-source"]

            info = await client.get_collection_info("alpha-docs")
            assert info.name == "alpha-docs"
            assert info.dimension == 384

            hits = await client.search_basic("alpha-docs", "q", 5)
            assert len(hits) == 2
            assert hits[0].id == "vec-0"

    @pytest.mark.asyncio
    async def test_data_plane_call_before_hello_is_rejected_locally(
        self, fake_server: str
    ) -> None:
        async with await AsyncRpcClient.connect(fake_server) as client:
            with pytest.raises(RpcNotAuthenticated):
                await client.list_collections()

    @pytest.mark.asyncio
    async def test_concurrent_calls_demultiplexed_by_id(self, fake_server: str) -> None:
        async with await AsyncRpcClient.connect(fake_server) as client:
            await client.hello(HelloPayload(client_name="async-concurrent"))
            results = await asyncio.gather(
                *[client.list_collections() for _ in range(16)]
            )
            assert all(r == ["alpha-docs", "beta-source"] for r in results)

    @pytest.mark.asyncio
    async def test_connect_url_accepts_vectorizer_scheme(
        self, fake_server: str
    ) -> None:
        url = f"vectorizer://{fake_server}"
        async with await AsyncRpcClient.connect_url(url) as client:
            assert await client.ping() == "PONG"

    @pytest.mark.asyncio
    async def test_connect_url_rejects_http_scheme(self) -> None:
        with pytest.raises(RpcServerError, match="REST URL"):
            await AsyncRpcClient.connect_url("http://localhost:15002")
