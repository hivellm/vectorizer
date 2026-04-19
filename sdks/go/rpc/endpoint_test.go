package rpc

import (
	"errors"
	"strings"
	"testing"
)

// Unit tests for ParseEndpoint.
//
// The contract is shared with the Rust + Python + TypeScript SDKs;
// the golden URL forms here are the same ones tested in
// sdks/rust/src/rpc/endpoint.rs, sdks/python/tests/rpc/test_endpoint.py,
// and sdks/typescript/tests/rpc/endpoint.test.ts. Keeping them aligned
// across SDKs prevents subtle behaviour drift between language runtimes.

func TestParseEndpoint_RPCWithExplicitHostAndPort(t *testing.T) {
	ep, err := ParseEndpoint("vectorizer://example.com:9000")
	if err != nil {
		t.Fatalf("unexpected err: %v", err)
	}
	if ep.Kind != EndpointRPC || ep.Host != "example.com" || ep.Port != 9000 {
		t.Fatalf("got %+v", ep)
	}
}

func TestParseEndpoint_RPCWithoutPortDefaultsTo15503(t *testing.T) {
	ep, err := ParseEndpoint("vectorizer://example.com")
	if err != nil {
		t.Fatalf("unexpected err: %v", err)
	}
	if ep.Kind != EndpointRPC || ep.Host != "example.com" || ep.Port != DefaultRPCPort {
		t.Fatalf("got %+v", ep)
	}
	if DefaultRPCPort != 15503 {
		t.Fatalf("DefaultRPCPort drifted: %d", DefaultRPCPort)
	}
}

func TestParseEndpoint_BareHostPortIsRPC(t *testing.T) {
	ep, err := ParseEndpoint("localhost:15503")
	if err != nil {
		t.Fatalf("unexpected err: %v", err)
	}
	if ep.Kind != EndpointRPC || ep.Host != "localhost" || ep.Port != 15503 {
		t.Fatalf("got %+v", ep)
	}
}

func TestParseEndpoint_HTTPRoutesToREST(t *testing.T) {
	ep, err := ParseEndpoint("http://localhost:15002")
	if err != nil {
		t.Fatalf("unexpected err: %v", err)
	}
	if ep.Kind != EndpointREST || ep.URL != "http://localhost:15002" {
		t.Fatalf("got %+v", ep)
	}
	ep, err = ParseEndpoint("https://api.example.com")
	if err != nil {
		t.Fatalf("unexpected err: %v", err)
	}
	if ep.Kind != EndpointREST || ep.URL != "https://api.example.com" {
		t.Fatalf("got %+v", ep)
	}
}

func TestParseEndpoint_UnsupportedSchemeRejectedByName(t *testing.T) {
	_, err := ParseEndpoint("ftp://server.example.com")
	if !errors.Is(err, ErrUnsupportedScheme) {
		t.Fatalf("expected ErrUnsupportedScheme, got %v", err)
	}
	if !strings.Contains(err.Error(), "ftp") {
		t.Fatalf("error should name the offending scheme: %v", err)
	}
	if !strings.Contains(err.Error(), "vectorizer") {
		t.Fatalf("error should list valid schemes: %v", err)
	}
}

func TestParseEndpoint_EmptyStringRejected(t *testing.T) {
	_, err := ParseEndpoint("")
	if !errors.Is(err, ErrEmptyURL) {
		t.Fatalf("expected ErrEmptyURL, got %v", err)
	}
	_, err = ParseEndpoint("   ")
	if !errors.Is(err, ErrEmptyURL) {
		t.Fatalf("expected ErrEmptyURL for whitespace, got %v", err)
	}
}

func TestParseEndpoint_UserinfoRejected(t *testing.T) {
	// Both schemes — credentials must go through HELLO, not the URL,
	// to avoid logging or shell-history-saving a token-bearing URL.
	_, err := ParseEndpoint("vectorizer://user:pass@host:15503")
	if !errors.Is(err, ErrCredentialsInURL) {
		t.Fatalf("vectorizer scheme: expected ErrCredentialsInURL, got %v", err)
	}
	_, err = ParseEndpoint("https://user:secret@api.example.com")
	if !errors.Is(err, ErrCredentialsInURL) {
		t.Fatalf("https scheme: expected ErrCredentialsInURL, got %v", err)
	}
}

func TestParseEndpoint_MalformedPortRejected(t *testing.T) {
	_, err := ParseEndpoint("vectorizer://host:not-a-port")
	if err == nil {
		t.Fatal("expected error for non-numeric port")
	}
	if !strings.Contains(err.Error(), "invalid port") {
		t.Fatalf("error should mention 'invalid port': %v", err)
	}
}

func TestParseEndpoint_IPv6WithPort(t *testing.T) {
	ep, err := ParseEndpoint("vectorizer://[::1]:15503")
	if err != nil {
		t.Fatalf("unexpected err: %v", err)
	}
	if ep.Kind != EndpointRPC || ep.Host != "[::1]" || ep.Port != 15503 {
		t.Fatalf("got %+v", ep)
	}
}

func TestParseEndpoint_IPv6WithoutPortDefaults(t *testing.T) {
	ep, err := ParseEndpoint("vectorizer://[::1]")
	if err != nil {
		t.Fatalf("unexpected err: %v", err)
	}
	if ep.Kind != EndpointRPC || ep.Host != "[::1]" || ep.Port != DefaultRPCPort {
		t.Fatalf("got %+v", ep)
	}
}
