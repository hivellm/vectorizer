//! Canonical URL parser for the SDK's connection string.
//!
//! The contract (from every `phase6_sdk-*-rpc/proposal.md`):
//!
//! - `vectorizer://host:port` → RPC on the given port.
//! - `vectorizer://host` (no port) → RPC on default port 15503.
//! - `host:port` (no scheme) → RPC.
//! - `http://host:port` / `https://host:port` → REST (legacy fallback).
//! - Anything else → [`ParseError::UnsupportedScheme`].
//!
//! URLs that carry credentials in the userinfo (`user:pass@host`) are
//! REJECTED — credentials cross the wire in the `HELLO` handshake, NOT
//! in the URL. This avoids accidentally logging or shell-history-saving
//! a token-bearing URL.

/// Default RPC port (matches `RpcConfig::default_port()` in the
/// server crate). Documented in wire spec § 12.
pub const DEFAULT_RPC_PORT: u16 = 15503;

/// Default REST port (matches `ServerConfig::default()` in the server
/// crate).
pub const DEFAULT_HTTP_PORT: u16 = 15002;

/// A parsed endpoint; what transport to use and where to connect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Endpoint {
    /// Speak VectorizerRPC at `host:port`.
    Rpc {
        /// DNS hostname or IP literal.
        host: String,
        /// TCP port; defaults to [`DEFAULT_RPC_PORT`] when omitted.
        port: u16,
    },
    /// Speak REST at the given URL. The URL is preserved verbatim
    /// (with scheme + host + port + path) so the HTTP transport can
    /// pass it straight to `reqwest`.
    Rest {
        /// Full URL including scheme — `http://host:port` or
        /// `https://host:port`.
        url: String,
    },
}

/// Reasons [`parse_endpoint`] can fail.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    /// The URL string was empty.
    #[error("endpoint URL is empty")]
    Empty,

    /// The URL used a scheme other than `vectorizer://`, `http://`,
    /// or `https://`.
    #[error("unsupported URL scheme '{scheme}'; expected 'vectorizer', 'http', or 'https'")]
    UnsupportedScheme {
        /// The unrecognised scheme.
        scheme: String,
    },

    /// The URL's authority section couldn't be parsed (e.g. missing
    /// host, malformed port).
    #[error("invalid authority in URL '{raw}': {reason}")]
    InvalidAuthority {
        /// The original URL.
        raw: String,
        /// What went wrong.
        reason: String,
    },

    /// The URL carried credentials in the userinfo section. These
    /// MUST go through the HELLO handshake, not the URL.
    #[error(
        "URL carries credentials in the userinfo section; \
         pass credentials to the HELLO handshake instead of embedding them in the URL"
    )]
    CredentialsInUrl,
}

/// Parse a connection string into a typed [`Endpoint`].
///
/// See the module docstring for the contract. Returns the first
/// matching endpoint shape; never falls through silently.
pub fn parse_endpoint(url: &str) -> Result<Endpoint, ParseError> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err(ParseError::Empty);
    }

    // Split on the first "://" to recognise an explicit scheme.
    if let Some((scheme, rest)) = trimmed.split_once("://") {
        let scheme_lower = scheme.to_ascii_lowercase();
        match scheme_lower.as_str() {
            "vectorizer" => parse_rpc_authority(rest),
            "http" | "https" => parse_rest(scheme_lower.as_str(), rest, trimmed),
            _ => Err(ParseError::UnsupportedScheme {
                scheme: scheme.to_owned(),
            }),
        }
    } else {
        // No scheme — treat as bare `host[:port]` for RPC.
        parse_rpc_authority(trimmed)
    }
}

/// Parse the post-`vectorizer://` part as `host[:port]` and return
/// an [`Endpoint::Rpc`].
fn parse_rpc_authority(authority: &str) -> Result<Endpoint, ParseError> {
    if authority.is_empty() {
        return Err(ParseError::InvalidAuthority {
            raw: authority.to_owned(),
            reason: "missing host".to_owned(),
        });
    }
    if authority.contains('@') {
        return Err(ParseError::CredentialsInUrl);
    }
    // Trim a trailing path; we don't support paths on the RPC scheme.
    let host_port = authority.split(['/', '?', '#']).next().unwrap_or(authority);
    if host_port.is_empty() {
        return Err(ParseError::InvalidAuthority {
            raw: authority.to_owned(),
            reason: "missing host".to_owned(),
        });
    }

    let (host, port) = if let Some(idx) = host_port.rfind(':') {
        // Don't treat IPv6-bracket colons as port separators.
        if host_port.starts_with('[') {
            // IPv6 literal: [::1]:1234. Locate the closing bracket
            // before splitting on the LAST colon after it.
            let close = host_port
                .find(']')
                .ok_or_else(|| ParseError::InvalidAuthority {
                    raw: authority.to_owned(),
                    reason: "unterminated IPv6 literal '['".to_owned(),
                })?;
            let host_part = &host_port[..=close];
            let after_bracket = &host_port[close + 1..];
            if after_bracket.is_empty() {
                (host_part.to_owned(), DEFAULT_RPC_PORT)
            } else if let Some(port_str) = after_bracket.strip_prefix(':') {
                let port = port_str
                    .parse::<u16>()
                    .map_err(|e| ParseError::InvalidAuthority {
                        raw: authority.to_owned(),
                        reason: format!("invalid port: {e}"),
                    })?;
                (host_part.to_owned(), port)
            } else {
                return Err(ParseError::InvalidAuthority {
                    raw: authority.to_owned(),
                    reason: format!("expected ':<port>' after IPv6 literal, got '{after_bracket}'"),
                });
            }
        } else {
            let host = &host_port[..idx];
            let port_str = &host_port[idx + 1..];
            if host.is_empty() {
                return Err(ParseError::InvalidAuthority {
                    raw: authority.to_owned(),
                    reason: "missing host before ':<port>'".to_owned(),
                });
            }
            let port = port_str
                .parse::<u16>()
                .map_err(|e| ParseError::InvalidAuthority {
                    raw: authority.to_owned(),
                    reason: format!("invalid port: {e}"),
                })?;
            (host.to_owned(), port)
        }
    } else {
        // No colon → no explicit port → use the default.
        (host_port.to_owned(), DEFAULT_RPC_PORT)
    };

    Ok(Endpoint::Rpc { host, port })
}

/// Parse an `http(s)://` URL into [`Endpoint::Rest`]. We rebuild the
/// URL rather than echoing `raw` because some callers might pass
/// trailing whitespace or odd casing on the scheme; the rebuild
/// normalises both.
fn parse_rest(scheme: &str, rest: &str, raw: &str) -> Result<Endpoint, ParseError> {
    if rest.is_empty() {
        return Err(ParseError::InvalidAuthority {
            raw: raw.to_owned(),
            reason: "missing host".to_owned(),
        });
    }
    if rest.contains('@') {
        return Err(ParseError::CredentialsInUrl);
    }
    let url = format!("{scheme}://{rest}");
    Ok(Endpoint::Rest { url })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn rpc_with_explicit_host_and_port() {
        let ep = parse_endpoint("vectorizer://example.com:9000").unwrap();
        assert_eq!(
            ep,
            Endpoint::Rpc {
                host: "example.com".into(),
                port: 9000,
            }
        );
    }

    #[test]
    fn rpc_without_port_defaults_to_15503() {
        let ep = parse_endpoint("vectorizer://example.com").unwrap();
        assert_eq!(
            ep,
            Endpoint::Rpc {
                host: "example.com".into(),
                port: DEFAULT_RPC_PORT,
            }
        );
        assert_eq!(DEFAULT_RPC_PORT, 15503);
    }

    #[test]
    fn bare_host_port_without_scheme_is_rpc() {
        let ep = parse_endpoint("localhost:15503").unwrap();
        assert_eq!(
            ep,
            Endpoint::Rpc {
                host: "localhost".into(),
                port: 15503,
            }
        );
    }

    #[test]
    fn http_url_routes_to_rest_endpoint() {
        let ep = parse_endpoint("http://localhost:15002").unwrap();
        assert_eq!(
            ep,
            Endpoint::Rest {
                url: "http://localhost:15002".into(),
            }
        );

        let ep = parse_endpoint("https://api.example.com").unwrap();
        assert_eq!(
            ep,
            Endpoint::Rest {
                url: "https://api.example.com".into(),
            }
        );
    }

    #[test]
    fn unsupported_scheme_is_rejected_by_name() {
        let err = parse_endpoint("ftp://server.example.com").unwrap_err();
        match err {
            ParseError::UnsupportedScheme { scheme } => assert_eq!(scheme, "ftp"),
            other => panic!("expected UnsupportedScheme, got {other:?}"),
        }
    }

    #[test]
    fn empty_string_is_rejected() {
        let err = parse_endpoint("").unwrap_err();
        assert_eq!(err, ParseError::Empty);

        let err = parse_endpoint("   ").unwrap_err();
        assert_eq!(err, ParseError::Empty);
    }

    #[test]
    fn url_with_userinfo_credentials_is_rejected() {
        // RPC scheme: token@host
        let err = parse_endpoint("vectorizer://user:pass@host:15503").unwrap_err();
        assert_eq!(err, ParseError::CredentialsInUrl);

        // REST scheme: same protection so callers can't shell-history
        // a token-bearing URL by accident.
        let err = parse_endpoint("https://user:secret@api.example.com").unwrap_err();
        assert_eq!(err, ParseError::CredentialsInUrl);
    }

    #[test]
    fn malformed_port_is_rejected() {
        let err = parse_endpoint("vectorizer://host:not-a-port").unwrap_err();
        match err {
            ParseError::InvalidAuthority { raw, reason } => {
                assert!(raw.contains("host:not-a-port"));
                assert!(reason.contains("invalid port"), "got reason: {reason}");
            }
            other => panic!("expected InvalidAuthority, got {other:?}"),
        }
    }

    #[test]
    fn ipv6_literal_with_port_works() {
        let ep = parse_endpoint("vectorizer://[::1]:15503").unwrap();
        assert_eq!(
            ep,
            Endpoint::Rpc {
                host: "[::1]".into(),
                port: 15503,
            }
        );
    }

    #[test]
    fn ipv6_literal_without_port_defaults() {
        let ep = parse_endpoint("vectorizer://[::1]").unwrap();
        assert_eq!(
            ep,
            Endpoint::Rpc {
                host: "[::1]".into(),
                port: DEFAULT_RPC_PORT,
            }
        );
    }
}
