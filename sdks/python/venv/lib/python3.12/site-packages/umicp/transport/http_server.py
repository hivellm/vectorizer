"""HTTP/2 server implementation."""

import json
from typing import Optional, Callable, Awaitable, Dict
from datetime import datetime
from aiohttp import web
from aiohttp.web import middleware

from umicp.envelope import Envelope
from umicp.types import TransportStats
from umicp.events import EventEmitter
from umicp.error import TransportError


class HttpServer:
    """Async HTTP/2 server with enhanced features."""

    def __init__(
        self,
        host: str = "0.0.0.0",
        port: int = 8080,
        cors: bool = True,
        compression: bool = True,
        max_request_size: int = 100 * 1024 * 1024,  # 100MB
    ) -> None:
        """Initialize HTTP server.

        Args:
            host: Host to bind
            port: Port to bind
            cors: Enable CORS headers
            compression: Enable response compression
            max_request_size: Maximum request size in bytes
        """
        self.host = host
        self.port = port
        self.cors = cors
        self.compression = compression
        self.max_request_size = max_request_size
        self.stats = TransportStats()
        self.events = EventEmitter()
        self.started_at = datetime.now()

        self._message_handler: Optional[Callable[[Envelope], Awaitable[Optional[Envelope]]]] = None
        self._runner: Optional[web.AppRunner] = None

        # Create application with middlewares
        middlewares = []
        if cors:
            middlewares.append(self._cors_middleware)

        self.app = web.Application(
            middlewares=middlewares,
            client_max_size=max_request_size
        )

        # Setup routes
        self.app.router.add_get("/", self._handle_root)
        self.app.router.add_get("/health", self._handle_health)
        self.app.router.add_get("/stats", self._handle_stats)
        self.app.router.add_post("/message", self._handle_message)
        self.app.router.add_post("/envelope", self._handle_message)  # Alias

    @middleware
    async def _cors_middleware(self, request: web.Request, handler):
        """CORS middleware."""
        response = await handler(request)
        response.headers["Access-Control-Allow-Origin"] = "*"
        response.headers["Access-Control-Allow-Methods"] = "GET, POST, OPTIONS"
        response.headers["Access-Control-Allow-Headers"] = "Content-Type"
        return response

    def set_message_handler(
        self, handler: Callable[[Envelope], Awaitable[Optional[Envelope]]]
    ) -> None:
        """Set message handler callback.

        Args:
            handler: Async function(envelope) that returns optional response envelope
        """
        self._message_handler = handler

    async def _handle_root(self, request: web.Request) -> web.Response:
        """Handle root endpoint."""
        return web.Response(
            text="UMICP HTTP Server",
            content_type="text/plain"
        )

    async def _handle_health(self, request: web.Request) -> web.Response:
        """Handle health check endpoint."""
        uptime = (datetime.now() - self.started_at).total_seconds()
        health_data = {
            "status": "healthy",
            "uptime_seconds": uptime,
            "version": "0.1.3"
        }
        return web.json_response(health_data)

    async def _handle_stats(self, request: web.Request) -> web.Response:
        """Handle stats endpoint."""
        stats_data = {
            "messages_sent": self.stats.messages_sent,
            "messages_received": self.stats.messages_received,
            "bytes_sent": self.stats.bytes_sent,
            "bytes_received": self.stats.bytes_received,
            "active_connections": self.stats.active_connections,
            "total_connections": self.stats.total_connections,
        }
        return web.json_response(stats_data)

    async def _handle_message(self, request: web.Request) -> web.Response:
        """Handle incoming message.

        Args:
            request: HTTP request

        Returns:
            HTTP response with optional envelope
        """
        try:
            # Parse request
            data = await request.json()
            self.stats.messages_received += 1
            self.stats.bytes_received += len(await request.read())

            # Create envelope
            envelope = Envelope.from_dict(data)

            # Emit event
            self.events.emit("message_received", {"envelope": envelope})

            # Call handler
            response_envelope = None
            if self._message_handler:
                try:
                    response_envelope = await self._message_handler(envelope)
                except Exception as e:
                    self.events.emit("error", {"error": str(e)})
                    raise web.HTTPInternalServerError(text=str(e))

            # Return response
            if response_envelope:
                response_data = response_envelope.to_dict()
                self.stats.messages_sent += 1
                response_json = json.dumps(response_data)
                self.stats.bytes_sent += len(response_json)
                return web.json_response(response_data)
            else:
                return web.json_response({"status": "ok"})

        except json.JSONDecodeError as e:
            self.events.emit("error", {"error": f"Invalid JSON: {e}"})
            raise web.HTTPBadRequest(text=f"Invalid JSON: {e}")
        except Exception as e:
            self.events.emit("error", {"error": str(e)})
            raise web.HTTPInternalServerError(text=str(e))

    async def start(self) -> None:
        """Start server."""
        self._runner = web.AppRunner(
            self.app,
            handle_signals=False
        )
        await self._runner.setup()

        site = web.TCPSite(
            self._runner,
            self.host,
            self.port
        )
        await site.start()

        self.events.emit("server_started", {"host": self.host, "port": self.port})

    async def stop(self) -> None:
        """Stop server."""
        if self._runner:
            await self._runner.cleanup()
            self._runner = None

        self.events.emit("server_stopped")

    def get_stats(self) -> TransportStats:
        """Get server statistics."""
        return self.stats

