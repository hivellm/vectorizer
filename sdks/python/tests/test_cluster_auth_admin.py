"""Unit tests for the SDK phase15 cluster + auth admin surface:
``cluster_failover``, ``cluster_resync_replica``, ``cluster_add_peer``,
``cluster_rebalance``, ``cluster_rebalance_status`` on
:class:`ReplicationClient`, and ``rotate_api_key``,
``create_scoped_api_key``, ``introspect_token``, ``list_audit_log``
on :class:`AuthClient`.

Wire shapes are validated against the canonical Rust implementation at
``sdks/rust/src/client/replication.rs`` and
``sdks/rust/src/client/auth.rs``.
"""

from __future__ import annotations

import asyncio
import os
import sys
import unittest
from unittest.mock import AsyncMock, MagicMock

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from models import (  # type: ignore[import-not-found]
    AddPeerRequest,
    AuditEntry,
    AuditQuery,
    CreateScopedApiKeyRequest,
    FailoverReport,
    PeerInfo,
    RebalanceJob,
    ResyncJob,
    RotatedKey,
    TokenIntrospection,
    TokenScope,
)
from vectorizer.auth import AuthClient  # type: ignore[import-not-found]
from vectorizer.replication import ReplicationClient  # type: ignore[import-not-found]


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _make_replication_client() -> tuple[ReplicationClient, MagicMock]:
    """Return a ``ReplicationClient`` with a mock transport."""
    client = ReplicationClient.__new__(ReplicationClient)
    transport = MagicMock()
    transport.post = AsyncMock()
    transport.get = AsyncMock()
    transport.put = AsyncMock()
    transport.delete = AsyncMock()
    client._transport = transport  # type: ignore[attr-defined]
    return client, transport


def _make_auth_client() -> tuple[AuthClient, MagicMock]:
    """Return an ``AuthClient`` with a mock transport."""
    client = AuthClient.__new__(AuthClient)
    # Minimal auth state so headers() works without a real init
    from vectorizer._base import AuthState  # type: ignore[import-not-found]
    client._auth = AuthState()
    transport = MagicMock()
    transport.post = AsyncMock()
    transport.get = AsyncMock()
    transport.put = AsyncMock()
    transport.delete = AsyncMock()
    client._transport = transport  # type: ignore[attr-defined]
    return client, transport


def run(coro):
    return asyncio.get_event_loop().run_until_complete(coro)


# ---------------------------------------------------------------------------
# ReplicationClient — cluster admin
# ---------------------------------------------------------------------------


class TestClusterFailover(unittest.TestCase):
    def setUp(self):
        self.client, self.transport = _make_replication_client()

    def _server_response(self):
        return {
            "promoted_replica_id": "replica-1",
            "master_offset_at_promotion": 1000,
            "replica_offset_at_promotion": 999,
            "residual_lag_operations": 1,
        }

    def test_posts_to_cluster_failover(self):
        self.transport.post.return_value = self._server_response()
        run(self.client.cluster_failover("replica-1"))
        self.transport.post.assert_called_once_with(
            "/cluster/failover", data={"replica_id": "replica-1"}
        )

    def test_returns_failover_report(self):
        self.transport.post.return_value = self._server_response()
        result = run(self.client.cluster_failover("replica-1"))
        self.assertIsInstance(result, FailoverReport)
        self.assertEqual(result.promoted_replica_id, "replica-1")
        self.assertEqual(result.master_offset_at_promotion, 1000)
        self.assertEqual(result.residual_lag_operations, 1)


class TestClusterResyncReplica(unittest.TestCase):
    def setUp(self):
        self.client, self.transport = _make_replication_client()

    def _server_response(self):
        return {
            "replica_id": "replica-2",
            "snapshot_offset": 5000,
            "full_snapshot": True,
        }

    def test_posts_empty_body_to_correct_url(self):
        self.transport.post.return_value = self._server_response()
        run(self.client.cluster_resync_replica("replica-2"))
        self.transport.post.assert_called_once_with(
            "/cluster/replicas/replica-2/resync", data={}
        )

    def test_returns_resync_job(self):
        self.transport.post.return_value = self._server_response()
        result = run(self.client.cluster_resync_replica("replica-2"))
        self.assertIsInstance(result, ResyncJob)
        self.assertEqual(result.replica_id, "replica-2")
        self.assertEqual(result.snapshot_offset, 5000)
        self.assertTrue(result.full_snapshot)


class TestClusterAddPeer(unittest.TestCase):
    def setUp(self):
        self.client, self.transport = _make_replication_client()

    def _server_response(self, role="member"):
        return {"node_id": "peer-abc", "address": "10.0.0.2:15003", "role": role}

    def test_posts_address_and_role(self):
        self.transport.post.return_value = self._server_response()
        req = AddPeerRequest(address="10.0.0.2:15003", role="member")
        run(self.client.cluster_add_peer(req))
        self.transport.post.assert_called_once_with(
            "/cluster/peers", data={"address": "10.0.0.2:15003", "role": "member"}
        )

    def test_returns_peer_info(self):
        self.transport.post.return_value = self._server_response()
        result = run(self.client.cluster_add_peer(AddPeerRequest(address="10.0.0.2:15003")))
        self.assertIsInstance(result, PeerInfo)
        self.assertEqual(result.node_id, "peer-abc")
        self.assertEqual(result.role, "member")

    def test_observer_role(self):
        self.transport.post.return_value = self._server_response(role="observer")
        result = run(
            self.client.cluster_add_peer(
                AddPeerRequest(address="10.0.0.3:15003", role="observer")
            )
        )
        self.assertEqual(result.role, "observer")


class TestClusterRebalance(unittest.TestCase):
    def setUp(self):
        self.client, self.transport = _make_replication_client()

    def _server_response(self):
        return {
            "job_id": "job-xyz",
            "status": "running",
            "shards_to_move": 4,
            "shards_moved": 0,
            "message": "Rebalance started",
        }

    def test_posts_empty_body_to_cluster_rebalance(self):
        self.transport.post.return_value = self._server_response()
        run(self.client.cluster_rebalance())
        self.transport.post.assert_called_once_with("/cluster/rebalance", data={})

    def test_returns_rebalance_job(self):
        self.transport.post.return_value = self._server_response()
        result = run(self.client.cluster_rebalance())
        self.assertIsInstance(result, RebalanceJob)
        self.assertEqual(result.job_id, "job-xyz")
        self.assertEqual(result.status, "running")
        self.assertEqual(result.shards_to_move, 4)
        self.assertIsNone(result.last_checkpoint_node)


class TestClusterRebalanceStatus(unittest.TestCase):
    def setUp(self):
        self.client, self.transport = _make_replication_client()

    def test_gets_cluster_rebalance_status(self):
        self.transport.get.return_value = {
            "job_id": "job-xyz",
            "status": "completed",
            "shards_to_move": 4,
            "shards_moved": 4,
            "message": "done",
        }
        run(self.client.cluster_rebalance_status())
        self.transport.get.assert_called_once_with("/cluster/rebalance/status")

    def test_returns_rebalance_job_when_active(self):
        self.transport.get.return_value = {
            "job_id": "job-xyz",
            "status": "completed",
            "shards_to_move": 4,
            "shards_moved": 4,
            "message": "done",
        }
        result = run(self.client.cluster_rebalance_status())
        self.assertIsNotNone(result)
        self.assertEqual(result.status, "completed")
        self.assertEqual(result.shards_moved, 4)

    def test_returns_none_when_idle(self):
        self.transport.get.return_value = {
            "status": "idle",
            "message": "No rebalance has been triggered on this node",
        }
        result = run(self.client.cluster_rebalance_status())
        self.assertIsNone(result)


# ---------------------------------------------------------------------------
# AuthClient — auth admin
# ---------------------------------------------------------------------------


class TestRotateApiKey(unittest.TestCase):
    def setUp(self):
        self.client, self.transport = _make_auth_client()

    def _server_response(self):
        return {
            "old_key_id": "key-old",
            "new_key_id": "key-new",
            "new_token": "sk-new-token",
            "grace_until": 1714694400,
        }

    def test_posts_empty_body_to_rotate_url(self):
        self.transport.post.return_value = self._server_response()
        run(self.client.rotate_api_key("key-old"))
        self.transport.post.assert_called_once_with(
            "/auth/keys/key-old/rotate", data={}
        )

    def test_returns_rotated_key(self):
        self.transport.post.return_value = self._server_response()
        result = run(self.client.rotate_api_key("key-old"))
        self.assertIsInstance(result, RotatedKey)
        self.assertEqual(result.old_key_id, "key-old")
        self.assertEqual(result.new_key_id, "key-new")
        self.assertEqual(result.new_token, "sk-new-token")
        self.assertEqual(result.grace_until, 1714694400)


class TestCreateScopedApiKey(unittest.TestCase):
    def setUp(self):
        self.client, self.transport = _make_auth_client()

    def _server_response(self):
        return {
            "id": "key-scoped",
            "name": "scoped-key",
            "permissions": ["Read"],
            "api_key": "sk-scoped-abc",
            "created_at": 1714608000,
            "active": True,
            "warning": "Save this API key now! It will not be shown again.",
        }

    def test_posts_to_auth_keys_with_scopes(self):
        self.transport.post.return_value = self._server_response()
        req = CreateScopedApiKeyRequest(
            name="scoped-key",
            permissions=["Read"],
            scopes=[TokenScope(collection="my-col", permissions=["read", "write"])],
        )
        run(self.client.create_scoped_api_key(req))
        call_args = self.transport.post.call_args
        self.assertEqual(call_args[0][0], "/auth/keys")
        payload = call_args[1]["data"]
        self.assertEqual(payload["name"], "scoped-key")
        self.assertEqual(len(payload["scopes"]), 1)
        self.assertEqual(payload["scopes"][0]["collection"], "my-col")
        self.assertIn("write", payload["scopes"][0]["permissions"])

    def test_returns_api_key_with_token(self):
        self.transport.post.return_value = self._server_response()
        from models import ApiKey  # type: ignore[import-not-found]
        result = run(
            self.client.create_scoped_api_key(CreateScopedApiKeyRequest(name="scoped-key"))
        )
        self.assertIsInstance(result, ApiKey)
        self.assertEqual(result.id, "key-scoped")
        self.assertEqual(result.api_key, "sk-scoped-abc")

    def test_sends_empty_scopes_by_default(self):
        self.transport.post.return_value = self._server_response()
        run(self.client.create_scoped_api_key(CreateScopedApiKeyRequest(name="plain-key")))
        payload = self.transport.post.call_args[1]["data"]
        self.assertEqual(payload["scopes"], [])


class TestIntrospectToken(unittest.TestCase):
    def setUp(self):
        self.client, self.transport = _make_auth_client()

    def test_posts_token_to_introspect(self):
        self.transport.post.return_value = {"active": True, "sub": "user-1"}
        run(self.client.introspect_token("my-token"))
        self.transport.post.assert_called_once_with(
            "/auth/introspect", data={"token": "my-token"}
        )

    def test_returns_active_introspection(self):
        self.transport.post.return_value = {
            "active": True,
            "sub": "user-1",
            "exp": 9999999999,
            "username": "alice",
            "scope": "docs:read",
        }
        result = run(self.client.introspect_token("valid-jwt"))
        self.assertIsInstance(result, TokenIntrospection)
        self.assertTrue(result.active)
        self.assertEqual(result.sub, "user-1")
        self.assertEqual(result.username, "alice")
        self.assertEqual(result.scope, "docs:read")

    def test_returns_inactive_for_bad_token(self):
        self.transport.post.return_value = {"active": False}
        result = run(self.client.introspect_token("garbage"))
        self.assertFalse(result.active)
        self.assertIsNone(result.sub)
        self.assertIsNone(result.exp)


class TestListAuditLog(unittest.TestCase):
    def setUp(self):
        self.client, self.transport = _make_auth_client()

    def _server_response(self):
        return {
            "entries": [
                {
                    "actor": "admin",
                    "action": "rotate_api_key",
                    "target": "key-1",
                    "at": "2026-05-02T12:00:00Z",
                    "correlation_id": "corr-abc",
                },
                {
                    "actor": "admin",
                    "action": "create_api_key",
                    "target": "key-2",
                    "at": "2026-05-02T13:00:00Z",
                },
            ],
            "total": 2,
        }

    def test_gets_auth_audit_with_no_params(self):
        self.transport.get.return_value = self._server_response()
        run(self.client.list_audit_log())
        self.transport.get.assert_called_once_with("/auth/audit")

    def test_appends_query_params_when_provided(self):
        self.transport.get.return_value = {"entries": [], "total": 0}
        run(self.client.list_audit_log(AuditQuery(actor="admin", limit=10)))
        url = self.transport.get.call_args[0][0]
        self.assertIn("actor=admin", url)
        self.assertIn("limit=10", url)

    def test_returns_list_of_audit_entries(self):
        self.transport.get.return_value = self._server_response()
        result = run(self.client.list_audit_log())
        self.assertEqual(len(result), 2)
        self.assertIsInstance(result[0], AuditEntry)
        self.assertEqual(result[0].actor, "admin")
        self.assertEqual(result[0].action, "rotate_api_key")
        self.assertEqual(result[0].correlation_id, "corr-abc")
        self.assertIsNone(result[1].correlation_id)

    def test_returns_empty_list_when_entries_missing(self):
        self.transport.get.return_value = {"total": 0}
        result = run(self.client.list_audit_log())
        self.assertEqual(result, [])

    def test_applies_since_and_until_params(self):
        self.transport.get.return_value = {"entries": [], "total": 0}
        run(
            self.client.list_audit_log(
                AuditQuery(
                    since="2026-05-01T00:00:00Z",
                    until="2026-05-02T00:00:00Z",
                )
            )
        )
        url = self.transport.get.call_args[0][0]
        self.assertIn("since=", url)
        self.assertIn("until=", url)


# ---------------------------------------------------------------------------
# Dataclass round-trip tests (no transport needed)
# ---------------------------------------------------------------------------


class TestPhase15Dataclasses(unittest.TestCase):
    def test_failover_report_from_dict(self):
        d = {
            "promoted_replica_id": "r-1",
            "master_offset_at_promotion": 500,
            "replica_offset_at_promotion": 499,
            "residual_lag_operations": 1,
        }
        r = FailoverReport.from_dict(d)
        self.assertEqual(r.promoted_replica_id, "r-1")
        self.assertEqual(r.residual_lag_operations, 1)

    def test_resync_job_from_dict(self):
        d = {"replica_id": "r-2", "snapshot_offset": 1000, "full_snapshot": True}
        j = ResyncJob.from_dict(d)
        self.assertEqual(j.replica_id, "r-2")
        self.assertTrue(j.full_snapshot)

    def test_peer_info_from_dict(self):
        d = {"node_id": "p-1", "address": "10.0.0.1:15003", "role": "member"}
        p = PeerInfo.from_dict(d)
        self.assertEqual(p.node_id, "p-1")
        self.assertEqual(p.role, "member")

    def test_rebalance_job_from_dict_with_checkpoint(self):
        d = {
            "job_id": "j-1",
            "status": "completed",
            "shards_to_move": 3,
            "shards_moved": 3,
            "last_checkpoint_node": "node-3",
            "message": "done",
        }
        j = RebalanceJob.from_dict(d)
        self.assertEqual(j.last_checkpoint_node, "node-3")

    def test_rebalance_job_from_dict_without_checkpoint(self):
        d = {
            "job_id": "j-2",
            "status": "running",
            "shards_to_move": 2,
            "shards_moved": 0,
            "message": "started",
        }
        j = RebalanceJob.from_dict(d)
        self.assertIsNone(j.last_checkpoint_node)

    def test_rotated_key_from_dict(self):
        d = {
            "old_key_id": "old",
            "new_key_id": "new",
            "new_token": "sk-abc",
            "grace_until": 9999,
        }
        r = RotatedKey.from_dict(d)
        self.assertEqual(r.new_token, "sk-abc")
        self.assertEqual(r.grace_until, 9999)

    def test_token_introspection_active(self):
        d = {"active": True, "sub": "u-1", "exp": 9999, "username": "alice"}
        t = TokenIntrospection.from_dict(d)
        self.assertTrue(t.active)
        self.assertEqual(t.username, "alice")

    def test_token_introspection_inactive(self):
        d = {"active": False}
        t = TokenIntrospection.from_dict(d)
        self.assertFalse(t.active)
        self.assertIsNone(t.sub)

    def test_audit_entry_with_correlation_id(self):
        d = {
            "actor": "admin",
            "action": "rotate_api_key",
            "target": "k-1",
            "at": "2026-05-02T12:00:00Z",
            "correlation_id": "c-1",
        }
        e = AuditEntry.from_dict(d)
        self.assertEqual(e.correlation_id, "c-1")

    def test_audit_entry_without_correlation_id(self):
        d = {
            "actor": "admin",
            "action": "create_api_key",
            "target": "k-2",
            "at": "2026-05-02T13:00:00Z",
        }
        e = AuditEntry.from_dict(d)
        self.assertIsNone(e.correlation_id)

    def test_token_scope_defaults(self):
        s = TokenScope(collection="col-1")
        self.assertEqual(s.permissions, [])

    def test_add_peer_request_default_role(self):
        r = AddPeerRequest(address="1.2.3.4:15003")
        self.assertEqual(r.role, "member")

    def test_create_scoped_api_key_request_defaults(self):
        req = CreateScopedApiKeyRequest(name="k")
        self.assertEqual(req.permissions, [])
        self.assertEqual(req.scopes, [])
        self.assertIsNone(req.expires_in)

    def test_audit_query_all_none_by_default(self):
        q = AuditQuery()
        self.assertIsNone(q.actor)
        self.assertIsNone(q.limit)


if __name__ == "__main__":
    unittest.main()
