package rpc

import "fmt"

// Wire spec § 2 + § 3: docs/specs/VECTORIZER_RPC.md. Mirrors the
// Rust SDK at sdks/rust/src/rpc/types.rs and the Python SDK at
// sdks/python/rpc/types.py byte-for-byte.
//
// VectorizerValue is a tagged union encoded with rmp-serde's default
// externally-tagged enum representation:
//
//   - Unit variant (Null) → bare string "Null".
//   - Newtype variant (Int(42)) → single-key map {"Int": 42}.
//
// Both Result<T, E> (used by Response.Result) and VectorizerValue
// use this same encoding, which means an Ok(Str("PONG")) round-trips
// as TWO nested single-key maps ({"Ok": {"Str": "PONG"}}). Decoders
// here unwrap both layers; callers see plain Go values.

// ValueKind discriminates the variants of VectorizerValue.
type ValueKind int

// VectorizerValue variants. Order matches the Rust enum.
const (
	ValueNull ValueKind = iota
	ValueBool
	ValueInt
	ValueFloat
	ValueBytes
	ValueStr
	ValueArray
	ValueMap
)

// Tag strings — match the Rust enum variant names exactly.
const (
	tagNull   = "Null"
	tagBool   = "Bool"
	tagInt    = "Int"
	tagFloat  = "Float"
	tagBytes  = "Bytes"
	tagStr    = "Str"
	tagArray  = "Array"
	tagMap    = "Map"
	resultOk  = "Ok"
	resultErr = "Err"
)

// VectorizerValue is a dynamically-typed value that crosses the wire.
// Construct via the named constructors (NullValue, BoolValue, …) so
// the on-wire encoding stays consistent.
type VectorizerValue struct {
	Kind ValueKind
	// Variant payload. Set by the constructor; readers should use the
	// AsX accessors which return zero values + ok=false on mismatch.
	Bool  bool
	Int   int64
	Float float64
	Bytes []byte
	Str   string
	Array []VectorizerValue
	Map   []MapPair
}

// MapPair is one entry of a VectorizerValue.Map. Vec preserves
// insertion order and allows non-string keys, matching MessagePack
// maps and the Rust enum's Vec<(Value, Value)> shape.
type MapPair struct {
	Key, Value VectorizerValue
}

// ── Constructors ──────────────────────────────────────────────────

// NullValue returns the Null variant.
func NullValue() VectorizerValue { return VectorizerValue{Kind: ValueNull} }

// BoolValue wraps b in a Bool variant.
func BoolValue(b bool) VectorizerValue { return VectorizerValue{Kind: ValueBool, Bool: b} }

// IntValue wraps i in an Int variant.
func IntValue(i int64) VectorizerValue { return VectorizerValue{Kind: ValueInt, Int: i} }

// FloatValue wraps f in a Float variant.
func FloatValue(f float64) VectorizerValue { return VectorizerValue{Kind: ValueFloat, Float: f} }

// BytesValue wraps b in a Bytes variant. The slice is stored by reference; do not mutate after passing in.
func BytesValue(b []byte) VectorizerValue { return VectorizerValue{Kind: ValueBytes, Bytes: b} }

// StrValue wraps s in a Str variant.
func StrValue(s string) VectorizerValue { return VectorizerValue{Kind: ValueStr, Str: s} }

// ArrayValue wraps items in an Array variant.
func ArrayValue(items []VectorizerValue) VectorizerValue {
	return VectorizerValue{Kind: ValueArray, Array: items}
}

// MapValue wraps pairs in a Map variant.
func MapValue(pairs []MapPair) VectorizerValue {
	return VectorizerValue{Kind: ValueMap, Map: pairs}
}

// ── Accessors ─────────────────────────────────────────────────────

// AsStr returns the inner string when v is a Str, ok=false otherwise.
func (v VectorizerValue) AsStr() (string, bool) {
	if v.Kind != ValueStr {
		return "", false
	}
	return v.Str, true
}

// AsInt returns the inner integer when v is an Int, ok=false otherwise.
func (v VectorizerValue) AsInt() (int64, bool) {
	if v.Kind != ValueInt {
		return 0, false
	}
	return v.Int, true
}

// AsFloat returns the inner float when v is a Float (or coerces from Int), ok=false otherwise.
func (v VectorizerValue) AsFloat() (float64, bool) {
	switch v.Kind {
	case ValueFloat:
		return v.Float, true
	case ValueInt:
		return float64(v.Int), true
	}
	return 0, false
}

// AsBool returns the inner bool when v is a Bool, ok=false otherwise.
func (v VectorizerValue) AsBool() (bool, bool) {
	if v.Kind != ValueBool {
		return false, false
	}
	return v.Bool, true
}

// AsArray returns the inner slice when v is an Array, nil/false otherwise.
func (v VectorizerValue) AsArray() ([]VectorizerValue, bool) {
	if v.Kind != ValueArray {
		return nil, false
	}
	return v.Array, true
}

// AsMap returns the inner pairs when v is a Map, nil/false otherwise.
func (v VectorizerValue) AsMap() ([]MapPair, bool) {
	if v.Kind != ValueMap {
		return nil, false
	}
	return v.Map, true
}

// MapGet looks up a string-keyed map entry. Returns ok=false when v
// is not a Map or the key is missing. Workhorse for decoding HELLO
// responses and other named-field maps.
func (v VectorizerValue) MapGet(key string) (VectorizerValue, bool) {
	pairs, ok := v.AsMap()
	if !ok {
		return VectorizerValue{}, false
	}
	for _, p := range pairs {
		if s, ok := p.Key.AsStr(); ok && s == key {
			return p.Value, true
		}
	}
	return VectorizerValue{}, false
}

// ── Codec ─────────────────────────────────────────────────────────

// ToMsgpack converts v to the Go value that the msgpack library
// will encode in the externally-tagged shape.
func (v VectorizerValue) ToMsgpack() any {
	switch v.Kind {
	case ValueNull:
		return tagNull
	case ValueBool:
		return map[string]any{tagBool: v.Bool}
	case ValueInt:
		return map[string]any{tagInt: v.Int}
	case ValueFloat:
		return map[string]any{tagFloat: v.Float}
	case ValueBytes:
		return map[string]any{tagBytes: v.Bytes}
	case ValueStr:
		return map[string]any{tagStr: v.Str}
	case ValueArray:
		out := make([]any, len(v.Array))
		for i, item := range v.Array {
			out[i] = item.ToMsgpack()
		}
		return map[string]any{tagArray: out}
	case ValueMap:
		out := make([][2]any, len(v.Map))
		for i, p := range v.Map {
			out[i] = [2]any{p.Key.ToMsgpack(), p.Value.ToMsgpack()}
		}
		return map[string]any{tagMap: out}
	}
	return tagNull
}

// ValueFromMsgpack decodes an externally-tagged msgpack value back
// to a typed VectorizerValue. Returns an error on shape violations.
func ValueFromMsgpack(raw any) (VectorizerValue, error) {
	if s, ok := raw.(string); ok {
		if s == tagNull {
			return NullValue(), nil
		}
		return VectorizerValue{}, fmt.Errorf("unknown unit-variant tag: %q", s)
	}
	m, ok := raw.(map[string]any)
	if !ok {
		return VectorizerValue{}, fmt.Errorf(
			"expected externally-tagged map or 'Null', got %T: %v", raw, raw,
		)
	}
	if len(m) != 1 {
		return VectorizerValue{}, fmt.Errorf(
			"externally-tagged value must have exactly one key, got %d", len(m),
		)
	}
	for tag, payload := range m {
		switch tag {
		case tagBool:
			b, ok := payload.(bool)
			if !ok {
				return VectorizerValue{}, fmt.Errorf("Bool payload must be bool, got %T", payload)
			}
			return BoolValue(b), nil
		case tagInt:
			i, err := coerceInt(payload)
			if err != nil {
				return VectorizerValue{}, fmt.Errorf("Int payload: %w", err)
			}
			return IntValue(i), nil
		case tagFloat:
			f, err := coerceFloat(payload)
			if err != nil {
				return VectorizerValue{}, fmt.Errorf("Float payload: %w", err)
			}
			return FloatValue(f), nil
		case tagBytes:
			b, ok := payload.([]byte)
			if !ok {
				return VectorizerValue{}, fmt.Errorf("Bytes payload must be []byte, got %T", payload)
			}
			return BytesValue(b), nil
		case tagStr:
			s, ok := payload.(string)
			if !ok {
				return VectorizerValue{}, fmt.Errorf("Str payload must be string, got %T", payload)
			}
			return StrValue(s), nil
		case tagArray:
			arr, ok := payload.([]any)
			if !ok {
				return VectorizerValue{}, fmt.Errorf("Array payload must be []any, got %T", payload)
			}
			items := make([]VectorizerValue, len(arr))
			for i, item := range arr {
				v, err := ValueFromMsgpack(item)
				if err != nil {
					return VectorizerValue{}, fmt.Errorf("Array[%d]: %w", i, err)
				}
				items[i] = v
			}
			return ArrayValue(items), nil
		case tagMap:
			pairs, err := decodeMapPairs(payload)
			if err != nil {
				return VectorizerValue{}, err
			}
			return MapValue(pairs), nil
		default:
			return VectorizerValue{}, fmt.Errorf("unknown VectorizerValue tag: %q", tag)
		}
	}
	return VectorizerValue{}, fmt.Errorf("unreachable")
}

// decodeMapPairs handles the two shapes the wire can deliver for a
// Map payload: an array of [key, value] arrays (the on-wire encoding
// for `Vec<(K, V)>`) or a slice of MapPair. The msgpack library
// decodes [2]any into []any{2 elements} via reflection.
func decodeMapPairs(payload any) ([]MapPair, error) {
	arr, ok := payload.([]any)
	if !ok {
		return nil, fmt.Errorf("Map payload must be []any, got %T", payload)
	}
	pairs := make([]MapPair, 0, len(arr))
	for i, entry := range arr {
		// Each entry is a 2-element array [key, value].
		entryArr, ok := entry.([]any)
		if !ok || len(entryArr) != 2 {
			return nil, fmt.Errorf("Map[%d] must be a 2-element array, got %T", i, entry)
		}
		k, err := ValueFromMsgpack(entryArr[0])
		if err != nil {
			return nil, fmt.Errorf("Map[%d].key: %w", i, err)
		}
		v, err := ValueFromMsgpack(entryArr[1])
		if err != nil {
			return nil, fmt.Errorf("Map[%d].value: %w", i, err)
		}
		pairs = append(pairs, MapPair{Key: k, Value: v})
	}
	return pairs, nil
}

func coerceInt(payload any) (int64, error) {
	switch v := payload.(type) {
	case int64:
		return v, nil
	case int:
		return int64(v), nil
	case int8:
		return int64(v), nil
	case int16:
		return int64(v), nil
	case int32:
		return int64(v), nil
	case uint8:
		return int64(v), nil
	case uint16:
		return int64(v), nil
	case uint32:
		return int64(v), nil
	case uint64:
		return int64(v), nil
	}
	return 0, fmt.Errorf("expected integer, got %T", payload)
}

func coerceFloat(payload any) (float64, error) {
	switch v := payload.(type) {
	case float64:
		return v, nil
	case float32:
		return float64(v), nil
	}
	return 0, fmt.Errorf("expected float, got %T", payload)
}

// ── Wire frames ───────────────────────────────────────────────────

// Request is one frame from client to server. Wire spec § 2.
//
// Encoded on the wire as a 3-element MessagePack array
// [id, command, args] to match rmp-serde's default struct
// representation.
type Request struct {
	ID      uint32
	Command string
	Args    []VectorizerValue
}

// ToMsgpack returns the wire-shaped value (a 3-element slice).
func (r Request) ToMsgpack() []any {
	args := make([]any, len(r.Args))
	for i, a := range r.Args {
		args[i] = a.ToMsgpack()
	}
	return []any{r.ID, r.Command, args}
}

// ResponseResult is the discriminated union mirroring Rust's
// Result<Value, String>.
type ResponseResult struct {
	IsOk    bool
	Value   VectorizerValue // populated when IsOk
	Message string          // populated when !IsOk
}

// Response is one frame from server to client. Wire spec § 2.
type Response struct {
	ID     uint32
	Result ResponseResult
}

// ResponseOk builds a successful response. Used by tests/fixtures.
func ResponseOk(id uint32, value VectorizerValue) Response {
	return Response{ID: id, Result: ResponseResult{IsOk: true, Value: value}}
}

// ResponseErr builds an error response. Used by tests/fixtures.
func ResponseErr(id uint32, message string) Response {
	return Response{ID: id, Result: ResponseResult{IsOk: false, Message: message}}
}

// ToMsgpack returns the wire-shaped value (a 2-element slice).
func (r Response) ToMsgpack() []any {
	if r.Result.IsOk {
		return []any{r.ID, map[string]any{resultOk: r.Result.Value.ToMsgpack()}}
	}
	return []any{r.ID, map[string]any{resultErr: r.Result.Message}}
}

// ResponseFromMsgpack decodes a Response from its on-wire shape.
func ResponseFromMsgpack(raw any) (Response, error) {
	arr, ok := raw.([]any)
	if !ok || len(arr) != 2 {
		return Response{}, fmt.Errorf("Response wire frame must be a 2-element array, got %T", raw)
	}
	id, err := coerceInt(arr[0])
	if err != nil {
		return Response{}, fmt.Errorf("Response.id: %w", err)
	}
	resultRaw, ok := arr[1].(map[string]any)
	if !ok || len(resultRaw) != 1 {
		return Response{}, fmt.Errorf("Response.result must be a single-key map, got %T", arr[1])
	}
	for tag, payload := range resultRaw {
		switch tag {
		case resultOk:
			v, err := ValueFromMsgpack(payload)
			if err != nil {
				return Response{}, fmt.Errorf("Ok payload: %w", err)
			}
			return Response{ID: uint32(id), Result: ResponseResult{IsOk: true, Value: v}}, nil
		case resultErr:
			s, ok := payload.(string)
			if !ok {
				return Response{}, fmt.Errorf("Err payload must be string, got %T", payload)
			}
			return Response{ID: uint32(id), Result: ResponseResult{IsOk: false, Message: s}}, nil
		default:
			return Response{}, fmt.Errorf("unknown Result tag: %q", tag)
		}
	}
	return Response{}, fmt.Errorf("unreachable")
}
