"""
Command-line interface for the Hive Vectorizer Python SDK.

This module provides a CLI tool for interacting with the Vectorizer
service from the command line.
"""

import asyncio
import json
import sys
from typing import Optional, List
import argparse
import logging

from client import VectorizerClient
from exceptions import VectorizerError


def setup_logging(verbose: bool = False):
    """Setup logging configuration."""
    level = logging.DEBUG if verbose else logging.INFO
    logging.basicConfig(
        level=level,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )


async def health_check(client: VectorizerClient):
    """Check service health."""
    try:
        health = await client.health_check()
        print(f"✅ Service is {health['status']}")
        print(f"   Service: {health.get('service', 'N/A')}")
        print(f"   Version: {health.get('version', 'N/A')}")
        return True
    except VectorizerError as e:
        print(f"❌ Health check failed: {e}")
        return False


async def list_collections(client: VectorizerClient):
    """List all collections."""
    try:
        collections = await client.list_collections()
        if not collections:
            print("No collections found.")
            return
        
        print(f"Found {len(collections)} collections:")
        for collection in collections:
            print(f"  - {collection.name}")
            print(f"    Dimension: {collection.dimension}")
            print(f"    Vectors: {collection.vector_count}")
            print(f"    Status: {collection.status}")
            print()
    except VectorizerError as e:
        print(f"❌ Failed to list collections: {e}")


async def create_collection(client: VectorizerClient, name: str, dimension: int, description: Optional[str] = None):
    """Create a new collection."""
    try:
        collection = await client.create_collection(
            name=name,
            dimension=dimension,
            description=description
        )
        print(f"✅ Created collection '{collection.name}'")
        print(f"   Dimension: {collection.dimension}")
        print(f"   Status: {collection.status}")
    except VectorizerError as e:
        print(f"❌ Failed to create collection: {e}")


async def delete_collection(client: VectorizerClient, name: str):
    """Delete a collection."""
    try:
        await client.delete_collection(name)
        print(f"✅ Deleted collection '{name}'")
    except VectorizerError as e:
        print(f"❌ Failed to delete collection: {e}")


async def embed_text(client: VectorizerClient, text: str):
    """Generate embedding for text."""
    try:
        embedding = await client.embed_text(text)
        print(f"✅ Generated embedding (dimension: {len(embedding)})")
        print(f"   First 5 values: {embedding[:5]}")
    except VectorizerError as e:
        print(f"❌ Failed to generate embedding: {e}")


async def search_vectors(client: VectorizerClient, collection: str, query: str, limit: int = 10):
    """Search for similar vectors."""
    try:
        results = await client.search_vectors(
            collection=collection,
            query=query,
            limit=limit
        )
        
        if not results:
            print("No results found.")
            return
        
        print(f"Found {len(results)} results:")
        for i, result in enumerate(results, 1):
            print(f"  {i}. ID: {result.id}")
            print(f"     Score: {result.score:.4f}")
            if result.metadata:
                print(f"     Metadata: {result.metadata}")
            print()
    except VectorizerError as e:
        print(f"❌ Failed to search vectors: {e}")


async def get_collection_info(client: VectorizerClient, name: str):
    """Get collection information."""
    try:
        info = await client.get_collection_info(name)
        print(f"Collection: {info.name}")
        print(f"  Dimension: {info.dimension}")
        print(f"  Similarity Metric: {info.similarity_metric}")
        print(f"  Vector Count: {info.vector_count}")
        print(f"  Document Count: {info.document_count}")
        print(f"  Status: {info.status}")
        if info.error_message:
            print(f"  Error: {info.error_message}")
    except VectorizerError as e:
        print(f"❌ Failed to get collection info: {e}")


async def main():
    """Main CLI entry point."""
    parser = argparse.ArgumentParser(
        description="Hive Vectorizer CLI",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  vectorizer-cli health
  vectorizer-cli list-collections
  vectorizer-cli create-collection my_collection 512
  vectorizer-cli embed "Hello, world!"
  vectorizer-cli search my_collection "machine learning" --limit 5
  vectorizer-cli info my_collection
  vectorizer-cli delete-collection my_collection
        """
    )
    
    parser.add_argument(
        "--url",
        default="http://localhost:15001",
        help="Vectorizer service URL (default: http://localhost:15001)"
    )
    parser.add_argument(
        "--api-key",
        help="API key for authentication"
    )
    parser.add_argument(
        "--verbose", "-v",
        action="store_true",
        help="Enable verbose logging"
    )
    
    subparsers = parser.add_subparsers(dest="command", help="Available commands")
    
    # Health check command
    subparsers.add_parser("health", help="Check service health")
    
    # Collection commands
    subparsers.add_parser("list-collections", help="List all collections")
    
    create_parser = subparsers.add_parser("create-collection", help="Create a new collection")
    create_parser.add_argument("name", help="Collection name")
    create_parser.add_argument("dimension", type=int, help="Vector dimension")
    create_parser.add_argument("--description", help="Collection description")
    
    delete_parser = subparsers.add_parser("delete-collection", help="Delete a collection")
    delete_parser.add_argument("name", help="Collection name")
    
    info_parser = subparsers.add_parser("info", help="Get collection information")
    info_parser.add_argument("name", help="Collection name")
    
    # Vector commands
    embed_parser = subparsers.add_parser("embed", help="Generate text embedding")
    embed_parser.add_argument("text", help="Text to embed")
    
    search_parser = subparsers.add_parser("search", help="Search for similar vectors")
    search_parser.add_argument("collection", help="Collection name")
    search_parser.add_argument("query", help="Search query")
    search_parser.add_argument("--limit", type=int, default=10, help="Maximum results (default: 10)")
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return
    
    setup_logging(args.verbose)
    
    # Initialize client
    async with VectorizerClient(
        base_url=args.url,
        api_key=args.api_key
    ) as client:
        
        # Execute command
        if args.command == "health":
            success = await health_check(client)
            sys.exit(0 if success else 1)
            
        elif args.command == "list-collections":
            await list_collections(client)
            
        elif args.command == "create-collection":
            await create_collection(
                client,
                args.name,
                args.dimension,
                args.description
            )
            
        elif args.command == "delete-collection":
            await delete_collection(client, args.name)
            
        elif args.command == "info":
            await get_collection_info(client, args.name)
            
        elif args.command == "embed":
            await embed_text(client, args.text)
            
        elif args.command == "search":
            await search_vectors(
                client,
                args.collection,
                args.query,
                args.limit
            )


if __name__ == "__main__":
    asyncio.run(main())
