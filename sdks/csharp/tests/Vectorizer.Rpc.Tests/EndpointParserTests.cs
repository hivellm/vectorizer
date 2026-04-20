using System;
using Vectorizer.Rpc;
using Xunit;

namespace Vectorizer.Rpc.Tests;

/// <summary>
/// Test vectors mirror the Go SDK's <c>endpoint_test.go</c> and the
/// TypeScript SDK's <c>endpoint.test.ts</c>, so all five first-party
/// SDKs share one contract.
/// </summary>
public class EndpointParserTests
{
    [Theory]
    [InlineData("vectorizer://example.com:16000", "example.com", 16000)]
    [InlineData("vectorizer://example.com", "example.com", EndpointParser.DefaultRpcPort)]
    [InlineData("example.com:16000", "example.com", 16000)]
    [InlineData("example.com", "example.com", EndpointParser.DefaultRpcPort)]
    [InlineData("VECTORIZER://Example.COM:15503", "Example.COM", 15503)]
    [InlineData("[::1]:16000", "[::1]", 16000)]
    [InlineData("[::1]", "[::1]", EndpointParser.DefaultRpcPort)]
    public void Parse_RpcForms_ReturnsRpcEndpoint(string url, string expectedHost, int expectedPort)
    {
        var ep = EndpointParser.Parse(url);

        Assert.Equal(EndpointKind.Rpc, ep.Kind);
        Assert.Equal(expectedHost, ep.Host);
        Assert.Equal(expectedPort, ep.Port);
    }

    [Theory]
    [InlineData("http://api.example.com:15002")]
    [InlineData("https://api.example.com")]
    [InlineData("HTTPS://api.example.com/v1")]
    public void Parse_RestForms_ReturnsRestEndpoint(string url)
    {
        var ep = EndpointParser.Parse(url);

        Assert.Equal(EndpointKind.Rest, ep.Kind);
        Assert.False(string.IsNullOrEmpty(ep.Url));
    }

    [Fact]
    public void Parse_VectorizerWithoutPort_UsesDefault15503()
    {
        var ep = EndpointParser.Parse("vectorizer://prod.hivellm.internal");
        Assert.Equal(EndpointKind.Rpc, ep.Kind);
        Assert.Equal("prod.hivellm.internal", ep.Host);
        Assert.Equal(15503, ep.Port);
    }

    [Theory]
    [InlineData("ftp://example.com")]
    [InlineData("ws://example.com")]
    [InlineData("grpc://example.com")]
    public void Parse_UnsupportedScheme_Throws(string url)
    {
        var ex = Assert.Throws<ArgumentException>(() => EndpointParser.Parse(url));
        Assert.Contains("unsupported URL scheme", ex.Message, StringComparison.Ordinal);
    }

    [Fact]
    public void Parse_NullUrl_Throws()
    {
        var ex = Assert.Throws<ArgumentException>(() => EndpointParser.Parse(null!));
        Assert.Contains("null", ex.Message, StringComparison.Ordinal);
    }

    [Theory]
    [InlineData("")]
    [InlineData("   ")]
    public void Parse_EmptyUrl_Throws(string url)
    {
        var ex = Assert.Throws<ArgumentException>(() => EndpointParser.Parse(url));
        Assert.Contains("empty", ex.Message, StringComparison.Ordinal);
    }

    [Theory]
    [InlineData("vectorizer://user:pass@host:15503")]
    [InlineData("user@host:15503")]
    [InlineData("https://admin:secret@api.example.com")]
    public void Parse_CredentialsInUserinfo_Throws(string url)
    {
        // Per the spec comment in Endpoint.cs and every other SDK, URLs
        // carrying credentials in the userinfo section MUST be rejected —
        // credentials cross the wire in HELLO, not the URL, so they do
        // not end up in shell history or logs.
        var ex = Assert.Throws<ArgumentException>(() => EndpointParser.Parse(url));
        Assert.Contains("userinfo", ex.Message, StringComparison.Ordinal);
    }

    [Theory]
    [InlineData("vectorizer://host:-1")]
    [InlineData("vectorizer://host:70000")]
    [InlineData("vectorizer://host:abc")]
    [InlineData("vectorizer://:15503")]
    public void Parse_MalformedAuthority_Throws(string url)
    {
        Assert.Throws<ArgumentException>(() => EndpointParser.Parse(url));
    }

    [Fact]
    public void Parse_TrailingPath_IsStrippedForRpc()
    {
        var ep = EndpointParser.Parse("vectorizer://host:15503/ignored");
        Assert.Equal(EndpointKind.Rpc, ep.Kind);
        Assert.Equal("host", ep.Host);
        Assert.Equal(15503, ep.Port);
    }
}
