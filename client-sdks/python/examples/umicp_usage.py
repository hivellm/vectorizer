"""
Example: Using the Vectorizer client with UMICP protocol

UMICP (Universal Messaging and Inter-process Communication Protocol) provides:
- High-performance communication
- Official umicp-python package integration
- Efficient transport layer
"""

import asyncio
import sys
sys.path.insert(0, '..')

from client import VectorizerClient


async def main():
    print("=== Vectorizer Client with UMICP ===\n")

    # Option 1: Using connection string
    print("Option 1: Connection string")
    client1 = VectorizerClient(
        connection_string="umicp://localhost:15003",
        api_key="your-api-key-here"
    )
    print(f"Protocol: {client1.get_protocol()}")

    # Option 2: Using explicit configuration
    print("\nOption 2: Explicit configuration")
    client2 = VectorizerClient(
        protocol="umicp",
        api_key="your-api-key-here",
        umicp={
            "host": "localhost",
            "port": 15003
        }
    )
    print(f"Protocol: {client2.get_protocol()}")

    try:
        async with client2:
            # Health check
            print("\n1. Health Check")
            health = await client2.health_check()
            print(f"Server status: {health}")

            # List collections
            print("\n2. List Collections")
            collections = await client2.list_collections()
            print(f"Found {len(collections)} collection(s)")

            if collections:
                # Search in first collection
                collection_name = collections[0].name
                print(f"\n3. Searching in collection: {collection_name}")
                
                search_results = await client2.search_vectors(
                    collection_name,
                    query="example search query",
                    limit=5
                )
                
                print(f"Found {len(search_results.results)} result(s)")
                for i, result in enumerate(search_results.results, 1):
                    print(f"  {i}. Score: {result.score:.4f}")

            # Create collection
            print("\n4. Create Collection")
            await client2.create_collection(
                name="test-umicp-collection",
                dimension=384,
                similarity_metric="cosine"
            )
            print("Collection created successfully")

            # Insert vectors
            print("\n5. Insert Vectors")
            await client2.insert_texts(
                collection="test-umicp-collection",
                texts=[
                    {"id": "1", "text": "Hello UMICP world"},
                    {"id": "2", "text": "UMICP provides efficient communication"},
                    {"id": "3", "text": "Vector search with UMICP"}
                ]
            )
            print("Vectors inserted successfully")

            # Search with UMICP
            print("\n6. Search with UMICP")
            umicp_results = await client2.search_vectors(
                "test-umicp-collection",
                query="efficient communication",
                limit=3
            )

            print(f"Found {len(umicp_results.results)} result(s):")
            for i, result in enumerate(umicp_results.results, 1):
                print(f"  {i}. ID: {result.id}, Score: {result.score:.4f}")
                if hasattr(result, 'metadata') and result.metadata:
                    text = result.metadata.get('text', '')
                    if text:
                        print(f"     Text: {text}")

            # Cleanup
            print("\n7. Cleanup")
            await client2.delete_collection("test-umicp-collection")
            print("Collection deleted")

    except Exception as e:
        print(f"Error: {e}")

    print("\n=== UMICP Demo Complete ===")


if __name__ == "__main__":
    asyncio.run(main())

