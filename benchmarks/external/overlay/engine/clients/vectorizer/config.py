"""Connection settings for the Vectorizer engine client.

Kept deliberately small: everything tunable comes from the experiment JSON so
a run is reproducible from the config alone, not from the environment it
happened to run in.
"""

import os

VECTORIZER_PORT = int(os.getenv("VECTORIZER_PORT", 15002))

# The server refuses to bind 0.0.0.0 without authentication, so a benchmark
# deployment always has auth on and this is required rather than optional.
# Either a JWT from `POST /auth/login` or an API key from `POST /auth/keys`.
VECTORIZER_API_KEY = os.getenv("VECTORIZER_API_KEY")

# The server clamps search `limit` to this, silently
# (`MAX_SEARCH_LIMIT` in crates/vectorizer-server/src/server/rest_handlers/search.rs).
# A dataset asking for top-1000 would therefore be scored against 100 results
# and look like a recall failure rather than a truncation. The searcher checks
# this and fails loudly instead.
MAX_SEARCH_LIMIT = 100

# One collection name for every run so a crashed run leaves something the next
# `configure` step can clean rather than an accumulating pile.
COLLECTION_NAME = os.getenv("VECTORIZER_COLLECTION", "benchmark")


def get_base_url(host: str, connection_params: dict) -> str:
    port = connection_params.get("port", VECTORIZER_PORT)
    scheme = connection_params.get("scheme", "http")
    return f"{scheme}://{host}:{port}"


def get_auth_headers() -> dict:
    """Auth headers, when the server under test has auth enabled.

    Vectorizer sniffs the credential shape: a JWT goes as `Authorization:
    Bearer`, anything else as `X-API-Key`. Mirrored here so a benchmark run
    against an authenticated deployment does not silently 401.
    """
    if not VECTORIZER_API_KEY:
        return {}
    looks_like_jwt = VECTORIZER_API_KEY.count(".") == 2 and all(
        part for part in VECTORIZER_API_KEY.split(".")
    )
    if looks_like_jwt:
        return {"Authorization": f"Bearer {VECTORIZER_API_KEY}"}
    return {"X-API-Key": VECTORIZER_API_KEY}
