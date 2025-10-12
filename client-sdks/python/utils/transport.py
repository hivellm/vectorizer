"""
Transport abstraction layer for Vectorizer client.

Supports multiple transport protocols:
- HTTP/HTTPS (default)
- UMICP (Universal Messaging and Inter-process Communication Protocol)
"""

from typing import Optional, Dict, Any, Protocol as TypingProtocol
from enum import Enum
from urllib.parse import urlparse


class TransportProtocol(str, Enum):
    """Transport protocol enum."""
    HTTP = "http"
    UMICP = "umicp"


class Transport(TypingProtocol):
    """Transport protocol interface."""
    
    async def get(self, path: str) -> Any:
        """Make a GET request."""
        ...
    
    async def post(self, path: str, data: Optional[Dict[str, Any]] = None) -> Any:
        """Make a POST request."""
        ...
    
    async def put(self, path: str, data: Optional[Dict[str, Any]] = None) -> Any:
        """Make a PUT request."""
        ...
    
    async def delete(self, path: str) -> Any:
        """Make a DELETE request."""
        ...


class TransportFactory:
    """Factory for creating transport clients."""
    
    @staticmethod
    def create(protocol: TransportProtocol, config: Dict[str, Any]) -> Transport:
        """
        Create a transport client based on protocol.
        
        Args:
            protocol: Transport protocol to use
            config: Configuration dict
            
        Returns:
            Transport client instance
        """
        if protocol == TransportProtocol.HTTP:
            # Import here to avoid circular dependency
            from .http_client import HTTPClient
            return HTTPClient(**config)
        
        elif protocol == TransportProtocol.UMICP:
            from .umicp_client import UMICPClient
            return UMICPClient(**config)
        
        else:
            raise ValueError(f"Unsupported protocol: {protocol}")


def parse_connection_string(connection_string: str, api_key: Optional[str] = None) -> tuple:
    """
    Parse a connection string into protocol and configuration.
    
    Examples:
        - "http://localhost:15002" -> HTTP transport
        - "https://api.example.com" -> HTTPS transport
        - "umicp://localhost:15003" -> UMICP transport
    
    Args:
        connection_string: Connection URI
        api_key: Optional API key
        
    Returns:
        Tuple of (protocol, config_dict)
    """
    parsed = urlparse(connection_string)
    
    if parsed.scheme in ("http", "https"):
        return (
            TransportProtocol.HTTP,
            {
                "base_url": f"{parsed.scheme}://{parsed.netloc}",
                "api_key": api_key,
            }
        )
    
    elif parsed.scheme == "umicp":
        port = parsed.port or 15003
        return (
            TransportProtocol.UMICP,
            {
                "host": parsed.hostname,
                "port": port,
                "api_key": api_key,
            }
        )
    
    else:
        raise ValueError(f"Unsupported protocol in connection string: {parsed.scheme}")

