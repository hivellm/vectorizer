package rpc

import (
	"bytes"
	"encoding/hex"
	"errors"
	"testing"
)

// Codec + types unit tests.
//
// Covers two distinct invariants:
//
// 1. Wire-spec golden vectors — bytes produced by encoding a Request
//    or Response exactly match the hex dumps in
//    docs/specs/VECTORIZER_RPC.md § 11. If these break, the Go SDK
//    can no longer talk to a Rust server.
//
// 2. Round-trip for every VectorizerValue variant: encode → decode →
//    equal-value-back.

// hexConcat returns the concatenation of multiple hex-encoded bytes.
func hexConcat(parts ...string) []byte {
	var out []byte
	for _, p := range parts {
		b, err := hex.DecodeString(p)
		if err != nil {
			panic(err)
		}
		out = append(out, b...)
	}
	return out
}

func TestWireGolden_RequestPING(t *testing.T) {
	// Spec § 11: 08 00 00 00  93  01  a4 50 49 4e 47  90
	frame, err := EncodeFrame(Request{ID: 1, Command: "PING"}.ToMsgpack())
	if err != nil {
		t.Fatalf("EncodeFrame: %v", err)
	}
	expected := hexConcat(
		"08000000",
		"93",
		"01",
		"a4", hex.EncodeToString([]byte("PING")),
		"90",
	)
	if !bytes.Equal(frame, expected) {
		t.Fatalf("frame mismatch:\n  got:      %x\n  expected: %x", frame, expected)
	}
}

func TestWireGolden_ResponseOkPONG(t *testing.T) {
	// Spec § 11:
	//   10 00 00 00  92  01  81 a2 4f 6b  81 a3 53 74 72 a4 50 4f 4e 47
	frame, err := EncodeFrame(ResponseOk(1, StrValue("PONG")).ToMsgpack())
	if err != nil {
		t.Fatalf("EncodeFrame: %v", err)
	}
	expected := hexConcat(
		"10000000",
		"92",
		"01",
		"81",
		"a2", hex.EncodeToString([]byte("Ok")),
		"81",
		"a3", hex.EncodeToString([]byte("Str")),
		"a4", hex.EncodeToString([]byte("PONG")),
	)
	if !bytes.Equal(frame, expected) {
		t.Fatalf("frame mismatch:\n  got:      %x\n  expected: %x", frame, expected)
	}
}

func TestRoundTrip_ValueVariants(t *testing.T) {
	cases := map[string]VectorizerValue{
		"Null":         NullValue(),
		"Bool true":    BoolValue(true),
		"Bool false":   BoolValue(false),
		"Int small":    IntValue(42),
		"Int negative": IntValue(-9999),
		"Float 1.5":    FloatValue(1.5),
		"Float pi":     FloatValue(-3.14159),
		"Bytes":        BytesValue([]byte{0, 1, 2, 255}),
		"Str empty":    StrValue(""),
		"Str ascii":    StrValue("hello"),
		"Str unicode":  StrValue("ünïcödé"),
		"Array": ArrayValue([]VectorizerValue{
			IntValue(1), StrValue("two"),
		}),
		"Map nested": MapValue([]MapPair{
			{Key: StrValue("k"), Value: IntValue(99)},
			{Key: StrValue("nested"), Value: ArrayValue([]VectorizerValue{BoolValue(true)})},
		}),
	}
	for name, v := range cases {
		v := v
		t.Run(name, func(t *testing.T) {
			frame, err := EncodeFrame(v.ToMsgpack())
			if err != nil {
				t.Fatalf("EncodeFrame: %v", err)
			}
			body := frame[HeaderSize:]
			raw, err := DecodeBody(body)
			if err != nil {
				t.Fatalf("DecodeBody: %v", err)
			}
			decoded, err := ValueFromMsgpack(raw)
			if err != nil {
				t.Fatalf("ValueFromMsgpack: %v", err)
			}
			assertValueEqual(t, decoded, v)
		})
	}
}

func TestRoundTrip_ResponseErr(t *testing.T) {
	resp := ResponseErr(42, "something went wrong")
	frame, err := EncodeFrame(resp.ToMsgpack())
	if err != nil {
		t.Fatalf("EncodeFrame: %v", err)
	}
	raw, err := DecodeBody(frame[HeaderSize:])
	if err != nil {
		t.Fatalf("DecodeBody: %v", err)
	}
	decoded, err := ResponseFromMsgpack(raw)
	if err != nil {
		t.Fatalf("ResponseFromMsgpack: %v", err)
	}
	if decoded.ID != 42 || decoded.Result.IsOk || decoded.Result.Message != "something went wrong" {
		t.Fatalf("got %+v", decoded)
	}
}

func TestFrameTooLarge(t *testing.T) {
	oversize := BytesValue(make([]byte, MaxBodySize+1))
	_, err := EncodeFrame(oversize.ToMsgpack())
	if err == nil {
		t.Fatal("expected FrameTooLargeError, got nil")
	}
	var ftl *FrameTooLargeError
	if !errors.As(err, &ftl) {
		t.Fatalf("expected *FrameTooLargeError, got %T (%v)", err, err)
	}
}

func TestVectorizerValue_Accessors(t *testing.T) {
	if s, ok := StrValue("hi").AsStr(); !ok || s != "hi" {
		t.Fatalf("AsStr: got (%q, %v)", s, ok)
	}
	if _, ok := IntValue(7).AsStr(); ok {
		t.Fatal("AsStr should reject Int variant")
	}
	if i, ok := IntValue(42).AsInt(); !ok || i != 42 {
		t.Fatalf("AsInt: got (%d, %v)", i, ok)
	}
	if f, ok := IntValue(3).AsFloat(); !ok || f != 3.0 {
		t.Fatalf("AsFloat from Int: got (%f, %v)", f, ok)
	}
	m := MapValue([]MapPair{
		{Key: StrValue("name"), Value: StrValue("alpha")},
		{Key: StrValue("count"), Value: IntValue(99)},
	})
	v, ok := m.MapGet("name")
	if !ok {
		t.Fatal("MapGet name missing")
	}
	if s, _ := v.AsStr(); s != "alpha" {
		t.Fatalf("MapGet name = %q", s)
	}
	if _, ok := m.MapGet("missing"); ok {
		t.Fatal("MapGet should miss for unknown key")
	}
}

func TestValueFromMsgpack_RejectsBadShapes(t *testing.T) {
	if _, err := ValueFromMsgpack(map[string]any{"BogusTag": 1}); err == nil {
		t.Fatal("expected error for unknown tag")
	}
	if _, err := ValueFromMsgpack(map[string]any{"Int": 1, "Str": "x"}); err == nil {
		t.Fatal("expected error for multi-key dict")
	}
}

// assertValueEqual compares two VectorizerValues for structural equality.
// VectorizerValue contains slices, so a direct == doesn't work.
func assertValueEqual(t *testing.T, got, want VectorizerValue) {
	t.Helper()
	if got.Kind != want.Kind {
		t.Fatalf("kind: got %d, want %d", got.Kind, want.Kind)
	}
	switch got.Kind {
	case ValueNull:
		// nothing more to check
	case ValueBool:
		if got.Bool != want.Bool {
			t.Fatalf("bool: got %v, want %v", got.Bool, want.Bool)
		}
	case ValueInt:
		if got.Int != want.Int {
			t.Fatalf("int: got %d, want %d", got.Int, want.Int)
		}
	case ValueFloat:
		if got.Float != want.Float {
			t.Fatalf("float: got %f, want %f", got.Float, want.Float)
		}
	case ValueBytes:
		if !bytes.Equal(got.Bytes, want.Bytes) {
			t.Fatalf("bytes: got %x, want %x", got.Bytes, want.Bytes)
		}
	case ValueStr:
		if got.Str != want.Str {
			t.Fatalf("str: got %q, want %q", got.Str, want.Str)
		}
	case ValueArray:
		if len(got.Array) != len(want.Array) {
			t.Fatalf("array len: got %d, want %d", len(got.Array), len(want.Array))
		}
		for i := range got.Array {
			assertValueEqual(t, got.Array[i], want.Array[i])
		}
	case ValueMap:
		if len(got.Map) != len(want.Map) {
			t.Fatalf("map len: got %d, want %d", len(got.Map), len(want.Map))
		}
		for i := range got.Map {
			assertValueEqual(t, got.Map[i].Key, want.Map[i].Key)
			assertValueEqual(t, got.Map[i].Value, want.Map[i].Value)
		}
	}
}

