"""MockTransport regression guard for the split SDK.

Purpose
-------
``phase4_split-sdk-python-client`` splits the flat ``VectorizerClient``
into per-surface modules (collections, vectors, search, graph, admin,
auth) that must talk to a generic :class:`vectorizer.Transport`, never
directly to ``aiohttp`` / ``httpx``. The forthcoming
``phase6_sdk-python-rpc`` work will plug an RPC transport into those
same surface modules by subclassing the same ABC.

These tests construct a :class:`MockTransport` subclass of
:class:`vectorizer.Transport`, wire every surface client on top of it,
and assert that:

1. Each surface client accepts the generic ABC without requiring the
   concrete :class:`vectorizer.RestTransport`.
2. Methods that go through the canonical ``transport.get/post/...``
   path (the contract we are committing to for the RPC transport)
   actually route via the mock, with the expected path + payload.

If this file fails in CI, the RPC transport also cannot plug in —
that's the whole point of the guard.
"""

from __future__ import annotations

import os
import sys
import unittest
from typing import Any, Dict, List, Optional, Tuple

sys.path.insert(0, os.path.dirname(os.path.dirname(__file__)))

from vectorizer import (  # noqa: E402
    AdminClient,
    AuthClient,
    CollectionsClient,
    GraphClient,
    SearchClient,
    Transport,
    TransportRouter,
    VectorsClient,
    VectorizerClient,
)
from vectorizer._base import AuthState  # noqa: E402


class MockTransport(Transport):
    """Minimal in-memory transport; captures calls and returns canned data."""

    def __init__(self) -> None:
        self.base_url = "mock://test"
        self.calls: List[Tuple[str, str, Optional[Any]]] = []
        self.responses: Dict[Tuple[str, str], Any] = {}
        self.default_response: Any = {}
        self.closed = False

    def set_response(self, method: str, path: str, body: Any) -> None:
        self.responses[(method.upper(), path)] = body

    async def _dispatch(self, method: str, path: str, data: Optional[Any] = None) -> Any:
        self.calls.append((method.upper(), path, data))
        return self.responses.get((method.upper(), path), self.default_response)

    async def get(self, path: str) -> Any:
        return await self._dispatch("GET", path)

    async def post(self, path: str, data: Optional[Any] = None) -> Any:
        return await self._dispatch("POST", path, data)

    async def put(self, path: str, data: Optional[Any] = None) -> Any:
        return await self._dispatch("PUT", path, data)

    async def delete(self, path: str) -> Any:
        return await self._dispatch("DELETE", path)

    async def close(self) -> None:
        self.closed = True


class TestTransportAbcContract(unittest.TestCase):
    """MockTransport is a legitimate subclass of the SDK's Transport ABC."""

    def test_mock_transport_is_a_transport(self) -> None:
        mock = MockTransport()
        self.assertIsInstance(mock, Transport)

    def test_transport_abc_refuses_abstract_instantiation(self) -> None:
        with self.assertRaises(TypeError):
            Transport()  # type: ignore[abstract]


class TestSurfaceClientsAcceptAnyTransport(unittest.TestCase):
    """Each per-surface client constructs with a generic Transport."""

    def _clients(self, mock: MockTransport):
        router = TransportRouter(primary=mock)
        auth = AuthState(api_key="sk-mock")
        common = dict(base_url="mock://test", router=router, auth=auth)
        return [
            ("collections", CollectionsClient(mock, **common)),
            ("vectors", VectorsClient(mock, **common)),
            ("search", SearchClient(mock, **common)),
            ("graph", GraphClient(mock, **common)),
            ("admin", AdminClient(mock, **common)),
            ("auth", AuthClient(mock, api_key="sk-mock", **common)),
        ]

    def test_every_sub_client_accepts_transport_abc(self) -> None:
        mock = MockTransport()
        for name, client in self._clients(mock):
            self.assertIsInstance(
                client._transport, Transport, f"{name} is not hooked to a Transport"
            )
            self.assertIs(
                client._transport, mock, f"{name} lost its transport reference"
            )
            self.assertEqual(client.base_url, "mock://test")

    def test_router_read_write_use_mock(self) -> None:
        mock = MockTransport()
        router = TransportRouter(primary=mock)
        self.assertIs(router.write_transport(), mock)
        self.assertIs(router.read_transport(), mock)

    def test_auth_client_exposes_header(self) -> None:
        mock = MockTransport()
        auth = AuthClient(mock, api_key="sk-1")
        self.assertEqual(auth.headers(), {"Authorization": "Bearer sk-1"})
        auth.set_api_key(None)
        self.assertEqual(auth.headers(), {})


class TestCollectionsSurfaceRoutesThroughMock(unittest.IsolatedAsyncioTestCase):
    """The canonical ``transport.get/post/delete`` path works over MockTransport."""

    def _make(self, mock: MockTransport) -> CollectionsClient:
        router = TransportRouter(primary=mock)
        return CollectionsClient(mock, base_url="mock://test", router=router)

    async def test_list_collections_uses_transport(self) -> None:
        mock = MockTransport()
        mock.set_response("GET", "/collections", {"collections": []})
        client = self._make(mock)
        result = await client.list_collections()
        self.assertEqual(result, [])
        self.assertIn(("GET", "/collections", None), mock.calls)

    async def test_create_collection_uses_transport(self) -> None:
        mock = MockTransport()
        mock.set_response(
            "POST",
            "/collections",
            {
                "name": "demo",
                "dimension": 384,
                "vector_count": 0,
                "similarity_metric": "cosine",
            },
        )
        client = self._make(mock)
        info = await client.create_collection("demo", dimension=384)
        self.assertEqual(info.name, "demo")
        self.assertEqual(info.dimension, 384)
        self.assertTrue(
            any(call[0] == "POST" and call[1] == "/collections" for call in mock.calls)
        )

    async def test_delete_collection_uses_transport(self) -> None:
        mock = MockTransport()
        client = self._make(mock)
        ok = await client.delete_collection("demo")
        self.assertTrue(ok)
        self.assertIn(("DELETE", "/collections/demo", None), mock.calls)


class TestAdminSurfaceRoutesThroughMock(unittest.IsolatedAsyncioTestCase):
    """Admin.health_check is the canonical read — must travel the ABC."""

    async def test_health_check_uses_transport(self) -> None:
        mock = MockTransport()
        mock.set_response("GET", "/health", {"status": "healthy"})
        router = TransportRouter(primary=mock)
        admin = AdminClient(mock, base_url="mock://test", router=router)
        result = await admin.health_check()
        self.assertEqual(result, {"status": "healthy"})
        self.assertIn(("GET", "/health", None), mock.calls)


class TestFacadeWiresAllSubClientsToSameRouter(unittest.TestCase):
    """VectorizerClient composes sub-clients; they share one router."""

    def test_sub_clients_are_wired(self) -> None:
        client = VectorizerClient(base_url="http://localhost:15002")
        self.assertIsInstance(client.collections, CollectionsClient)
        self.assertIsInstance(client.vectors, VectorsClient)
        self.assertIsInstance(client.search, SearchClient)
        self.assertIsInstance(client.graph, GraphClient)
        self.assertIsInstance(client.admin, AdminClient)
        self.assertIsInstance(client.auth, AuthClient)

    def test_flat_api_delegates_to_sub_client(self) -> None:
        """``client.list_collections`` resolves to ``client.collections.list_collections``."""
        client = VectorizerClient(base_url="http://localhost:15002")
        self.assertEqual(
            client.list_collections.__func__,
            client.collections.list_collections.__func__,
        )
        self.assertEqual(
            client.health_check.__func__,
            client.admin.health_check.__func__,
        )
        self.assertEqual(
            client.find_graph_path.__func__,
            client.graph.find_graph_path.__func__,
        )


if __name__ == "__main__":
    unittest.main()
