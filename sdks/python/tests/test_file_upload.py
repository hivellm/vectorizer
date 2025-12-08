"""Tests for file upload functionality."""
import os
import pytest
from vectorizer_sdk import VectorizerClient
from vectorizer_sdk.models import FileUploadResponse, FileUploadConfig


@pytest.fixture
def client():
    """Create a test client."""
    base_url = os.getenv("VECTORIZER_TEST_URL", "http://localhost:15002")
    return VectorizerClient(base_url=base_url)


@pytest.mark.asyncio
async def test_upload_file_content(client):
    """Test uploading file content."""
    content = """
    This is a test document for file upload.
    It contains multiple lines of text to be chunked and indexed.
    The vectorizer should automatically extract, chunk, and create embeddings.
    """

    try:
        response = await client.upload_file_content(
            content=content,
            filename="test.txt",
            collection_name="test-uploads",
            chunk_size=100,
            chunk_overlap=20,
        )

        assert response is not None
        assert response.success is True
        assert response.filename == "test.txt"
        assert response.collection_name == "test-uploads"
        assert response.chunks_created > 0
        assert response.vectors_created > 0

        print(
            f"✓ Upload successful: {response.chunks_created} chunks, "
            f"{response.vectors_created} vectors"
        )
    except Exception as e:
        # Skip test if server not available
        if "Connection refused" in str(e) or "Failed to establish" in str(e):
            pytest.skip(f"Server not available: {e}")
        raise


@pytest.mark.asyncio
async def test_upload_file_with_metadata(client):
    """Test uploading file with metadata."""
    content = "Document with metadata for testing."
    metadata = {
        "source": "test",
        "type": "document",
        "version": 1,
    }

    try:
        response = await client.upload_file_content(
            content=content,
            filename="test.txt",
            collection_name="test-uploads",
            metadata=metadata,
        )

        assert response is not None
        assert response.success is True
    except Exception as e:
        if "Connection refused" in str(e) or "Failed to establish" in str(e):
            pytest.skip(f"Server not available: {e}")
        raise


@pytest.mark.asyncio
async def test_get_upload_config(client):
    """Test getting upload configuration."""
    try:
        config = await client.get_upload_config()

        assert config is not None
        assert config.max_file_size > 0
        assert config.max_file_size_mb > 0
        assert config.default_chunk_size > 0
        assert isinstance(config.allowed_extensions, list)
        assert len(config.allowed_extensions) > 0

        print(
            f"✓ Config: max={config.max_file_size_mb}MB, "
            f"chunk={config.default_chunk_size}"
        )
    except Exception as e:
        if "Connection refused" in str(e) or "Failed to establish" in str(e):
            pytest.skip(f"Server not available: {e}")
        raise


def test_file_upload_response_model():
    """Test FileUploadResponse model."""
    data = {
        "success": True,
        "filename": "test.pdf",
        "collection_name": "docs",
        "chunks_created": 10,
        "vectors_created": 10,
        "file_size": 2048,
        "language": "pdf",
        "processing_time_ms": 150,
    }

    response = FileUploadResponse(**data)

    assert response.success is True
    assert response.filename == "test.pdf"
    assert response.collection_name == "docs"
    assert response.chunks_created == 10
    assert response.vectors_created == 10
    assert response.file_size == 2048
    assert response.language == "pdf"
    assert response.processing_time_ms == 150


def test_file_upload_config_model():
    """Test FileUploadConfig model."""
    data = {
        "max_file_size": 10485760,
        "max_file_size_mb": 10,
        "allowed_extensions": [".txt", ".pdf", ".md"],
        "reject_binary": True,
        "default_chunk_size": 1000,
        "default_chunk_overlap": 200,
    }

    config = FileUploadConfig(**data)

    assert config.max_file_size == 10485760
    assert config.max_file_size_mb == 10
    assert len(config.allowed_extensions) == 3
    assert config.reject_binary is True
    assert config.default_chunk_size == 1000
    assert config.default_chunk_overlap == 200


def test_file_upload_response_validation():
    """Test FileUploadResponse validation."""
    # Valid data
    valid_data = {
        "success": True,
        "filename": "test.txt",
        "collection_name": "docs",
        "chunks_created": 5,
        "vectors_created": 5,
        "file_size": 1024,
        "language": "text",
        "processing_time_ms": 100,
    }

    response = FileUploadResponse(**valid_data)
    assert response.chunks_created == 5

    # Test negative values should raise error
    invalid_data = valid_data.copy()
    invalid_data["chunks_created"] = -1

    with pytest.raises(ValueError):
        FileUploadResponse(**invalid_data)
