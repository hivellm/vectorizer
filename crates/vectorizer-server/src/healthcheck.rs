//! Readiness probe for the container `HEALTHCHECK`.
//!
//! The runtime image is `scratch` — no shell, no `wget`, no busybox — so the
//! only executable available to probe with is the server binary itself. Same
//! approach as the other HiveLLM services.
//!
//! Deliberately plain `std`: this runs before the tokio runtime starts, and a
//! probe that needed an async runtime to report "is the server up" would be
//! carrying more machinery than the question deserves.
//!
//! The exit lives in the binary; the decision lives here, so it can be tested
//! against a real socket without the test process exiting.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// How long to wait for a connect, a write, or the response.
///
/// Docker's own `--timeout` kills the probe process, but that leaves the
/// container's health state as "unhealthy after timeout" rather than a clean
/// negative. Failing on our own clock first keeps the signal precise.
const PROBE_TIMEOUT: Duration = Duration::from_secs(3);

/// Ask a running server whether it is ready to serve traffic.
///
/// **`/ready`, not `/health`.** `/health` answers 200 while the collection
/// catalog is still loading (issue #391), so a probe pointed at it reports a
/// half-warm server as ready — and an orchestrator would start routing to an
/// instance still filling its store. On a large instance that window is tens
/// of seconds.
///
/// Any failure — connection refused, timeout, malformed reply, non-200 — is
/// `false`. A probe that could not reach the server has not proved it healthy,
/// which is the only claim this return value makes.
#[must_use]
pub fn probe_ready(addr: &str) -> bool {
    (|| -> std::io::Result<bool> {
        let mut stream = TcpStream::connect(addr)?;
        stream.set_read_timeout(Some(PROBE_TIMEOUT))?;
        stream.set_write_timeout(Some(PROBE_TIMEOUT))?;
        stream.write_all(
            format!("GET /ready HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n").as_bytes(),
        )?;

        // Only the status line matters, and an unbounded read would hang the
        // probe against a server that holds the socket open.
        let mut response = String::new();
        stream.take(1024).read_to_string(&mut response)?;
        Ok(is_ok_status(&response))
    })()
    .unwrap_or(false)
}

/// Whether an HTTP response begins with a 200 status line.
///
/// Split out because "starts with 200" is the whole readiness contract, and a
/// substring check would accept `HTTP/1.1 500 ... 200 ...` in a body.
fn is_ok_status(response: &str) -> bool {
    response.starts_with("HTTP/1.1 200 ") || response.starts_with("HTTP/1.0 200 ")
}

/// The address the probe targets, from `VECTORIZER_PORT` or the default port.
///
/// Always loopback: the probe runs inside the container it is checking, and
/// the server may be bound to `0.0.0.0`.
#[must_use]
pub fn probe_addr() -> String {
    let port = std::env::var("VECTORIZER_PORT").unwrap_or_else(|_| "15002".to_string());
    format!("127.0.0.1:{port}")
}

#[cfg(test)]
mod tests {
    use std::io::Write as _;
    use std::net::TcpListener;
    use std::thread;

    use super::*;

    /// Serve one connection with a canned response, then close.
    fn serve_once(response: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind an ephemeral port");
        let addr = listener.local_addr().expect("read back the bound port");
        thread::spawn(move || {
            if let Ok((mut socket, _)) = listener.accept() {
                // Read the request so the client's write completes, then reply.
                let mut scratch = [0u8; 1024];
                let _ = std::io::Read::read(&mut socket, &mut scratch);
                let _ = socket.write_all(response.as_bytes());
            }
        });
        addr.to_string()
    }

    #[test]
    fn a_200_means_ready() {
        let addr = serve_once("HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n");
        assert!(probe_ready(&addr));
    }

    #[test]
    fn a_503_means_not_ready() {
        // What `/ready` actually answers while the catalog is loading.
        let addr = serve_once("HTTP/1.1 503 Service Unavailable\r\nRetry-After: 5\r\n\r\n");
        assert!(
            !probe_ready(&addr),
            "a warming server must not be reported ready — that is the whole \
             reason this probes /ready instead of /health"
        );
    }

    #[test]
    fn nothing_listening_means_not_ready() {
        // Bind and drop, so the port is almost certainly free and closed.
        let addr = {
            let l = TcpListener::bind("127.0.0.1:0").unwrap();
            l.local_addr().unwrap().to_string()
        };
        assert!(
            !probe_ready(&addr),
            "a probe that cannot connect has not proved the server healthy"
        );
    }

    #[test]
    fn a_200_inside_the_body_is_not_a_200_status() {
        let addr = serve_once("HTTP/1.1 500 Internal Server Error\r\n\r\n{\"code\":200}");
        assert!(
            !probe_ready(&addr),
            "the status line is the contract; a 200 elsewhere in the response \
             is not one"
        );
    }

    #[test]
    fn the_probe_targets_loopback() {
        // The server may be bound to 0.0.0.0; the probe runs beside it.
        assert!(probe_addr().starts_with("127.0.0.1:"));
    }
}
