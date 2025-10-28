"""UMICP event system."""

from enum import Enum
from typing import Any, Callable, Dict, List, Optional, Awaitable
from dataclasses import dataclass
import asyncio


class EventType(str, Enum):
    """Event types."""

    MESSAGE = "message"
    PEER_CONNECT = "peer_connect"
    PEER_DISCONNECT = "peer_disconnect"
    ERROR = "error"
    HANDSHAKE_COMPLETE = "handshake_complete"
    CONNECTION_STATE_CHANGE = "connection_state_change"


@dataclass
class Event:
    """Event data."""

    type: EventType
    data: Any
    source: Optional[str] = None
    timestamp: Optional[float] = None


EventHandler = Callable[[Event], Awaitable[None]]


class EventEmitter:
    """Async event emitter."""

    def __init__(self) -> None:
        """Initialize event emitter."""
        self._handlers: Dict[EventType, List[EventHandler]] = {}
        self._lock = asyncio.Lock()

    async def on(self, event_type: EventType, handler: EventHandler) -> None:
        """Register event handler.

        Args:
            event_type: Event type to listen for
            handler: Async callback function
        """
        async with self._lock:
            if event_type not in self._handlers:
                self._handlers[event_type] = []
            self._handlers[event_type].append(handler)

    async def off(self, event_type: EventType, handler: EventHandler) -> None:
        """Unregister event handler.

        Args:
            event_type: Event type
            handler: Handler to remove
        """
        async with self._lock:
            if event_type in self._handlers:
                self._handlers[event_type] = [
                    h for h in self._handlers[event_type] if h != handler
                ]

    async def emit(self, event: Event) -> None:
        """Emit event to all handlers.

        Args:
            event: Event to emit
        """
        async with self._lock:
            handlers = self._handlers.get(event.type, []).copy()

        # Call handlers concurrently
        if handlers:
            await asyncio.gather(*[handler(event) for handler in handlers], return_exceptions=True)

    def remove_all_listeners(self, event_type: Optional[EventType] = None) -> None:
        """Remove all event listeners.

        Args:
            event_type: Specific event type, or None for all
        """
        if event_type:
            self._handlers.pop(event_type, None)
        else:
            self._handlers.clear()

