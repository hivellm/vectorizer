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
        RateLimitError,
    )
except ImportError:
    from exceptions import (
        NetworkError,
        ServerError,
        AuthenticationError,
        RateLimitError,
    )

# Issue #263: cap Retry-After respect at this many seconds so a
# misconfigured server can't pin a client into a half-hour sleep.
_RETRY_AFTER_MAX_SECONDS = 30
# Floor for parsed Retry-After so a `0` or missing header still yields
# a noticeable backoff rather than busy-looping the server.
_RETRY_AFTER_DEFAULT_SECONDS = 1

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
                # JWT shape → `Authorization: Bearer`; anything else
                # (raw API keys from `POST /auth/keys`) → `X-API-Key`.
                # Server middleware won't fall back Bearer → api_key,
                # so routing has to happen client-side.
                parts = self.api_key.split(".")
                if len(parts) == 3 and all(p for p in parts):
                    headers["Authorization"] = f"Bearer {self.api_key}"
                else:
                    headers["X-API-Key"] = self.api_key
            
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

        Honors `Retry-After` on 429 responses (issue #263): the client
        sleeps for the header's value (capped) and retries up to
        ``max_retries`` times. After exhaustion a ``RateLimitError`` is
        raised so callers can surface the back-pressure to the user.

        Args:
            method: HTTP method
            path: API endpoint path
            data: Request data

        Returns:
            Response data
        """
        await self._ensure_session()

        url = f"{self.base_url}{path}"
        attempts_remaining = self.max_retries
        last_429_text: Optional[str] = None

        while True:
            try:
                async with self._session.request(
                    method,
                    url,
                    json=data if data else None,
                ) as response:
                    if response.status == 429:
                        last_429_text = await response.text()
                        if attempts_remaining <= 0:
                            raise RateLimitError(
                                f"HTTP 429 after {self.max_retries} retries: "
                                f"{last_429_text}"
                            )
                        delay = _parse_retry_after(
                            response.headers.get("Retry-After")
                        )
                        logger.info(
                            "Vectorizer 429 — sleeping %.1fs before retry "
                            "(remaining attempts=%d)",
                            delay,
                            attempts_remaining,
                        )
                        attempts_remaining -= 1
                        await asyncio.sleep(delay)
                        continue

                    if response.status >= 400:
                        error_text = await response.text()
                        raise self._handle_error(response.status, error_text)

                    content_type = response.headers.get('Content-Type', '')
                    if 'application/json' in content_type:
                        return await response.json()
                    return await response.text()

            except (ServerError, AuthenticationError, RateLimitError):
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
        elif status == 429:
            # 429 is handled in `request()` via Retry-After; reaching
            # here means the caller bypassed retry handling, so
            # surface a typed RateLimitError instead of a generic 5xx.
            return RateLimitError(message)
        elif status in (500, 502, 503, 504):
            return ServerError(message)
        else:
            return ServerError(message)


def _parse_retry_after(value: Optional[str]) -> float:
    """Parse a ``Retry-After`` header value (seconds form only).

    Returns a sane default + caps an unreasonably large server hint
    so a misconfigured server can't pin a client into a long sleep.
    """
    if not value:
        return _RETRY_AFTER_DEFAULT_SECONDS
    try:
        seconds = float(value.strip())
    except ValueError:
        return _RETRY_AFTER_DEFAULT_SECONDS
    if seconds <= 0:
        return _RETRY_AFTER_DEFAULT_SECONDS
    return min(seconds, _RETRY_AFTER_MAX_SECONDS)

