"""RPC quickstart example for the Vectorizer Python SDK (v3.x default).

Connects to a server on ``127.0.0.1:15503`` (the default RPC port),
does the HELLO handshake, lists collections, and runs a search against
the first one.

Run a Vectorizer server with RPC enabled (the v3.x default config does
this automatically), then::

    cd sdks/python
    python -m examples.rpc_quickstart

Or, with a custom URL::

    VECTORIZER_URL=vectorizer://my-host:15503 python -m examples.rpc_quickstart
"""

from __future__ import annotations

import asyncio
import os
import sys

# Make the SDK importable when running this script directly from the
# checkout (without ``pip install -e .``). Real users who installed the
# package via pip can drop this prelude.
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

from rpc import AsyncRpcClient, HelloPayload  # noqa: E402


async def main() -> None:
    # Two equivalent ways to dial:
    #   1. ``vectorizer://host:port`` — the canonical URL form. This
    #      is the recommended choice because it makes the transport
    #      explicit and round-trips through configuration files
    #      unchanged.
    #   2. Bare ``host:port`` (no scheme) — also accepted, treated as
    #      RPC.
    url = os.environ.get("VECTORIZER_URL", "vectorizer://127.0.0.1:15503")
    print(f"→ Dialing {url}")
    client = await AsyncRpcClient.connect_url(url)

    try:
        # HELLO handshake. In single-user mode (``auth.enabled: false``
        # on the server side), credentials are accepted-but-ignored.
        # When auth is enabled, attach a JWT or API key:
        #     HelloPayload(client_name="rpc-quickstart").with_token("<jwt>")
        hello = await client.hello(HelloPayload(client_name="rpc-quickstart"))
        print(
            f"✓ HELLO ok — server={hello.server_version}  "
            f"protocol_version={hello.protocol_version}  "
            f"authenticated={hello.authenticated}  admin={hello.admin}"
        )
        print(f"  capabilities: {hello.capabilities}")

        # PING (auth-exempt — works pre-HELLO too).
        pong = await client.ping()
        print(f"✓ PING → {pong}")

        # List collections.
        collections = await client.list_collections()
        print(f"✓ {len(collections)} collection(s): {collections}")

        # If we have at least one collection, run a search against it.
        if collections:
            first = collections[0]
            print(f"→ Searching '{first}' for 'vector database'")

            info = await client.get_collection_info(first)
            print(
                f"  collection has {info.vector_count} vectors across "
                f"{info.document_count} documents (dim={info.dimension})"
            )

            hits = await client.search_basic(first, "vector database", 5)
            print(f"  top {len(hits)} hit(s):")
            for hit in hits:
                print(f"    {hit.id} (score={hit.score:.4f})")
        else:
            print("  (no collections to search — create one via REST/MCP or the dashboard)")
    finally:
        await client.close()


if __name__ == "__main__":
    asyncio.run(main())
