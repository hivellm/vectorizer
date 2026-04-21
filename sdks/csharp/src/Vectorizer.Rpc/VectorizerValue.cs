using System;
using System.Collections.Generic;
using System.Globalization;

namespace Vectorizer.Rpc;

/// <summary>
/// Discriminator for the <see cref="VectorizerValue"/> tagged union.
/// Order matches the Rust enum in <c>docs/specs/VECTORIZER_RPC.md</c> § 3.
/// </summary>
public enum ValueKind
{
    Null,
    Bool,
    Int,
    Float,
    Bytes,
    Str,
    Array,
    Map,
}

/// <summary>
/// A single key/value pair inside a <see cref="VectorizerValue"/> of kind
/// <see cref="ValueKind.Map"/>. MessagePack maps preserve insertion order
/// and permit non-string keys, so the payload is a list of pairs rather
/// than a dictionary.
/// </summary>
public readonly struct MapPair
{
    public VectorizerValue Key { get; }
    public VectorizerValue Value { get; }

    public MapPair(VectorizerValue key, VectorizerValue value)
    {
        Key = key;
        Value = value;
    }

    public void Deconstruct(out VectorizerValue key, out VectorizerValue value)
    {
        key = Key;
        value = Value;
    }
}

/// <summary>
/// Dynamically-typed value that crosses the Vectorizer RPC wire. Mirrors
/// the <c>VectorizerValue</c> Rust enum with its externally-tagged
/// msgpack representation (unit variants are a bare string, newtype
/// variants are a single-key map). Construct via the static factories
/// — the ctor is intentionally private to keep the encoding stable.
/// </summary>
public sealed class VectorizerValue : IEquatable<VectorizerValue>
{
    /// <summary>Wire variant this value holds.</summary>
    public ValueKind Kind { get; }

    private readonly bool _bool;
    private readonly long _int;
    private readonly double _float;
    private readonly byte[]? _bytes;
    private readonly string? _str;
    private readonly IReadOnlyList<VectorizerValue>? _array;
    private readonly IReadOnlyList<MapPair>? _map;

    private VectorizerValue(
        ValueKind kind,
        bool @bool = false,
        long @int = 0,
        double @float = 0,
        byte[]? bytes = null,
        string? str = null,
        IReadOnlyList<VectorizerValue>? array = null,
        IReadOnlyList<MapPair>? map = null)
    {
        Kind = kind;
        _bool = @bool;
        _int = @int;
        _float = @float;
        _bytes = bytes;
        _str = str;
        _array = array;
        _map = map;
    }

    /// <summary>Returns the canonical <c>Null</c> variant.</summary>
    public static VectorizerValue Null { get; } = new(ValueKind.Null);

    public static VectorizerValue OfBool(bool v) => new(ValueKind.Bool, @bool: v);

    public static VectorizerValue OfInt(long v) => new(ValueKind.Int, @int: v);

    public static VectorizerValue OfFloat(double v) => new(ValueKind.Float, @float: v);

    public static VectorizerValue OfBytes(byte[] v)
    {
        ArgumentNullException.ThrowIfNull(v);
        return new VectorizerValue(ValueKind.Bytes, bytes: v);
    }

    public static VectorizerValue OfStr(string v)
    {
        ArgumentNullException.ThrowIfNull(v);
        return new VectorizerValue(ValueKind.Str, str: v);
    }

    public static VectorizerValue OfArray(IReadOnlyList<VectorizerValue> items)
    {
        ArgumentNullException.ThrowIfNull(items);
        return new VectorizerValue(ValueKind.Array, array: items);
    }

    public static VectorizerValue OfMap(IReadOnlyList<MapPair> pairs)
    {
        ArgumentNullException.ThrowIfNull(pairs);
        return new VectorizerValue(ValueKind.Map, map: pairs);
    }

    // ── Accessors ──────────────────────────────────────────────────────

    public bool TryAsBool(out bool v) { v = _bool; return Kind == ValueKind.Bool; }

    public bool TryAsInt(out long v) { v = _int; return Kind == ValueKind.Int; }

    /// <summary>
    /// Returns the numeric content for either <see cref="ValueKind.Float"/>
    /// or <see cref="ValueKind.Int"/>. MessagePack promotes a small enough
    /// float to an int on the wire, so consumers that want a double must
    /// accept both.
    /// </summary>
    public bool TryAsFloat(out double v)
    {
        switch (Kind)
        {
            case ValueKind.Float: v = _float; return true;
            case ValueKind.Int: v = _int; return true;
            default: v = 0; return false;
        }
    }

    public bool TryAsBytes(out byte[] v)
    {
        if (Kind == ValueKind.Bytes) { v = _bytes!; return true; }
        v = Array.Empty<byte>();
        return false;
    }

    public bool TryAsStr(out string v)
    {
        if (Kind == ValueKind.Str) { v = _str!; return true; }
        v = string.Empty;
        return false;
    }

    public bool TryAsArray(out IReadOnlyList<VectorizerValue> v)
    {
        if (Kind == ValueKind.Array) { v = _array!; return true; }
        v = System.Array.Empty<VectorizerValue>();
        return false;
    }

    public bool TryAsMap(out IReadOnlyList<MapPair> v)
    {
        if (Kind == ValueKind.Map) { v = _map!; return true; }
        v = System.Array.Empty<MapPair>();
        return false;
    }

    /// <summary>Looks up a string-keyed entry in a <see cref="ValueKind.Map"/>.</summary>
    public bool TryMapGet(string key, out VectorizerValue value)
    {
        if (Kind == ValueKind.Map)
        {
            foreach (var (k, v) in _map!)
            {
                if (k.TryAsStr(out var s) && s == key)
                {
                    value = v;
                    return true;
                }
            }
        }
        value = Null;
        return false;
    }

    /// <summary>Returns the inner string or throws <see cref="InvalidOperationException"/>.</summary>
    public string AsStr()
    {
        if (TryAsStr(out var v)) return v;
        throw new InvalidOperationException($"VectorizerValue is {Kind}, not Str");
    }

    /// <summary>Returns the inner long or throws <see cref="InvalidOperationException"/>.</summary>
    public long AsInt()
    {
        if (TryAsInt(out var v)) return v;
        throw new InvalidOperationException($"VectorizerValue is {Kind}, not Int");
    }

    /// <summary>Returns the inner double or throws <see cref="InvalidOperationException"/>.</summary>
    public double AsFloat()
    {
        if (TryAsFloat(out var v)) return v;
        throw new InvalidOperationException($"VectorizerValue is {Kind}, not Float/Int");
    }

    public IReadOnlyList<VectorizerValue> AsArray()
    {
        if (TryAsArray(out var v)) return v;
        throw new InvalidOperationException($"VectorizerValue is {Kind}, not Array");
    }

    public IReadOnlyList<MapPair> AsMap()
    {
        if (TryAsMap(out var v)) return v;
        throw new InvalidOperationException($"VectorizerValue is {Kind}, not Map");
    }

    // ── Wire mapping ────────────────────────────────────────────────────
    // rmp-serde encodes an externally-tagged enum as:
    //   unit variant  → bare string  "Null"
    //   newtype variant → map { "<Tag>": payload }
    // We round-trip through plain object graphs so MessagePackSerializer
    // (with ContractlessStandardResolver) produces the same bytes.

    internal const string TagNull = "Null";
    internal const string TagBool = "Bool";
    internal const string TagInt = "Int";
    internal const string TagFloat = "Float";
    internal const string TagBytes = "Bytes";
    internal const string TagStr = "Str";
    internal const string TagArray = "Array";
    internal const string TagMap = "Map";
    internal const string ResultOk = "Ok";
    internal const string ResultErr = "Err";

    /// <summary>
    /// Converts this value into a tree of plain objects
    /// (<see cref="Dictionary{TKey, TValue}"/>, <see cref="Array"/>, primitives)
    /// ready for <see cref="MessagePack.MessagePackSerializer"/>.
    /// </summary>
    public object ToWire()
    {
        switch (Kind)
        {
            case ValueKind.Null:
                return TagNull;
            case ValueKind.Bool:
                return new Dictionary<string, object?> { [TagBool] = _bool };
            case ValueKind.Int:
                return new Dictionary<string, object?> { [TagInt] = _int };
            case ValueKind.Float:
                return new Dictionary<string, object?> { [TagFloat] = _float };
            case ValueKind.Bytes:
                return new Dictionary<string, object?> { [TagBytes] = _bytes! };
            case ValueKind.Str:
                return new Dictionary<string, object?> { [TagStr] = _str! };
            case ValueKind.Array:
            {
                var arr = new object?[_array!.Count];
                for (var i = 0; i < arr.Length; i++)
                {
                    arr[i] = _array[i].ToWire();
                }
                return new Dictionary<string, object?> { [TagArray] = arr };
            }
            case ValueKind.Map:
            {
                var pairs = new object?[_map!.Count];
                for (var i = 0; i < pairs.Length; i++)
                {
                    var (k, v) = _map[i];
                    pairs[i] = new object?[] { k.ToWire(), v.ToWire() };
                }
                return new Dictionary<string, object?> { [TagMap] = pairs };
            }
            default:
                throw new InvalidOperationException($"unsupported VectorizerValue kind: {Kind}");
        }
    }

    /// <summary>
    /// Decodes a tree of plain objects (as produced by
    /// <see cref="MessagePack.MessagePackSerializer"/> on the wire body)
    /// back into a typed <see cref="VectorizerValue"/>.
    /// </summary>
    public static VectorizerValue FromWire(object? raw)
    {
        if (raw is string s)
        {
            if (s == TagNull) return Null;
            throw new InvalidOperationException(
                $"unknown VectorizerValue unit-variant tag: '{s}'");
        }

        if (raw is not IDictionary<object, object?> objMap)
        {
            if (raw is IDictionary<string, object?> strMap)
            {
                objMap = ToObjectKeyMap(strMap);
            }
            else
            {
                throw new InvalidOperationException(
                    $"expected externally-tagged map or 'Null', got {raw?.GetType().FullName ?? "null"}");
            }
        }

        if (objMap.Count != 1)
        {
            throw new InvalidOperationException(
                $"externally-tagged value must have exactly one key, got {objMap.Count}");
        }

        foreach (var kvp in objMap)
        {
            var tag = kvp.Key as string
                ?? throw new InvalidOperationException(
                    $"externally-tagged key must be string, got {kvp.Key?.GetType().FullName ?? "null"}");
            var payload = kvp.Value;
            return DecodeTagged(tag, payload);
        }

        throw new InvalidOperationException("unreachable");
    }

    private static VectorizerValue DecodeTagged(string tag, object? payload)
    {
        switch (tag)
        {
            case TagBool:
                return OfBool(CoerceBool(payload, tag));
            case TagInt:
                return OfInt(CoerceInt(payload, tag));
            case TagFloat:
                return OfFloat(CoerceFloat(payload, tag));
            case TagBytes:
                return OfBytes(CoerceBytes(payload, tag));
            case TagStr:
                return OfStr(CoerceStr(payload, tag));
            case TagArray:
            {
                var arr = CoerceArray(payload, tag);
                var items = new VectorizerValue[arr.Length];
                for (var i = 0; i < arr.Length; i++) items[i] = FromWire(arr[i]);
                return OfArray(items);
            }
            case TagMap:
            {
                var arr = CoerceArray(payload, tag);
                var pairs = new MapPair[arr.Length];
                for (var i = 0; i < arr.Length; i++)
                {
                    if (arr[i] is not object?[] entry || entry.Length != 2)
                    {
                        throw new InvalidOperationException(
                            $"Map[{i}] must be a 2-element array");
                    }
                    pairs[i] = new MapPair(FromWire(entry[0]), FromWire(entry[1]));
                }
                return OfMap(pairs);
            }
            default:
                throw new InvalidOperationException($"unknown VectorizerValue tag: '{tag}'");
        }
    }

    private static IDictionary<object, object?> ToObjectKeyMap(IDictionary<string, object?> src)
    {
        var dst = new Dictionary<object, object?>(src.Count);
        foreach (var kvp in src) dst[kvp.Key] = kvp.Value;
        return dst;
    }

    internal static bool CoerceBool(object? p, string ctx) =>
        p is bool b
            ? b
            : throw new InvalidOperationException($"{ctx} payload must be bool, got {p?.GetType().FullName ?? "null"}");

    internal static long CoerceInt(object? p, string ctx) => p switch
    {
        sbyte x => x,
        short x => x,
        int x => x,
        long x => x,
        byte x => x,
        ushort x => x,
        uint x => x,
        ulong x => checked((long)x),
        _ => throw new InvalidOperationException(
            $"{ctx} payload must be integer, got {p?.GetType().FullName ?? "null"}"),
    };

    internal static double CoerceFloat(object? p, string ctx) => p switch
    {
        float x => x,
        double x => x,
        _ => throw new InvalidOperationException(
            $"{ctx} payload must be float, got {p?.GetType().FullName ?? "null"}"),
    };

    internal static byte[] CoerceBytes(object? p, string ctx) => p switch
    {
        byte[] b => b,
        ReadOnlyMemory<byte> rom => rom.ToArray(),
        _ => throw new InvalidOperationException(
            $"{ctx} payload must be byte[], got {p?.GetType().FullName ?? "null"}"),
    };

    internal static string CoerceStr(object? p, string ctx) => p switch
    {
        string s => s,
        _ => throw new InvalidOperationException(
            $"{ctx} payload must be string, got {p?.GetType().FullName ?? "null"}"),
    };

    internal static object?[] CoerceArray(object? p, string ctx) => p switch
    {
        object?[] a => a,
        System.Collections.IList list => ToObjectArray(list),
        _ => throw new InvalidOperationException(
            $"{ctx} payload must be array, got {p?.GetType().FullName ?? "null"}"),
    };

    private static object?[] ToObjectArray(System.Collections.IList list)
    {
        var arr = new object?[list.Count];
        for (var i = 0; i < list.Count; i++) arr[i] = list[i];
        return arr;
    }

    // ── Equality / formatting ──────────────────────────────────────────

    public bool Equals(VectorizerValue? other)
    {
        if (other is null || other.Kind != Kind) return false;
        switch (Kind)
        {
            case ValueKind.Null: return true;
            case ValueKind.Bool: return _bool == other._bool;
            case ValueKind.Int: return _int == other._int;
            case ValueKind.Float: return _float.Equals(other._float);
            case ValueKind.Bytes: return BytesEqual(_bytes!, other._bytes!);
            case ValueKind.Str: return string.Equals(_str, other._str, StringComparison.Ordinal);
            case ValueKind.Array:
            {
                if (_array!.Count != other._array!.Count) return false;
                for (var i = 0; i < _array.Count; i++)
                {
                    if (!_array[i].Equals(other._array[i])) return false;
                }
                return true;
            }
            case ValueKind.Map:
            {
                if (_map!.Count != other._map!.Count) return false;
                for (var i = 0; i < _map.Count; i++)
                {
                    var (la, lb) = _map[i];
                    var (ra, rb) = other._map[i];
                    if (!la.Equals(ra) || !lb.Equals(rb)) return false;
                }
                return true;
            }
            default: return false;
        }
    }

    public override bool Equals(object? obj) => obj is VectorizerValue v && Equals(v);

    public override int GetHashCode() => Kind switch
    {
        ValueKind.Null => 0,
        ValueKind.Bool => _bool.GetHashCode(),
        ValueKind.Int => _int.GetHashCode(),
        ValueKind.Float => _float.GetHashCode(),
        ValueKind.Str => StringComparer.Ordinal.GetHashCode(_str ?? string.Empty),
        _ => Kind.GetHashCode(),
    };

    public override string ToString() => Kind switch
    {
        ValueKind.Null => "Null",
        ValueKind.Bool => _bool ? "Bool(true)" : "Bool(false)",
        ValueKind.Int => $"Int({_int.ToString(CultureInfo.InvariantCulture)})",
        ValueKind.Float => $"Float({_float.ToString(CultureInfo.InvariantCulture)})",
        ValueKind.Bytes => $"Bytes[{_bytes!.Length}]",
        ValueKind.Str => $"Str({_str})",
        ValueKind.Array => $"Array[{_array!.Count}]",
        ValueKind.Map => $"Map[{_map!.Count}]",
        _ => "?",
    };

    private static bool BytesEqual(byte[] a, byte[] b)
    {
        if (a.Length != b.Length) return false;
        for (var i = 0; i < a.Length; i++) if (a[i] != b[i]) return false;
        return true;
    }
}
