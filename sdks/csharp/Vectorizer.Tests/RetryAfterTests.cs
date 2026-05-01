// Retry-After header parser tests for the C# SDK
// (issue #263, phase9 §7).
//
// The full retry loop is exercised end-to-end at the server level by
// crates/vectorizer-server/tests/backpressure_429.rs; here we only
// lock in the value-parsing edges (default, cap, zero, junk) that
// determine how aggressively the SDK backs off.

using Xunit;
using Vectorizer;

namespace Vectorizer.Tests
{
    public class RetryAfterTests
    {
        [Theory]
        [InlineData(null, 1)]
        [InlineData("", 1)]
        [InlineData("   ", 1)]
        [InlineData("0", 1)]
        [InlineData("not-a-number", 1)]
        [InlineData("3", 3)]
        [InlineData("7", 7)]
        [InlineData(" 5 ", 5)]
        // If these cap assertions ever flip, audit RetryAfterMaxSeconds
        // in VectorizerClient.cs first.
        [InlineData("3600", 30)]
        [InlineData("31", 30)]
        public void ParsesRetryAfterValueWithExpectedSemantics(string? input, int expected)
        {
            int actual = VectorizerClient.ParseRetryAfterSeconds(input);
            Assert.Equal(expected, actual);
        }
    }
}
