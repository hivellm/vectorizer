"""
Examples demonstrating how to use the Hive Vectorizer Python SDK.

This module contains practical examples for common use cases
including basic operations, advanced features, and error handling.
"""

import asyncio
import logging
from typing import List, Dict, Any

from client import VectorizerClient
from models import Vector, CollectionInfo, SearchResult
from exceptions import (
    VectorizerError,
    CollectionNotFoundError,
    ValidationError,
    NetworkError
)

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


async def basic_example():
    """Basic example showing fundamental operations."""
    print("=== Basic Vectorizer Example ===")
    
    # Initialize client
    async with VectorizerClient(
        base_url="http://localhost:15001",
        api_key="your-api-key-here"
    ) as client:
        
        # Check service health
        health = await client.health_check()
        print(f"Service status: {health['status']}")
        
        # List existing collections
        collections = await client.list_collections()
        print(f"Found {len(collections)} collections")
        
        # Create a new collection
        collection_info = await client.create_collection(
            name="example_collection",
            dimension=512,
            description="Example collection for testing"
        )
        print(f"Created collection: {collection_info.name}")
        
        # Generate embeddings
        texts = [
            "The quick brown fox jumps over the lazy dog",
            "Python is a great programming language",
            "Machine learning and AI are transforming the world"
        ]
        
        vectors = []
        for i, text in enumerate(texts):
            embedding = await client.embed_text(text)
            vector = Vector(
                id=f"doc_{i}",
                data=embedding,
                metadata={"text": text, "index": i}
            )
            vectors.append(vector)
        
        # Insert vectors
        result = await client.insert_texts("example_collection", vectors)
        print(f"Inserted {len(vectors)} vectors")
        
        # Search for similar vectors
        query = "programming languages"
        results = await client.search_vectors(
            collection="example_collection",
            query=query,
            limit=3
        )
        
        print(f"Search results for '{query}':")
        for result in results:
            print(f"  - ID: {result.id}, Score: {result.score:.4f}")
            if result.metadata:
                print(f"    Text: {result.metadata.get('text', 'N/A')}")
        
        # Clean up
        await client.delete_collection("example_collection")
        print("Cleaned up example collection")


async def advanced_example():
    """Advanced example showing batch operations and error handling."""
    print("\n=== Advanced Vectorizer Example ===")
    
    async with VectorizerClient() as client:
        try:
            # Create collection with custom settings
            collection = await client.create_collection(
                name="advanced_collection",
                dimension=384,
                similarity_metric="cosine",
                description="Advanced example collection"
            )
            
            # Batch embedding generation
            documents = [
                {"id": "doc1", "content": "Artificial intelligence and machine learning"},
                {"id": "doc2", "content": "Natural language processing techniques"},
                {"id": "doc3", "content": "Deep learning neural networks"},
                {"id": "doc4", "content": "Computer vision and image recognition"},
                {"id": "doc5", "content": "Data science and analytics"}
            ]
            
            vectors = []
            for doc in documents:
                embedding = await client.embed_text(doc["content"])
                vector = Vector(
                    id=doc["id"],
                    data=embedding,
                    metadata={
                        "content": doc["content"],
                        "category": "AI/ML",
                        "created_at": "2025-01-01"
                    }
                )
                vectors.append(vector)
            
            # Insert vectors in batch
            await client.insert_texts("advanced_collection", vectors)
            print(f"Inserted {len(vectors)} vectors in batch")
            
            # Multiple search queries
            queries = [
                "machine learning algorithms",
                "image processing",
                "data analysis"
            ]
            
            for query in queries:
                results = await client.search_vectors(
                    collection="advanced_collection",
                    query=query,
                    limit=2
                )
                
                print(f"\nQuery: '{query}'")
                for result in results:
                    content = result.metadata.get("content", "N/A") if result.metadata else "N/A"
                    print(f"  - {result.id}: {content[:50]}... (score: {result.score:.4f})")
            
            # Get collection statistics
            collection_info = await client.get_collection_info("advanced_collection")
            print(f"\nCollection stats:")
            print(f"  - Name: {collection_info.name}")
            print(f"  - Dimension: {collection_info.dimension}")
            print(f"  - Vector count: {collection_info.vector_count}")
            print(f"  - Status: {collection_info.status}")
            
        except ValidationError as e:
            logger.error(f"Validation error: {e}")
        except NetworkError as e:
            logger.error(f"Network error: {e}")
        except VectorizerError as e:
            logger.error(f"Vectorizer error: {e}")
        finally:
            # Clean up
            try:
                await client.delete_collection("advanced_collection")
                print("Cleaned up advanced collection")
            except CollectionNotFoundError:
                pass  # Collection already deleted


async def error_handling_example():
    """Example demonstrating comprehensive error handling."""
    print("\n=== Error Handling Example ===")
    
    async with VectorizerClient() as client:
        # Example 1: Collection not found
        try:
            await client.get_collection_info("nonexistent_collection")
        except CollectionNotFoundError as e:
            print(f"Expected error: {e}")
        
        # Example 2: Invalid parameters
        try:
            await client.create_collection("", dimension=-1)
        except ValidationError as e:
            print(f"Expected validation error: {e}")
        
        # Example 3: Search in non-existent collection
        try:
            await client.search_vectors("nonexistent", "test query")
        except CollectionNotFoundError as e:
            print(f"Expected collection error: {e}")
        
        # Example 4: Invalid search parameters
        try:
            await client.search_vectors("test_collection", "", limit=-1)
        except ValidationError as e:
            print(f"Expected validation error: {e}")


async def real_world_example():
    """Real-world example: Document similarity search."""
    print("\n=== Real-World Example: Document Similarity ===")
    
    # Sample documents
    documents = [
        {
            "id": "python_tutorial",
            "title": "Python Programming Tutorial",
            "content": "Python is a versatile programming language used for web development, data science, and automation."
        },
        {
            "id": "javascript_guide",
            "title": "JavaScript Development Guide", 
            "content": "JavaScript is essential for frontend web development and creating interactive user interfaces."
        },
        {
            "id": "data_science_intro",
            "title": "Introduction to Data Science",
            "content": "Data science combines statistics, programming, and domain expertise to extract insights from data."
        },
        {
            "id": "machine_learning_basics",
            "title": "Machine Learning Fundamentals",
            "content": "Machine learning algorithms learn patterns from data to make predictions and classifications."
        },
        {
            "id": "web_development_tips",
            "title": "Web Development Best Practices",
            "content": "Modern web development involves responsive design, performance optimization, and user experience."
        }
    ]
    
    async with VectorizerClient() as client:
        # Create collection for documents
        await client.create_collection(
            name="document_search",
            dimension=512,
            description="Document similarity search collection"
        )
        
        # Process documents
        vectors = []
        for doc in documents:
            # Generate embedding from title + content
            text = f"{doc['title']} {doc['content']}"
            embedding = await client.embed_text(text)
            
            vector = Vector(
                id=doc["id"],
                data=embedding,
                metadata={
                    "title": doc["title"],
                    "content": doc["content"],
                    "type": "document"
                }
            )
            vectors.append(vector)
        
        # Insert all documents
        await client.insert_texts("document_search", vectors)
        print(f"Indexed {len(documents)} documents")
        
        # Search for similar documents
        search_queries = [
            "programming languages and development",
            "data analysis and statistics",
            "web design and user interface"
        ]
        
        for query in search_queries:
            results = await client.search_vectors(
                collection="document_search",
                query=query,
                limit=3
            )
            
            print(f"\nQuery: '{query}'")
            print("Most similar documents:")
            for i, result in enumerate(results, 1):
                title = result.metadata.get("title", "Unknown") if result.metadata else "Unknown"
                print(f"  {i}. {title} (similarity: {result.score:.4f})")
        
        # Clean up
        await client.delete_collection("document_search")
        print("\nCleaned up document search collection")


async def main():
    """Run all examples."""
    try:
        await basic_example()
        await advanced_example()
        await error_handling_example()
        await real_world_example()
        
        print("\n=== All Examples Completed Successfully ===")
        
    except Exception as e:
        logger.error(f"Example failed: {e}")
        raise


if __name__ == "__main__":
    # Run examples
    asyncio.run(main())
