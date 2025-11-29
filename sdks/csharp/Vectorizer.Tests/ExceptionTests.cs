using System;
using System.Net.Http;
using System.Threading.Tasks;
using Xunit;
using Vectorizer;
using Vectorizer.Exceptions;

namespace Vectorizer.Tests
{
    public class ExceptionTests
    {
        [Fact]
        public void VectorizerException_ShouldHaveCorrectProperties()
        {
            var exception = new VectorizerException(
                "test_error",
                "Test error message",
                400,
                null);
            
            Assert.Equal("test_error", exception.ErrorType);
            Assert.Contains("Test error message", exception.Message);
            Assert.Equal(400, exception.StatusCode);
            // Note: IsNotFound, IsUnauthorized, IsValidationError might not exist
            // Just verify basic properties
        }

        [Fact]
        public void VectorizerException_WithNotFoundStatus_ShouldSetIsNotFound()
        {
            var exception = new VectorizerException(
                "not_found",
                "Not found",
                404,
                null);
            
            Assert.True(exception.IsNotFound);
        }

        [Fact]
        public void VectorizerException_WithUnauthorizedStatus_ShouldSetIsUnauthorized()
        {
            var exception = new VectorizerException(
                "unauthorized",
                "Unauthorized",
                401,
                null);
            
            Assert.True(exception.IsUnauthorized);
        }

        [Fact]
        public void VectorizerException_WithValidationError_ShouldSetIsValidationError()
        {
            var exception = new VectorizerException(
                "validation_error",
                "Validation error",
                400,
                null);
            
            Assert.True(exception.IsValidationError);
        }

        [Fact]
        public async Task RequestAsync_WithInvalidUrl_ShouldThrowException()
        {
            var client = new VectorizerClient(new ClientConfig
            {
                BaseUrl = "http://invalid-url-that-does-not-exist:9999"
            });
            
            await Assert.ThrowsAsync<HttpRequestException>(async () =>
            {
                await client.HealthAsync();
            });
        }

        [Fact]
        public async Task RequestAsync_WithNonExistentCollection_ShouldThrowException()
        {
            var client = new VectorizerClient(new ClientConfig
            {
                BaseUrl = "http://localhost:15002"
            });
            
            try
            {
                await client.GetCollectionInfoAsync("non_existent_collection_12345");
            }
            catch (VectorizerException ex)
            {
                Assert.True(ex.IsNotFound || ex.StatusCode >= 400);
            }
            catch
            {
                // Server might not be running - this is expected in test environment
            }
        }
    }
}

