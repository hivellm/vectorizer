"""HTTP/2 client implementation."""

import httpx
from umicp.envelope import Envelope
from umicp.types import TransportStats


class HttpClient:
    """Async HTTP/2 client."""

    def __init__(self, base_url: str, path: str = "/message") -> None:
        """Initialize HTTP client.

        Args:
            base_url: Base URL of the server
            path: Endpoint path (default: "/message", vectorizer uses "/umicp")
        """
        self.base_url = base_url
        self.path = path
        self.stats = TransportStats()
        self._client = httpx.AsyncClient(http2=True)

    async def send(self, envelope: Envelope) -> None:
        """Send envelope via HTTP POST."""
        response = await self._client.post(
            f"{self.base_url}{self.path}",
            json=envelope.to_dict()
        )
        response.raise_for_status()
        self.stats.messages_sent += 1

    async def close(self) -> None:
        """Close client."""
        await self._client.aclose()

