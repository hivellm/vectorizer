//! VectorizerRPC frame codec — `[u32 LE len][MessagePack body]`.
//!
//! Wire spec § 1: `docs/specs/VECTORIZER_RPC.md`. Ported from
//! `../Synap/synap-server/src/protocol/synap_rpc/codec.rs` byte-for-byte
//! so a SynapRPC-conversant client only needs to swap command names to
//! talk to a Vectorizer server.

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use super::types::{Request, Response};

/// Maximum body size accepted on the wire. Frames declaring a larger
/// length crash the connection rather than allocate. 64 MiB is the
/// documented cap in wire spec § 1.
pub const MAX_BODY_SIZE: usize = 64 * 1024 * 1024;

/// Encode any `Serialize` value into a length-prefixed MessagePack frame.
pub fn encode_frame<T: Serialize>(msg: &T) -> Result<Vec<u8>, rmp_serde::encode::Error> {
    let body = rmp_serde::to_vec(msg)?;
    let len = body.len() as u32;
    let mut frame = Vec::with_capacity(4 + body.len());
    frame.extend_from_slice(&len.to_le_bytes());
    frame.extend_from_slice(&body);
    Ok(frame)
}

/// Decode one frame from a byte slice.
///
/// Returns `Ok(None)` if the buffer does not yet contain a complete
/// frame (the caller should read more bytes and retry). Returns
/// `Err(InvalidData)` if the declared length exceeds [`MAX_BODY_SIZE`]
/// — this prevents a malicious client from forcing the server to
/// allocate gigabytes for a body that will never arrive.
pub fn decode_frame<T: for<'de> Deserialize<'de>>(
    buf: &[u8],
) -> Result<Option<(T, usize)>, std::io::Error> {
    if buf.len() < 4 {
        return Ok(None);
    }
    let len = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
    if len > MAX_BODY_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("RPC frame too large: declared {len} bytes, max {MAX_BODY_SIZE}"),
        ));
    }
    let total = 4 + len;
    if buf.len() < total {
        return Ok(None);
    }
    let value = rmp_serde::from_slice(&buf[4..total])
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    Ok(Some((value, total)))
}

/// Read one [`Request`] frame from an async reader.
pub async fn read_request<R: AsyncRead + Unpin>(reader: &mut R) -> std::io::Result<Request> {
    read_frame(reader).await
}

/// Read one [`Response`] frame from an async reader. Used by client
/// implementations and round-trip tests.
pub async fn read_response<R: AsyncRead + Unpin>(reader: &mut R) -> std::io::Result<Response> {
    read_frame(reader).await
}

/// Write a [`Request`] frame to an async writer.
pub async fn write_request<W: AsyncWrite + Unpin>(
    writer: &mut W,
    req: &Request,
) -> std::io::Result<()> {
    write_frame(writer, req).await
}

/// Write a [`Response`] frame to an async writer.
pub async fn write_response<W: AsyncWrite + Unpin>(
    writer: &mut W,
    resp: &Response,
) -> std::io::Result<()> {
    write_frame(writer, resp).await
}

async fn read_frame<T: for<'de> Deserialize<'de>, R: AsyncRead + Unpin>(
    reader: &mut R,
) -> std::io::Result<T> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).await?;
    let len = u32::from_le_bytes(len_buf) as usize;
    if len > MAX_BODY_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("RPC frame too large: declared {len} bytes, max {MAX_BODY_SIZE}"),
        ));
    }
    let mut body = vec![0u8; len];
    reader.read_exact(&mut body).await?;
    rmp_serde::from_slice(&body)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
}

async fn write_frame<T: Serialize, W: AsyncWrite + Unpin>(
    writer: &mut W,
    msg: &T,
) -> std::io::Result<()> {
    let frame = encode_frame(msg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    writer.write_all(&frame).await
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::rpc_wire::types::VectorizerValue;

    #[test]
    fn encode_decode_roundtrip_request() {
        let req = Request {
            id: 1,
            command: "PING".into(),
            args: vec![],
        };
        let frame = encode_frame(&req).unwrap();
        let len = u32::from_le_bytes([frame[0], frame[1], frame[2], frame[3]]) as usize;
        assert_eq!(len + 4, frame.len());
        let (decoded, consumed): (Request, usize) = decode_frame(&frame).unwrap().unwrap();
        assert_eq!(consumed, frame.len());
        assert_eq!(decoded.id, req.id);
        assert_eq!(decoded.command, req.command);
    }

    #[test]
    fn decode_returns_none_on_partial_header() {
        let result: Result<Option<(Request, usize)>, _> = decode_frame(&[0, 0]);
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn decode_returns_none_on_partial_body() {
        let req = Request {
            id: 99,
            command: "PING".into(),
            args: vec![],
        };
        let mut frame = encode_frame(&req).unwrap();
        frame.truncate(frame.len() - 1);
        let result: Result<Option<(Request, usize)>, _> = decode_frame(&frame);
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn decode_rejects_oversized_frame() {
        // Hand-craft a length header that exceeds the cap. The body
        // never arrives — the cap check fires on the header alone so
        // we don't allocate a 1 GiB buffer just to fail.
        let big_len = (MAX_BODY_SIZE as u32) + 1;
        let mut frame = Vec::new();
        frame.extend_from_slice(&big_len.to_le_bytes());
        let result: Result<Option<(Request, usize)>, _> = decode_frame(&frame);
        let err = result.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn encode_all_value_variants() {
        use VectorizerValue::*;
        let variants = vec![
            Null,
            Bool(true),
            Int(-1),
            Float(2.71),
            Bytes(vec![0xff, 0x00]),
            Str("test".into()),
            Array(vec![Int(1), Null]),
            Map(vec![(Str("a".into()), Int(1))]),
        ];
        for v in variants {
            let req = Request {
                id: 0,
                command: "CMD".into(),
                args: vec![v],
            };
            let frame = encode_frame(&req).unwrap();
            let (decoded, _): (Request, usize) = decode_frame(&frame).unwrap().unwrap();
            assert_eq!(decoded.id, 0);
        }
    }

    #[tokio::test]
    async fn async_write_read_roundtrip() {
        use tokio::io::BufReader;
        let req = Request {
            id: 7,
            command: "collections.list".into(),
            args: vec![],
        };
        let mut buf = Vec::new();
        write_request(&mut buf, &req).await.unwrap();
        let mut cursor = BufReader::new(std::io::Cursor::new(buf));
        let decoded = read_request(&mut cursor).await.unwrap();
        assert_eq!(decoded.id, 7);
        assert_eq!(decoded.command, "collections.list");
    }

    #[test]
    fn ping_request_matches_wire_spec_test_vector() {
        // Wire spec § 11 reference vector for `Request { id: 1, command:
        // "PING", args: [] }`. If this test breaks, either the spec is
        // wrong or rmp-serde changed its array encoding in an
        // incompatible way — both are signals to investigate.
        let req = Request {
            id: 1,
            command: "PING".into(),
            args: vec![],
        };
        let frame = encode_frame(&req).unwrap();
        let expected: &[u8] = &[
            0x08, 0x00, 0x00, 0x00, // length = 8
            0x93, // array(3)
            0x01, // id = 1
            0xa4, b'P', b'I', b'N', b'G', // command = "PING"
            0x90, // args = array(0)
        ];
        assert_eq!(frame.as_slice(), expected, "wire-spec test vector drift");
    }
}
