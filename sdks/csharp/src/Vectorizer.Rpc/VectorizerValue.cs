using System;
using System.Collections.Generic;
using System.Globalization;

// Aliased rather than imported: HiveLLM.Thunder also declares `Value`,
// `ValueKind` and `MapPair`-shaped types, and this file defines its own.
using Thunder = HiveLLM.Thunder;

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

    // ── Thunder wire conversion ──────────────────────────────────
    // The bytes on the wire are Thunder's (HiveLLM.Thunder), the shared binary
    // RPC package whose Rust twin vectorizer-server runs, so encoding is not
    // implemented here. These two methods are the only seam between the two
    // value models, and the mapping is total: every variant has an exact
    // counterpart, so nothing is lost in either direction.

    /// <summary>
    /// Converts this value into Thunder's value model for transmission.
    /// </summary>
    public Thunder.Value ToThunder()
    {
        switch (Kind)
        {
            case ValueKind.Null:
                return Thunder.Value.Null;
            case ValueKind.Bool:
                return Thunder.Value.Bool(_bool);
            case ValueKind.Int:
                return Thunder.Value.Int(_int);
            case ValueKind.Float:
                return Thunder.Value.Float(_float);
            case ValueKind.Bytes:
                return Thunder.Value.Bytes(_bytes!);
            case ValueKind.Str:
                return Thunder.Value.Str(_str!);
            case ValueKind.Array:
            {
                var items = new Thunder.Value[_array!.Count];
                for (var i = 0; i < items.Length; i++)
                {
                    items[i] = _array[i].ToThunder();
                }
                return Thunder.Value.Array(items);
            }
            case ValueKind.Map:
            {
                var pairs = new (Thunder.Value, Thunder.Value)[_map!.Count];
                for (var i = 0; i < pairs.Length; i++)
                {
                    var (k, v) = _map[i];
                    pairs[i] = (k.ToThunder(), v.ToThunder());
                }
                return Thunder.Value.Map(pairs);
            }
            default:
                throw new InvalidOperationException($"unsupported VectorizerValue kind: {Kind}");
        }
    }

    /// <summary>
    /// Converts a Thunder value back into a typed <see cref="VectorizerValue"/>.
    /// </summary>
    public static VectorizerValue FromThunder(Thunder.Value value)
    {
        switch (value.Kind)
        {
            case Thunder.ValueKind.Null:
                return Null;
            case Thunder.ValueKind.Bool:
                return OfBool(value.AsBool() ?? false);
            case Thunder.ValueKind.Int:
                return OfInt(value.AsInt() ?? 0L);
            case Thunder.ValueKind.Float:
                return OfFloat(value.AsFloat() ?? 0d);
            case Thunder.ValueKind.Bytes:
                return OfBytes(value.AsBytes() ?? System.Array.Empty<byte>());
            case Thunder.ValueKind.Str:
                return OfStr(value.AsStr() ?? string.Empty);
            case Thunder.ValueKind.Array:
            {
                var src = value.AsArray() ?? (IReadOnlyList<Thunder.Value>)System.Array.Empty<Thunder.Value>();
                var items = new VectorizerValue[src.Count];
                for (var i = 0; i < items.Length; i++)
                {
                    items[i] = FromThunder(src[i]);
                }
                return OfArray(items);
            }
            case Thunder.ValueKind.Map:
            {
                var src = value.AsMap()
                    ?? (IReadOnlyList<KeyValuePair<Thunder.Value, Thunder.Value>>)
                        System.Array.Empty<KeyValuePair<Thunder.Value, Thunder.Value>>();
                var pairs = new MapPair[src.Count];
                for (var i = 0; i < pairs.Length; i++)
                {
                    pairs[i] = new MapPair(FromThunder(src[i].Key), FromThunder(src[i].Value));
                }
                return OfMap(pairs);
            }
            default:
                throw new InvalidOperationException($"unsupported Thunder value kind: {value.Kind}");
        }
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
