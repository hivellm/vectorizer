package rpc

import (
	"errors"
	"fmt"
	"strconv"
	"strings"
)

// DefaultRPCPort is the canonical port for VectorizerRPC. Matches the
// server's RpcConfig::default_port().
const DefaultRPCPort = 15503

// DefaultHTTPPort is the canonical port for the legacy REST surface.
// Matches the server's ServerConfig::default().
const DefaultHTTPPort = 15002

// EndpointKind discriminates between RPC and REST endpoints.
type EndpointKind int

const (
	// EndpointRPC indicates a vectorizer:// (or bare host:port) URL.
	EndpointRPC EndpointKind = iota
	// EndpointREST indicates an http(s):// URL.
	EndpointREST
)

// Endpoint is a parsed connection string. Either RPC or REST is
// populated, never both.
type Endpoint struct {
	Kind EndpointKind
	// RPC fields (Kind == EndpointRPC).
	Host string
	Port int
	// REST fields (Kind == EndpointREST).
	URL string
}

// ErrUnsupportedScheme wraps the error returned for any scheme other
// than vectorizer://, http://, or https://. Callers may use
// errors.Is(err, ErrUnsupportedScheme) to discriminate.
var ErrUnsupportedScheme = errors.New("unsupported URL scheme")

// ErrCredentialsInURL wraps the error returned when a URL carries
// credentials in the userinfo section (user:pass@host). Credentials
// MUST go through the HELLO handshake instead — embedding them in
// the URL risks logging or shell-history-saving a token-bearing URL.
var ErrCredentialsInURL = errors.New(
	"URL carries credentials in userinfo; pass credentials to HELLO instead",
)

// ErrEmptyURL is returned when ParseEndpoint is given an empty (or
// whitespace-only) string.
var ErrEmptyURL = errors.New("endpoint URL is empty")

// ParseEndpoint parses a connection string into a typed Endpoint.
//
// Mirrors the Rust SDK at sdks/rust/src/rpc/endpoint.rs and the
// Python SDK at sdks/python/rpc/endpoint.py:
//
//   - "vectorizer://host:port"      → EndpointRPC on the given port.
//   - "vectorizer://host"           → EndpointRPC on DefaultRPCPort (15503).
//   - "host:port" (no scheme)       → EndpointRPC.
//   - "http://host:port"            → EndpointREST.
//   - "https://host:port"           → EndpointREST.
//   - anything else                 → ErrUnsupportedScheme.
//
// URLs that carry credentials in the userinfo section are REJECTED
// with ErrCredentialsInURL.
func ParseEndpoint(url string) (Endpoint, error) {
	trimmed := strings.TrimSpace(url)
	if trimmed == "" {
		return Endpoint{}, ErrEmptyURL
	}

	if idx := strings.Index(trimmed, "://"); idx >= 0 {
		scheme := strings.ToLower(trimmed[:idx])
		rest := trimmed[idx+3:]
		switch scheme {
		case "vectorizer":
			return parseRPCAuthority(rest)
		case "http", "https":
			return parseREST(scheme, rest, trimmed)
		default:
			return Endpoint{}, fmt.Errorf(
				"%w '%s'; expected 'vectorizer', 'http', or 'https'",
				ErrUnsupportedScheme, trimmed[:idx],
			)
		}
	}

	// No scheme — treat as bare host[:port] for RPC.
	return parseRPCAuthority(trimmed)
}

func parseRPCAuthority(authority string) (Endpoint, error) {
	if authority == "" {
		return Endpoint{}, fmt.Errorf("invalid authority: missing host")
	}
	if strings.ContainsRune(authority, '@') {
		return Endpoint{}, ErrCredentialsInURL
	}

	// Trim a trailing path; the RPC scheme has no notion of paths.
	hostPort := authority
	for _, sep := range []string{"/", "?", "#"} {
		if idx := strings.Index(hostPort, sep); idx >= 0 {
			hostPort = hostPort[:idx]
		}
	}
	if hostPort == "" {
		return Endpoint{}, fmt.Errorf("invalid authority '%s': missing host", authority)
	}

	// IPv6 literal: [::1] or [::1]:port. Bracket-aware split.
	if strings.HasPrefix(hostPort, "[") {
		close := strings.Index(hostPort, "]")
		if close < 0 {
			return Endpoint{}, fmt.Errorf(
				"invalid authority '%s': unterminated IPv6 literal '['", authority,
			)
		}
		host := hostPort[:close+1]
		after := hostPort[close+1:]
		if after == "" {
			return Endpoint{Kind: EndpointRPC, Host: host, Port: DefaultRPCPort}, nil
		}
		if !strings.HasPrefix(after, ":") {
			return Endpoint{}, fmt.Errorf(
				"invalid authority '%s': expected ':<port>' after IPv6 literal, got '%s'",
				authority, after,
			)
		}
		port, err := parsePort(after[1:], authority)
		if err != nil {
			return Endpoint{}, err
		}
		return Endpoint{Kind: EndpointRPC, Host: host, Port: port}, nil
	}

	// Hostname or IPv4. Split on the LAST colon so a port-like suffix
	// wins over an unrelated colon in any future hostname extension.
	if colon := strings.LastIndex(hostPort, ":"); colon >= 0 {
		host := hostPort[:colon]
		portStr := hostPort[colon+1:]
		if host == "" {
			return Endpoint{}, fmt.Errorf(
				"invalid authority '%s': missing host before ':<port>'", authority,
			)
		}
		port, err := parsePort(portStr, authority)
		if err != nil {
			return Endpoint{}, err
		}
		return Endpoint{Kind: EndpointRPC, Host: host, Port: port}, nil
	}

	return Endpoint{Kind: EndpointRPC, Host: hostPort, Port: DefaultRPCPort}, nil
}

func parseREST(scheme, rest, raw string) (Endpoint, error) {
	if rest == "" {
		return Endpoint{}, fmt.Errorf("invalid authority in URL '%s': missing host", raw)
	}
	if strings.ContainsRune(rest, '@') {
		return Endpoint{}, ErrCredentialsInURL
	}
	return Endpoint{Kind: EndpointREST, URL: scheme + "://" + rest}, nil
}

func parsePort(portStr, authority string) (int, error) {
	port, err := strconv.Atoi(portStr)
	if err != nil {
		return 0, fmt.Errorf("invalid authority '%s': invalid port: %v", authority, err)
	}
	if port < 0 || port > 65535 {
		return 0, fmt.Errorf(
			"invalid authority '%s': port %d is out of range 0..65535", authority, port,
		)
	}
	return port, nil
}
