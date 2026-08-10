"""A persistent-connection JSON client over `http.client`.

Why not `requests`: it is not in the framework's dependency set, and adding
one for our engine only would mean our client and the engines we compare
against differ in a dimension nobody is measuring.

Why not plain `urllib.request`: it opens a new TCP connection per call. In a
search benchmark that measures handshakes, not the engine — pgvector's client
holds a `psycopg` connection with prepared statements and Qdrant's holds its
own pooled client, so a per-request connection would make Vectorizer look slow
for a reason that has nothing to do with Vectorizer.

`http.client.HTTPConnection` keeps the socket open across requests, which is
the like-for-like behaviour. One connection per worker process, created in
`init_client` and reused by every subsequent call.
"""

from __future__ import annotations

import http.client
import json
from typing import Any
from urllib.parse import urlparse


class JsonHttpError(RuntimeError):
    """Non-2xx response, carrying the server's own explanation."""

    def __init__(self, method: str, path: str, status: int, body: str):
        self.status = status
        self.body = body
        super().__init__(f"{method} {path} -> HTTP {status}: {body[:400]}")


class JsonHttpClient:
    """Minimal JSON-over-HTTP client with a connection held open."""

    def __init__(self, base_url: str, headers: dict | None = None, timeout: float = 60.0):
        parsed = urlparse(base_url)
        if parsed.scheme not in ("http", "https"):
            raise ValueError(
                f"unsupported scheme {parsed.scheme!r} in {base_url!r}; "
                "the benchmark client speaks HTTP"
            )
        self._secure = parsed.scheme == "https"
        self._host = parsed.hostname
        self._port = parsed.port or (443 if self._secure else 80)
        self._timeout = timeout
        self._headers = {"Content-Type": "application/json", **(headers or {})}
        self._conn: http.client.HTTPConnection | None = None

    def _connection(self) -> http.client.HTTPConnection:
        if self._conn is None:
            factory = (
                http.client.HTTPSConnection if self._secure else http.client.HTTPConnection
            )
            self._conn = factory(self._host, self._port, timeout=self._timeout)
        return self._conn

    def request(self, method: str, path: str, body: Any = None) -> Any:
        payload = json.dumps(body).encode("utf-8") if body is not None else None

        for attempt in (1, 2):
            conn = self._connection()
            try:
                conn.request(method, path, body=payload, headers=self._headers)
                response = conn.getresponse()
                raw = response.read()
                break
            except (http.client.HTTPException, OSError):
                # A kept-alive socket the server has since closed raises on the
                # next write. Reconnect once — a genuine failure raises again
                # on the retry and propagates.
                self.close()
                if attempt == 2:
                    raise

        text = raw.decode("utf-8", errors="replace")
        if not 200 <= response.status < 300:
            raise JsonHttpError(method, path, response.status, text)
        if not text:
            return None
        return json.loads(text)

    def close(self) -> None:
        if self._conn is not None:
            try:
                self._conn.close()
            finally:
                self._conn = None
