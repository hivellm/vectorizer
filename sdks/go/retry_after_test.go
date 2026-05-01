// Retry-After header parser tests for the Go SDK
// (issue #263, phase9 §7).
//
// The full retry loop is exercised end-to-end at the server level by
// crates/vectorizer-server/tests/backpressure_429.rs; here we only
// lock in the value-parsing edges (default, cap, zero, junk) that
// determine how aggressively the SDK backs off.

package vectorizer

import "testing"

func TestParseRetryAfterSeconds(t *testing.T) {
	cases := []struct {
		name     string
		input    string
		expected int
	}{
		{"empty header → default", "", 1},
		{"whitespace → default", "   ", 1},
		{"zero → default to avoid busy-loop", "0", 1},
		{"unparseable → default", "not-a-number", 1},
		{"small value passes through", "3", 3},
		{"another small value", "7", 7},
		{"trimmed", " 5 ", 5},
		// If the cap assertion ever flips, audit
		// retryAfterMaxSeconds in client.go first.
		{"large value capped at 30", "3600", 30},
		{"31 capped at 30", "31", 30},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			got := parseRetryAfterSeconds(tc.input)
			if got != tc.expected {
				t.Errorf("parseRetryAfterSeconds(%q) = %d, want %d", tc.input, got, tc.expected)
			}
		})
	}
}
