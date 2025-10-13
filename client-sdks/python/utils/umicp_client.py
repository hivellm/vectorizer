"""
UMICP client utility using the official umicp-python package.

Wrapper around UMICP client for Vectorizer API requests.
"""

from typing import Optional, Dict, Any
import json
import asyncio

try:
    from umicp import UMICPClient as BaseUMICPClient, Envelope, Priority
    UMICP_AVAILABLE = True
except ImportError:
    UMICP_AVAILABLE = False
    BaseUMICPClient = None
    Envelope = None
    Priority = None

try:
    from ..exceptions import NetworkError, ServerError, AuthenticationError
except ImportError:
    from exceptions import NetworkError, ServerError, AuthenticationError


class UMICPClient:
    """UMICP transport client for Vectorizer."""
    
    def __init__(
        self,
        host: str = "localhost",
        port: int = 15003,
        api_key: Optional[str] = None,
        timeout: int = 30
    ):
        """
        Initialize UMICP client.
        
        Args:
            host: Server hostname
            port: Server port
            api_key: API key for authentication
            timeout: Request timeout in seconds
        """
        if not UMICP_AVAILABLE:
            raise ImportError(
                "umicp-python is not installed. "
                "Install it with: pip install umicp-python"
            )
        
        self.host = host
        self.port = port
        self.api_key = api_key
        self.timeout = timeout
        self._client: Optional[BaseUMICPClient] = None
        self._connected = False
    
    async def connect(self) -> None:
        """Connect to UMICP server."""
        if self._connected and self._client:
            return
        
        try:
            self._client = BaseUMICPClient(
                host=self.host,
                port=self.port,
                timeout=self.timeout
            )
            await self._client.connect()
            self._connected = True
        except Exception as e:
            raise NetworkError(f"Failed to connect to UMICP server: {e}")
    
    async def disconnect(self) -> None:
        """Disconnect from UMICP server."""
        if self._client:
            await self._client.close()
            self._connected = False
            self._client = None
    
    def is_connected(self) -> bool:
        """Check if connected to UMICP server."""
        return self._connected and self._client is not None
    
    async def request(
        self,
        method: str,
        path: str,
        data: Optional[Dict[str, Any]] = None
    ) -> Any:
        """
        Make a request via UMICP.
        
        Args:
            method: HTTP method
            path: API endpoint path
            data: Request data
            
        Returns:
            Response data
        """
        if not self.is_connected():
            await self.connect()
        
        if not self._client:
            raise NetworkError("UMICP client not initialized")
        
        payload = {
            "method": method,
            "path": path,
        }
        
        if data:
            payload["body"] = data
        
        if self.api_key:
            payload["authorization"] = f"Bearer {self.api_key}"
        
        try:
            envelope = Envelope(
                from_addr="vectorizer-client",
                to_addr="vectorizer-server",
                content_type="application/json",
                payload=json.dumps(payload)
            )
            
            envelope.set_priority(Priority.NORMAL)
            
            response = await self._client.send(envelope)
            response_data = json.loads(response.get_payload())
            
            # Handle HTTP-like status codes
            if isinstance(response_data, dict) and "statusCode" in response_data:
                if response_data["statusCode"] >= 400:
                    raise self._handle_error(response_data)
            
            return response_data
        except (ServerError, AuthenticationError):
            raise
        except Exception as e:
            raise NetworkError(f"UMICP request failed: {e}")
    
    async def get(self, path: str) -> Any:
        """Make a GET request."""
        return await self.request("GET", path)
    
    async def post(self, path: str, data: Optional[Dict[str, Any]] = None) -> Any:
        """Make a POST request."""
        return await self.request("POST", path, data)
    
    async def put(self, path: str, data: Optional[Dict[str, Any]] = None) -> Any:
        """Make a PUT request."""
        return await self.request("PUT", path, data)
    
    async def delete(self, path: str) -> Any:
        """Make a DELETE request."""
        return await self.request("DELETE", path)
    
    def _handle_error(self, response_data: Dict[str, Any]) -> Exception:
        """Handle UMICP errors and convert to appropriate exceptions."""
        message = response_data.get("message", f"UMICP Error {response_data.get('statusCode', 'Unknown')}")
        status_code = response_data.get("statusCode", 500)
        
        if status_code == 401:
            return AuthenticationError(message)
        elif status_code == 403:
            return AuthenticationError("Access forbidden")
        elif status_code == 404:
            return ServerError("Resource not found")
        elif status_code in (429, 500, 502, 503, 504):
            return ServerError(message)
        else:
            return ServerError(message)

