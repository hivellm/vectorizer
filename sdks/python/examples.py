"""
Examples demonstrating how to use the Hive Vectorizer Python SDK.

This module contains practical examples for common use cases
including basic operations, advanced features, and error handling.
"""

import asyncio
import logging
from typing import List, Dict, Any

from client import VectorizerClient
from models import Vector, CollectionInfo, SearchResult, BatchInsertRequest, BatchTextRequest, BatchSearchRequest, BatchSearchQuery, BatchDeleteRequest, BatchConfig, SummarizeTextRequest, SummarizeContextRequest
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
    print("🐍 Vectorizer Python SDK Basic Example")
    print("======================================")
    
    # Initialize client
    async with VectorizerClient(
        base_url="http://localhost:15001",
        api_key="your-api-key-here"
    ) as client:
        print("✅ Client created successfully")
        
        collection_name = "example-documents"
        
        try:
            # Health check
            print("\n🔍 Checking server health...")
            try:
                health = await client.health_check()
                print(f"✅ Server status: {health.get('status', 'unknown')}")
                if 'version' in health:
                    print(f"   Version: {health['version']}")
                if 'collections' in health:
                    print(f"   Collections: {health['collections']}")
                if 'total_vectors' in health:
                    print(f"   Total Vectors: {health['total_vectors']}")
            except Exception as e:
                print(f"⚠️ Health check failed: {e}")
            
            # Get database stats
            print("\n📊 Getting database statistics...")
            try:
                stats = await client.get_database_stats()
                print("📈 Database stats:")
                print(f"   Collections: {stats.get('total_collections', 0)}")
                print(f"   Vectors: {stats.get('total_vectors', 0)}")
                size_mb = stats.get('total_size_bytes', 0) / 1024 / 1024
                print(f"   Size: {size_mb:.2f} MB")
            except Exception as e:
                print(f"⚠️ Get stats failed: {e}")
            
            # List existing collections
            print("\n📋 Listing collections...")
            try:
                collections = await client.list_collections()
                print(f"📁 Found {len(collections)} collections:")
                for collection in collections[:5]:
                    print(f"   - {collection.name if hasattr(collection, 'name') else collection}")
            except Exception as e:
                print(f"⚠️ Error listing collections: {e}")
            
            # Create a new collection
            print("\n🆕 Creating collection...")
            try:
                collection_info = await client.create_collection(
                    name=collection_name,
                    dimension=384,
                    similarity_metric="cosine",
                    description="Example collection for testing"
                )
                print(f"✅ Collection created: {collection_info.name}")
                print(f"   Dimension: {collection_info.dimension}")
                print(f"   Metric: {collection_info.metric if hasattr(collection_info, 'metric') else 'cosine'}")
            except Exception as e:
                print(f"⚠️ Collection creation failed (may already exist): {e}")
            
            # Insert texts
            print("\n📥 Inserting texts...")
            texts = [
                {
                    "id": "doc_1",
                    "text": "Introduction to Machine Learning",
                    "metadata": {
                        "source": "document1.pdf",
                        "title": "Introduction to Machine Learning",
                        "category": "AI"
                    }
                },
                {
                    "id": "doc_2",
                    "text": "Deep Learning Fundamentals",
                    "metadata": {
                        "source": "document2.pdf",
                        "title": "Deep Learning Fundamentals",
                        "category": "AI"
                    }
                },
                {
                    "id": "doc_3",
                    "text": "Data Science Best Practices",
                    "metadata": {
                        "source": "document3.pdf",
                        "title": "Data Science Best Practices",
                        "category": "Data"
                    }
                }
            ]
            
            try:
                result = await client.insert_texts(collection_name, texts)
                inserted = result.get('inserted', len(texts)) if isinstance(result, dict) else len(texts)
                print(f"✅ Texts inserted: {inserted}")
            except Exception as e:
                print(f"⚠️ Insert texts failed: {e}")
            
            # Search for similar vectors
            print("\n🔍 Searching for similar vectors...")
            try:
                results = await client.search_vectors(
                    collection=collection_name,
                    query="machine learning algorithms",
                    limit=3
                )
                print("🎯 Search results:")
                for i, result in enumerate(results):
                    score = result.score if hasattr(result, 'score') else result.get('score', 0)
                    print(f"   {i + 1}. Score: {score:.4f}")
                    metadata = result.metadata if hasattr(result, 'metadata') else result.get('metadata', {})
                    if metadata:
                        if 'title' in metadata:
                            print(f"      Title: {metadata['title']}")
                        if 'category' in metadata:
                            print(f"      Category: {metadata['category']}")
            except Exception as e:
                print(f"⚠️ Search failed: {e}")
            
            # Generate embeddings
            print("\n🧠 Generating embeddings...")
            try:
                embedding = await client.embed_text("artificial intelligence and machine learning")
                print("✅ Embedding generated:")
                if isinstance(embedding, dict):
                    print(f"   Dimension: {len(embedding.get('embedding', []))}")
                    print(f"   Model: {embedding.get('model', 'unknown')}")
                else:
                    print(f"   Dimension: {len(embedding) if hasattr(embedding, '__len__') else 'unknown'}")
            except Exception as e:
                print(f"⚠️ Embedding generation failed: {e}")
            
            # Get collection info
            print("\n📊 Getting collection information...")
            try:
                info = await client.get_collection(collection_name)
                print("📈 Collection info:")
                print(f"   Name: {info.name if hasattr(info, 'name') else info.get('name', 'unknown')}")
                print(f"   Dimension: {info.dimension if hasattr(info, 'dimension') else info.get('dimension', 0)}")
                vector_count = info.vector_count if hasattr(info, 'vector_count') else info.get('vector_count', 0)
                print(f"   Vector count: {vector_count}")
                size_bytes = info.size_bytes if hasattr(info, 'size_bytes') else info.get('size_bytes', 0)
                if size_bytes:
                    print(f"   Size: {size_bytes / 1024} KB")
            except Exception as e:
                print(f"⚠️ Get collection info failed: {e}")
            
            print("\n🌐 All operations completed successfully!")
            
            # Clean up
            print("\n🧹 Cleaning up...")
            try:
                await client.delete_collection(collection_name)
                print("✅ Collection deleted")
            except Exception as e:
                print(f"⚠️ Delete collection failed: {e}")
        
        except Exception as e:
            print(f"❌ Error: {e}")
            if hasattr(e, 'details'):
                print(f"📋 Details: {e.details}")
        
        print("\n👋 Example completed!")


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


async def batch_operations_example():
    """Example demonstrating batch operations."""
    print("=== Batch Operations Example ===")
    
    async with VectorizerClient(
        base_url="http://localhost:15001",
        api_key="your-api-key-here"
    ) as client:
        
        collection_name = "batch_example_collection"
        
        try:
            # Create collection for batch operations
            await client.create_collection(
                name=collection_name,
                dimension=512,
                description="Collection for batch operations testing"
            )
            print(f"Created collection: {collection_name}")
            
            # Batch insert texts
            print("\n--- Batch Insert Texts ---")
            batch_insert_request = BatchInsertRequest(
                texts=[
                    BatchTextRequest(
                        id="batch-text-1",
                        text="This is the first batch text for testing",
                        metadata={"source": "batch_test", "type": "example"}
                    ),
                    BatchTextRequest(
                        id="batch-text-2",
                        text="This is the second batch text for testing",
                        metadata={"source": "batch_test", "type": "example"}
                    ),
                    BatchTextRequest(
                        id="batch-text-3",
                        text="This is the third batch text for testing",
                        metadata={"source": "batch_test", "type": "example"}
                    )
                ],
                config=BatchConfig(
                    max_batch_size=100,
                    parallel_workers=4,
                    atomic=True
                )
            )
            
            batch_insert_result = await client.batch_insert_texts(collection_name, batch_insert_request)
            print(f"Batch insert completed:")
            print(f"  - Successful: {batch_insert_result.successful_operations}")
            print(f"  - Failed: {batch_insert_result.failed_operations}")
            print(f"  - Duration: {batch_insert_result.duration_ms}ms")
            
            # Batch search
            print("\n--- Batch Search ---")
            batch_search_request = BatchSearchRequest(
                queries=[
                    BatchSearchQuery(query="batch text", limit=5),
                    BatchSearchQuery(query="testing", limit=3),
                    BatchSearchQuery(query="example", limit=2)
                ],
                config=BatchConfig(parallel_workers=2)
            )
            
            batch_search_result = await client.batch_search_vectors(collection_name, batch_search_request)
            print(f"Batch search completed:")
            print(f"  - Successful queries: {batch_search_result.successful_queries}")
            print(f"  - Failed queries: {batch_search_result.failed_queries}")
            print(f"  - Duration: {batch_search_result.duration_ms}ms")
            print(f"  - Total results: {sum(len(r) for r in batch_search_result.results)}")
            
            # Batch delete
            print("\n--- Batch Delete ---")
            batch_delete_request = BatchDeleteRequest(
                vector_ids=["batch-text-1", "batch-text-2", "batch-text-3"],
                config=BatchConfig(atomic=True)
            )
            
            batch_delete_result = await client.batch_delete_vectors(collection_name, batch_delete_request)
            print(f"Batch delete completed:")
            print(f"  - Successful: {batch_delete_result.successful_operations}")
            print(f"  - Failed: {batch_delete_result.failed_operations}")
            print(f"  - Duration: {batch_delete_result.duration_ms}ms")
            
            # Clean up
            await client.delete_collection(collection_name)
            print(f"\nCleaned up collection: {collection_name}")
            
        except Exception as e:
            logger.error(f"Batch operations example failed: {e}")
            # Try to clean up on error
            try:
                await client.delete_collection(collection_name)
            except:
                pass
            raise


async def main():
    """Run all examples."""
    try:
        await basic_example()
        await advanced_example()
        await error_handling_example()
        await real_world_example()
        await batch_operations_example()
        
        print("\n=== All Examples Completed Successfully ===")
        
    except Exception as e:
        logger.error(f"Example failed: {e}")
        raise


async def summarization_example():
    """Example demonstrating text and context summarization."""
    print("=== Summarization Example ===")
    
    async with VectorizerClient(
        base_url="http://localhost:15001",
        api_key="your-api-key-here"
    ) as client:
        
        # Example text to summarize
        long_text = """
        Artificial Intelligence (AI) has revolutionized numerous industries and continues to shape the future of technology. 
        From healthcare to finance, AI applications are transforming how we work, live, and interact with the world around us.
        
        In healthcare, AI is being used for medical diagnosis, drug discovery, and personalized treatment plans. 
        Machine learning algorithms can analyze vast amounts of medical data to identify patterns and predict outcomes.
        
        In finance, AI powers algorithmic trading, fraud detection, and risk assessment. 
        These systems can process millions of transactions in real-time to identify suspicious activities.
        
        The automotive industry is leveraging AI for autonomous vehicles, traffic optimization, and predictive maintenance.
        Self-driving cars use computer vision and machine learning to navigate roads safely.
        
        As AI technology continues to advance, we can expect even more innovative applications across various sectors.
        However, it's important to consider the ethical implications and ensure responsible AI development.
        """
        
        try:
            # Summarize text using extractive method
            print("Summarizing text using extractive method...")
            text_request = SummarizeTextRequest(
                text=long_text,
                method="extractive",
                compression_ratio=0.3,
                language="en"
            )
            
            text_response = await client.summarize_text(text_request)
            print(f"Original length: {text_response.original_length} characters")
            print(f"Summary length: {text_response.summary_length} characters")
            print(f"Compression ratio: {text_response.compression_ratio:.2f}")
            print(f"Summary: {text_response.summary}")
            
            # Summarize context using keyword method
            print("\nSummarizing context using keyword method...")
            context_request = SummarizeContextRequest(
                context=long_text,
                method="keyword",
                max_length=100,
                language="en"
            )
            
            context_response = await client.summarize_context(context_request)
            print(f"Context summary: {context_response.summary}")
            
            # Get the summary by ID
            print(f"\nRetrieving summary by ID: {text_response.summary_id}")
            summary = await client.get_summary(text_response.summary_id)
            print(f"Retrieved summary: {summary.summary}")
            
            # List all summaries
            print("\nListing all summaries...")
            summaries = await client.list_summaries(limit=5)
            print(f"Found {summaries.total_count} total summaries")
            for summary_info in summaries.summaries:
                print(f"- {summary_info.summary_id}: {summary_info.method} ({summary_info.language})")
            
            print("\n=== Summarization Example Completed Successfully ===")
            
        except Exception as e:
            logger.error(f"Summarization example failed: {e}")
            raise


async def main():
    """Run all examples."""
    try:
        await basic_example()
        await batch_operations_example()
        await summarization_example()
        
        print("\n=== All Examples Completed Successfully ===")
        
    except Exception as e:
        logger.error(f"Example failed: {e}")
        raise
