using System.Text;
using Xunit;
using Vectorizer;
using Vectorizer.Models;

namespace Vectorizer.Tests;

public class FileUploadTests
{
    private readonly string _baseUrl;

    public FileUploadTests()
    {
        _baseUrl = Environment.GetEnvironmentVariable("VECTORIZER_TEST_URL") ?? "http://localhost:15002";
    }

    [Fact(Skip = "Integration test - requires running server")]
    public async Task UploadFileContent_ShouldSucceed()
    {
        // Arrange
        var client = new VectorizerClient(_baseUrl);
        var content = @"
            This is a test document for file upload.
            It contains multiple lines of text to be chunked and indexed.
            The vectorizer should automatically extract, chunk, and create embeddings.
        ";
        var collectionName = "test-uploads";

        // Act
        using var stream = new MemoryStream(Encoding.UTF8.GetBytes(content));
        var response = await client.UploadFileAsync(
            stream,
            "test.txt",
            collectionName,
            chunkSize: 100,
            chunkOverlap: 20
        );

        // Assert
        Assert.NotNull(response);
        Assert.True(response.Success);
        Assert.Equal("test.txt", response.Filename);
        Assert.Equal(collectionName, response.CollectionName);
        Assert.True(response.ChunksCreated > 0);
        Assert.True(response.VectorsCreated > 0);
    }

    [Fact(Skip = "Integration test - requires running server")]
    public async Task UploadFileContentAsync_ShouldSucceed()
    {
        // Arrange
        var client = new VectorizerClient(_baseUrl);
        var content = "This is a simple test document for upload.";
        var collectionName = "test-uploads";

        // Act
        var response = await client.UploadFileContentAsync(
            content,
            "test.txt",
            collectionName
        );

        // Assert
        Assert.NotNull(response);
        Assert.True(response.Success);
        Assert.Equal("test.txt", response.Filename);
        Assert.Equal(collectionName, response.CollectionName);
    }

    [Fact(Skip = "Integration test - requires running server")]
    public async Task UploadFile_WithMetadata_ShouldSucceed()
    {
        // Arrange
        var client = new VectorizerClient(_baseUrl);
        var content = "Document with metadata for testing.";
        var metadata = new Dictionary<string, object>
        {
            { "source", "test" },
            { "type", "document" },
            { "version", 1 }
        };

        // Act
        var response = await client.UploadFileContentAsync(
            content,
            "test.txt",
            "test-uploads",
            metadata: metadata
        );

        // Assert
        Assert.NotNull(response);
        Assert.True(response.Success);
    }

    [Fact(Skip = "Integration test - requires running server")]
    public async Task GetUploadConfig_ShouldReturnConfig()
    {
        // Arrange
        var client = new VectorizerClient(_baseUrl);

        // Act
        var config = await client.GetUploadConfigAsync();

        // Assert
        Assert.NotNull(config);
        Assert.True(config.MaxFileSize > 0);
        Assert.True(config.MaxFileSizeMb > 0);
        Assert.True(config.DefaultChunkSize > 0);
        Assert.NotNull(config.AllowedExtensions);
        Assert.NotEmpty(config.AllowedExtensions);
    }

    [Fact]
    public void FileUploadResponse_Deserialization_ShouldWork()
    {
        // Arrange
        var json = @"{
            ""success"": true,
            ""filename"": ""test.pdf"",
            ""collection_name"": ""docs"",
            ""chunks_created"": 10,
            ""vectors_created"": 10,
            ""file_size"": 2048,
            ""language"": ""pdf"",
            ""processing_time_ms"": 150
        }";

        // Act
        var response = System.Text.Json.JsonSerializer.Deserialize<FileUploadResponse>(json);

        // Assert
        Assert.NotNull(response);
        Assert.True(response.Success);
        Assert.Equal("test.pdf", response.Filename);
        Assert.Equal("docs", response.CollectionName);
        Assert.Equal(10, response.ChunksCreated);
        Assert.Equal(10, response.VectorsCreated);
        Assert.Equal(2048, response.FileSize);
        Assert.Equal("pdf", response.Language);
        Assert.Equal(150, response.ProcessingTimeMs);
    }

    [Fact]
    public void FileUploadConfig_Deserialization_ShouldWork()
    {
        // Arrange
        var json = @"{
            ""max_file_size"": 10485760,
            ""max_file_size_mb"": 10,
            ""allowed_extensions"": ["".txt"", "".pdf"", "".md""],
            ""reject_binary"": true,
            ""default_chunk_size"": 1000,
            ""default_chunk_overlap"": 200
        }";

        // Act
        var config = System.Text.Json.JsonSerializer.Deserialize<FileUploadConfig>(json);

        // Assert
        Assert.NotNull(config);
        Assert.Equal(10485760, config.MaxFileSize);
        Assert.Equal(10, config.MaxFileSizeMb);
        Assert.Equal(3, config.AllowedExtensions.Count);
        Assert.True(config.RejectBinary);
        Assert.Equal(1000, config.DefaultChunkSize);
        Assert.Equal(200, config.DefaultChunkOverlap);
    }
}
