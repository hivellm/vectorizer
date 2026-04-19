// Package rpc implements the VectorizerRPC binary transport.
//
// Wire format (docs/specs/VECTORIZER_RPC.md): each frame is a 4-byte
// little-endian unsigned length followed by a MessagePack-encoded body.
// Bodies larger than 64 MiB are rejected to match the server's
// MAX_BODY_SIZE and prevent OOM amplification on malformed inputs.
package rpc

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"io"

	"github.com/vmihailenco/msgpack/v5"
)

// MaxBodySize is the wire-spec § 1 cap on a single frame body.
const MaxBodySize = 64 * 1024 * 1024

// HeaderSize is the length-prefix header in bytes (u32 LE).
const HeaderSize = 4

// FrameTooLargeError is returned when a frame's declared length
// exceeds MaxBodySize. The server enforces the same cap; mirroring it
// client-side avoids allocating a 1 GiB buffer just because a
// malicious peer claimed a silly length.
type FrameTooLargeError struct {
	Size int
}

func (e *FrameTooLargeError) Error() string {
	return fmt.Sprintf("frame body is %d bytes, exceeds 64 MiB cap", e.Size)
}

// FrameDecodeError wraps msgpack decode failures with frame context.
type FrameDecodeError struct {
	Err error
}

func (e *FrameDecodeError) Error() string {
	return fmt.Sprintf("frame body is not valid MessagePack: %v", e.Err)
}

func (e *FrameDecodeError) Unwrap() error { return e.Err }

// EncodeFrame serialises value to a single complete wire frame.
//
// Returns []byte of [u32 LE length][msgpack body]. The header is
// written before the body so a single Conn.Write puts a complete
// frame on the wire.
//
// The encoder runs with UseCompactInts(true) so small uint values
// pack into the smallest representation (e.g. uint32(1) → one byte
// 0x01, not the five-byte 0xce 00 00 00 01). This matches rmp-serde's
// default behaviour and is required by the wire-spec golden vectors
// in docs/specs/VECTORIZER_RPC.md § 11.
func EncodeFrame(value any) ([]byte, error) {
	var buf bytes.Buffer
	enc := msgpack.NewEncoder(&buf)
	enc.UseCompactInts(true)
	if err := enc.Encode(value); err != nil {
		return nil, fmt.Errorf("msgpack encode: %w", err)
	}
	body := buf.Bytes()
	if len(body) > MaxBodySize {
		return nil, &FrameTooLargeError{Size: len(body)}
	}
	frame := make([]byte, HeaderSize+len(body))
	binary.LittleEndian.PutUint32(frame[:HeaderSize], uint32(len(body)))
	copy(frame[HeaderSize:], body)
	return frame, nil
}

// DecodeBody decodes the MessagePack body of a frame back to a Go
// value. Returns an interface{}; callers narrow via type assertion or
// pass through Response.FromMsgpack / VectorizerValue.FromMsgpack
// for typed shapes.
func DecodeBody(body []byte) (any, error) {
	var out any
	if err := msgpack.Unmarshal(body, &out); err != nil {
		return nil, &FrameDecodeError{Err: err}
	}
	return out, nil
}

// ReadFrame blocks until one complete frame has been read from r.
// Returns the decoded body. io.EOF surfaces unchanged when r closes
// cleanly between frames; an io.ErrUnexpectedEOF mid-frame becomes a
// FrameDecodeError so callers don't need to special-case partial
// reads.
func ReadFrame(r io.Reader) (any, error) {
	var header [HeaderSize]byte
	if _, err := io.ReadFull(r, header[:]); err != nil {
		// io.EOF here means the connection closed cleanly between
		// frames — propagate verbatim so callers can distinguish
		// "peer hung up" from "frame is corrupt".
		return nil, err
	}
	length := binary.LittleEndian.Uint32(header[:])
	if length > MaxBodySize {
		return nil, &FrameTooLargeError{Size: int(length)}
	}
	if length == 0 {
		return DecodeBody(nil)
	}
	body := make([]byte, length)
	if _, err := io.ReadFull(r, body); err != nil {
		return nil, fmt.Errorf("frame body read: %w", err)
	}
	return DecodeBody(body)
}
