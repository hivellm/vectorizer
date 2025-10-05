"""
Example usage of VectorizerStore with LangChain

This module demonstrates how to use VectorizerStore as a LangChain VectorStore
for document storage and retrieval.
"""

from vectorizer_store import VectorizerStore, VectorizerConfig, create_vectorizer_store
from langchain.schema import Document
from langchain.text_splitter import RecursiveCharacterTextSplitter
from langchain.document_loaders import TextLoader
import os


def basic_example():
    """Basic example of using VectorizerStore"""
    print("=== Basic VectorizerStore Example ===")
    
    # Create configuration
    config = VectorizerConfig(
        host="localhost",
        port=15001,
        collection_name="example_documents"
    )
    
    # Create store
    store = VectorizerStore(config)
    
    # Add some documents
    texts = [
        "The quick brown fox jumps over the lazy dog",
        "Python is a great programming language",
        "Machine learning is transforming the world",
        "Vector databases are essential for AI applications"
    ]
    
    metadatas = [
        {"source": "example1.txt", "topic": "animals"},
        {"source": "example2.txt", "topic": "programming"},
        {"source": "example3.txt", "topic": "technology"},
        {"source": "example4.txt", "topic": "databases"}
    ]
    
    # Add texts to store
    vector_ids = store.add_texts(texts, metadatas)
    print(f"Added {len(vector_ids)} documents")
    
    # Search for similar documents
    query = "programming languages"
    results = store.similarity_search(query, k=2)
    
    print(f"\nSearch results for '{query}':")
    for i, doc in enumerate(results, 1):
        print(f"{i}. {doc.page_content}")
        print(f"   Metadata: {doc.metadata}")
        print()
    
    # Search with scores
    results_with_scores = store.similarity_search_with_score(query, k=2)
    print(f"\nSearch results with scores:")
    for i, (doc, score) in enumerate(results_with_scores, 1):
        print(f"{i}. Score: {score:.3f} - {doc.page_content}")


def document_loading_example():
    """Example of loading documents from files"""
    print("\n=== Document Loading Example ===")
    
    # Create store
    store = create_vectorizer_store(
        host="localhost",
        port=15001,
        collection_name="file_documents"
    )
    
    # Create some sample files
    sample_files = {
        "sample1.txt": "Artificial intelligence is revolutionizing many industries.",
        "sample2.txt": "Natural language processing enables computers to understand human language.",
        "sample3.txt": "Computer vision allows machines to interpret visual information."
    }
    
    # Create sample files
    for filename, content in sample_files.items():
        with open(filename, "w", encoding="utf-8") as f:
            f.write(content)
    
    try:
        # Load documents
        documents = []
        for filename in sample_files.keys():
            loader = TextLoader(filename)
            docs = loader.load()
            documents.extend(docs)
        
        # Add documents to store
        store.add_texts(
            [doc.page_content for doc in documents],
            [doc.metadata for doc in documents]
        )
        
        print(f"Loaded {len(documents)} documents from files")
        
        # Search
        results = store.similarity_search("machine learning", k=2)
        print(f"\nSearch results for 'machine learning':")
        for doc in results:
            print(f"- {doc.page_content}")
    
    finally:
        # Clean up sample files
        for filename in sample_files.keys():
            if os.path.exists(filename):
                os.remove(filename)


def text_splitting_example():
    """Example of using text splitting with VectorizerStore"""
    print("\n=== Text Splitting Example ===")
    
    # Create store
    store = create_vectorizer_store(
        host="localhost",
        port=15001,
        collection_name="split_documents"
    )
    
    # Long text to split
    long_text = """
    Artificial intelligence (AI) is intelligence demonstrated by machines, 
    in contrast to the natural intelligence displayed by humans and animals. 
    Leading AI textbooks define the field as the study of "intelligent agents": 
    any device that perceives its environment and takes actions that maximize 
    its chance of successfully achieving its goals. Colloquially, the term 
    "artificial intelligence" is often used to describe machines that mimic 
    "cognitive" functions that humans associate with the human mind, such as 
    "learning" and "problem solving".
    """
    
    # Split text into chunks
    text_splitter = RecursiveCharacterTextSplitter(
        chunk_size=100,
        chunk_overlap=20
    )
    
    chunks = text_splitter.split_text(long_text)
    
    # Add chunks to store
    metadatas = [{"chunk_id": i, "source": "ai_text.txt"} for i in range(len(chunks))]
    store.add_texts(chunks, metadatas)
    
    print(f"Split text into {len(chunks)} chunks")
    
    # Search for specific information
    results = store.similarity_search("machine learning", k=3)
    print(f"\nSearch results for 'machine learning':")
    for i, doc in enumerate(results, 1):
        print(f"{i}. {doc.page_content[:100]}...")


def metadata_filtering_example():
    """Example of using metadata filtering"""
    print("\n=== Metadata Filtering Example ===")
    
    # Create store
    store = create_vectorizer_store(
        host="localhost",
        port=15001,
        collection_name="filtered_documents"
    )
    
    # Add documents with different metadata
    texts = [
        "Python is a versatile programming language",
        "Java is widely used in enterprise applications",
        "JavaScript powers modern web applications",
        "C++ is used for system programming"
    ]
    
    metadatas = [
        {"language": "python", "type": "programming", "year": 2023},
        {"language": "java", "type": "programming", "year": 2023},
        {"language": "javascript", "type": "web", "year": 2023},
        {"language": "cpp", "type": "system", "year": 2022}
    ]
    
    store.add_texts(texts, metadatas)
    
    # Search without filter
    print("Search without filter:")
    results = store.similarity_search("programming", k=4)
    for doc in results:
        print(f"- {doc.page_content}")
    
    # Search with metadata filter
    print("\nSearch with filter (type=programming):")
    filter_dict = {"type": "programming"}
    results = store.similarity_search("programming", k=4, filter=filter_dict)
    for doc in results:
        print(f"- {doc.page_content}")


def batch_operations_example():
    """Example of batch operations"""
    print("\n=== Batch Operations Example ===")
    
    # Create store
    store = create_vectorizer_store(
        host="localhost",
        port=15001,
        collection_name="batch_documents"
    )
    
    # Large batch of documents
    texts = [f"Document {i}: This is sample content for document number {i}" for i in range(100)]
    metadatas = [{"doc_id": i, "batch": "example"} for i in range(100)]
    
    # Add in batch
    vector_ids = store.add_texts(texts, metadatas)
    print(f"Added {len(vector_ids)} documents in batch")
    
    # Search
    results = store.similarity_search("sample content", k=5)
    print(f"Found {len(results)} similar documents")
    
    # Delete some documents
    ids_to_delete = vector_ids[:10]  # Delete first 10
    success = store.delete(ids_to_delete)
    print(f"Deleted {len(ids_to_delete)} documents: {success}")


if __name__ == "__main__":
    print("VectorizerStore LangChain Integration Examples")
    print("=" * 50)
    
    try:
        basic_example()
        document_loading_example()
        text_splitting_example()
        metadata_filtering_example()
        batch_operations_example()
        
        print("\n" + "=" * 50)
        print("All examples completed successfully!")
        
    except Exception as e:
        print(f"Error running examples: {e}")
        print("Make sure Vectorizer is running on localhost:15001")
