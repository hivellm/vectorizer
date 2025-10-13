"""
HTTP client utility for making API requests using aiohttp.
"""

from typing import Optional, Dict, Any
import aiohttp
import asyncio
import logging

try:
    from ..exceptions import (
        NetworkError,
        ServerError,
        AuthenticationError,
    )
except ImportError:
    from exceptions import (
        NetworkError,
        ServerError,
        AuthenticationError,
    )

logger = logging.getLogger(__name__)


class HTTPClient:
    """HTTP transport client."""
    
    def __init__(
        self,
        base_url: str = "http://localhost:15002",
        api_key: Optional[str] = None,
        timeout: int = 30,
        max_retries: int = 3
    ):
        """
        Initialize HTTP client.
        
        Args:
            base_url: Base URL for HTTP API
            api_key: API key for authentication
            timeout: Request timeout in seconds
            max_retries: Maximum number of retry attempts
        """
        self.base_url = base_url.rstrip('/')
        self.api_key = api_key
        self.timeout = timeout
        self.max_retries = max_retries
        self._session: Optional[aiohttp.ClientSession] = None
    
    async def _ensure_session(self):
        """Ensure aiohttp session is created."""
        if self._session is None or self._session.closed:
            headers = {"Content-Type": "application/json"}
            if self.api_key:
                headers["Authorization"] = f"Bearer {self.api_key}"
            
            timeout_config = aiohttp.ClientTimeout(total=self.timeout)
            self._session = aiohttp.ClientSession(
                headers=headers,
                timeout=timeout_config
            )
    
    async def close(self):
        """Close the HTTP session."""
        if self._session and not self._session.closed:
            await self._session.close()
            self._session = None
    
    async def request(
        self,
        method: str,
        path: str,
        data: Optional[Dict[str, Any]] = None
    ) -> Any:
        """
        Make an HTTP request.
        
        Args:
            method: HTTP method
            path: API endpoint path
            data: Request data
            
        Returns:
            Response data
        """
        await self._ensure_session()
        
        url = f"{self.base_url}{path}"
        
        try:
            async with self._session.request(
                method,
                url,
                json=data if data else None
            ) as response:
                if response.status >= 400:
                    error_text = await response.text()
                    raise self._handle_error(response.status, error_text)
                
                content_type = response.headers.get('Content-Type', '')
                if 'application/json' in content_type:
                    return await response.json()
                return await response.text()
        
        except (ServerError, AuthenticationError):
            raise
        except aiohttp.ClientError as e:
            raise NetworkError(f"HTTP request failed: {e}")
        except asyncio.TimeoutError:
            raise NetworkError("Request timeout")
        except Exception as e:
            raise NetworkError(f"Unknown error: {e}")
    
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
    
    def _handle_error(self, status: int, error_text: str) -> Exception:
        """Handle HTTP errors and convert to appropriate exceptions."""
        message = f"HTTP {status}: {error_text}"
        
        if status == 401:
            return AuthenticationError(message)
        elif status == 403:
            return AuthenticationError("Access forbidden")
        elif status == 404:
            return ServerError("Resource not found")
        elif status in (429, 500, 502, 503, 504):
            return ServerError(message)
        else:
            return ServerError(message)

