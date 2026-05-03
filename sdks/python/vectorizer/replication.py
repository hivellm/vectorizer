"""Replication surface.

Covers the ``/replication/*`` REST endpoints (status, configuration,
statistics, replica listing) and the ``/cluster/*`` admin endpoints
added in phase15 (failover, resync, add-peer, rebalance).
"""

from __future__ import annotations

import logging
from typing import Any, Dict, List, Optional

try:
    from ..models import (
        AddPeerRequest,
        FailoverReport,
        PeerInfo,
        RebalanceJob,
        ReplicaInfo,
        ReplicationConfig,
        ReplicationStats,
        ReplicationStatus,
        ResyncJob,
    )
except ImportError:  # pragma: no cover
    from models import (  # type: ignore[import-not-found]
        AddPeerRequest,
        FailoverReport,
        PeerInfo,
        RebalanceJob,
        ReplicaInfo,
        ReplicationConfig,
        ReplicationStats,
        ReplicationStatus,
        ResyncJob,
    )

from ._base import _ApiBase

logger = logging.getLogger(__name__)


class ReplicationClient(_ApiBase):
    """Replication status, configuration, stats, replica listing, and cluster admin."""

    async def get_replication_status(self) -> Dict[str, Any]:
        """Get the current replication status and role of this node.

        Calls ``GET /replication/status``.

        Returns:
            Raw replication-status dict with ``role``, ``enabled``, optional
            ``stats`` and ``replicas`` keys.
        """
        return await self._transport.get("/replication/status")

    async def configure_replication(self, config: ReplicationConfig) -> None:
        """Configure this node's replication role and parameters.

        Calls ``POST /replication/configure``. A server restart is
        required for the new config to take effect.

        Args:
            config: :class:`ReplicationConfig` with role, addresses, and
                optional heartbeat_interval / log_size.
        """
        payload: Dict[str, Any] = {"role": config.role}
        if config.bind_address is not None:
            payload["bind_address"] = config.bind_address
        if config.master_address is not None:
            payload["master_address"] = config.master_address
        if config.heartbeat_interval is not None:
            payload["heartbeat_interval"] = config.heartbeat_interval
        if config.log_size is not None:
            payload["log_size"] = config.log_size
        await self._transport.post("/replication/configure", data=payload)

    async def get_replication_stats(self) -> Dict[str, Any]:
        """Get raw replication statistics for the active replication node.

        Calls ``GET /replication/stats``. Returns an error when
        replication is not enabled on this node.

        Returns:
            Raw stats dict with offset / lag / bytes counters.
        """
        return await self._transport.get("/replication/stats")

    async def list_replicas(self) -> List[Dict[str, Any]]:
        """List the replica nodes connected to this master.

        Calls ``GET /replication/replicas``. Only available on master
        nodes; returns an error otherwise.

        Returns:
            List of raw replica-info dicts with replica_id, host, port, status.
        """
        data = await self._transport.get("/replication/replicas")
        if isinstance(data, dict):
            return list(data.get("replicas", []))
        return []

    # ── phase15 cluster admin ─────────────────────────────────────────────────

    async def cluster_failover(self, replica_id: str) -> FailoverReport:
        """Trigger a failover — promote a replica to primary.

        Calls ``POST /cluster/failover`` with ``{replica_id}``.
        The server returns 409 when the replica's WAL lag exceeds the threshold.

        Args:
            replica_id: ID of the replica to promote.

        Returns:
            :class:`FailoverReport` with promotion offsets and residual lag.
        """
        data = await self._transport.post(
            "/cluster/failover",
            data={"replica_id": replica_id},
        )
        return FailoverReport.from_dict(data if isinstance(data, dict) else {})

    async def cluster_resync_replica(self, replica_id: str) -> ResyncJob:
        """Force a full resync on a replica.

        Calls ``POST /cluster/replicas/{id}/resync`` with an empty body.

        Args:
            replica_id: ID of the replica to resync.

        Returns:
            :class:`ResyncJob` with snapshot offset and full_snapshot flag.
        """
        data = await self._transport.post(
            f"/cluster/replicas/{replica_id}/resync",
            data={},
        )
        return ResyncJob.from_dict(data if isinstance(data, dict) else {})

    async def cluster_add_peer(self, request: AddPeerRequest) -> PeerInfo:
        """Add a peer to the cluster.

        Calls ``POST /cluster/peers`` with ``{address, role}``.

        Args:
            request: :class:`AddPeerRequest` with address and optional role.

        Returns:
            :class:`PeerInfo` with the server-assigned node_id, address, and role.
        """
        payload: Dict[str, Any] = {"address": request.address, "role": request.role}
        data = await self._transport.post("/cluster/peers", data=payload)
        return PeerInfo.from_dict(data if isinstance(data, dict) else {})

    async def cluster_rebalance(self) -> RebalanceJob:
        """Trigger a shard rebalance across all active cluster nodes.

        Calls ``POST /cluster/rebalance`` with an empty body.
        The server returns 400 when fewer than 2 active nodes are present or
        a rebalance is already running.

        Returns:
            :class:`RebalanceJob` with job_id, status, and shard counts.
        """
        data = await self._transport.post("/cluster/rebalance", data={})
        return RebalanceJob.from_dict(data if isinstance(data, dict) else {})

    async def cluster_rebalance_status(self) -> Optional[RebalanceJob]:
        """Return progress of the active (or last completed) rebalance job.

        Calls ``GET /cluster/rebalance/status``.

        Returns:
            :class:`RebalanceJob` if a job exists, or ``None`` when no
            rebalance has been triggered on this node.
        """
        data = await self._transport.get("/cluster/rebalance/status")
        if isinstance(data, dict) and data.get("status") == "idle":
            return None
        return RebalanceJob.from_dict(data if isinstance(data, dict) else {})
