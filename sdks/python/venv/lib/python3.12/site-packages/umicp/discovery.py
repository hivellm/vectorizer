"""UMICP service discovery."""

from typing import Dict, List, Optional, Set
from dataclasses import dataclass, field
from datetime import datetime, timedelta
import asyncio


@dataclass
class ServiceInfo:
    """Service information."""

    id: str
    name: str
    version: str
    capabilities: Dict[str, any] = field(default_factory=dict)
    metadata: Dict[str, any] = field(default_factory=dict)
    registered_at: datetime = field(default_factory=datetime.utcnow)
    last_seen: datetime = field(default_factory=datetime.utcnow)
    health_status: str = "healthy"


class ServiceDiscovery:
    """Service discovery and registration."""

    def __init__(self, stale_timeout_seconds: int = 300) -> None:
        """Initialize service discovery.

        Args:
            stale_timeout_seconds: Timeout for stale services
        """
        self._services: Dict[str, ServiceInfo] = {}
        self._stale_timeout = timedelta(seconds=stale_timeout_seconds)
        self._lock = asyncio.Lock()
        self._cleanup_task: Optional[asyncio.Task] = None

    async def start(self) -> None:
        """Start discovery service."""
        if not self._cleanup_task:
            self._cleanup_task = asyncio.create_task(self._cleanup_loop())

    async def stop(self) -> None:
        """Stop discovery service."""
        if self._cleanup_task:
            self._cleanup_task.cancel()
            try:
                await self._cleanup_task
            except asyncio.CancelledError:
                pass
            self._cleanup_task = None

    async def register(self, service: ServiceInfo) -> None:
        """Register service.

        Args:
            service: Service information
        """
        async with self._lock:
            self._services[service.id] = service

    async def unregister(self, service_id: str) -> bool:
        """Unregister service.

        Args:
            service_id: Service ID

        Returns:
            True if service was removed
        """
        async with self._lock:
            return self._services.pop(service_id, None) is not None

    async def find_by_id(self, service_id: str) -> Optional[ServiceInfo]:
        """Find service by ID.

        Args:
            service_id: Service ID

        Returns:
            Service info or None
        """
        async with self._lock:
            return self._services.get(service_id)

    async def find_by_capability(self, capability: str, value: any = None) -> List[ServiceInfo]:
        """Find services by capability.

        Args:
            capability: Capability key
            value: Optional capability value

        Returns:
            List of matching services
        """
        async with self._lock:
            results = []
            for service in self._services.values():
                if capability in service.capabilities:
                    if value is None or service.capabilities[capability] == value:
                        results.append(service)
            return results

    async def list_all(self) -> List[ServiceInfo]:
        """List all services.

        Returns:
            List of all services
        """
        async with self._lock:
            return list(self._services.values())

    async def update_health(self, service_id: str, status: str) -> bool:
        """Update service health status.

        Args:
            service_id: Service ID
            status: Health status

        Returns:
            True if updated
        """
        async with self._lock:
            service = self._services.get(service_id)
            if service:
                service.health_status = status
                service.last_seen = datetime.utcnow()
                return True
            return False

    async def _cleanup_loop(self) -> None:
        """Background cleanup of stale services."""
        while True:
            try:
                await asyncio.sleep(60)  # Check every minute
                await self._remove_stale_services()
            except asyncio.CancelledError:
                break

    async def _remove_stale_services(self) -> None:
        """Remove services that haven't been seen recently."""
        now = datetime.utcnow()
        async with self._lock:
            stale_ids = [
                service_id
                for service_id, service in self._services.items()
                if now - service.last_seen > self._stale_timeout
            ]
            for service_id in stale_ids:
                del self._services[service_id]

