"""Typed QdrantFilter helpers for the Vectorizer Python SDK (phase23).

Matches the wire shape of the Rust + TypeScript SDKs exactly:

.. code-block:: json

    {
        "must":     [<Condition>...],
        "should":   [<Condition>...],
        "must_not": [<Condition>...]
    }

Where each ``Condition`` is ``{key, match?, range?, filter?}``:

- ``match``  — exact value (``{"value": any}``) or membership check
  (``{"any": [...]}``)
- ``range``  — numeric bounds (``{"gte": float?, "lte": float?}``)
- ``filter`` — nested ``QdrantFilter`` sub-object

Quick start::

    from vectorizer.filter import QdrantFilter, filter_eq, filter_range, must_filter

    f = must_filter(
        filter_eq("topic", "index"),
        filter_range("score", gte=0.8),
    )
    # f.to_dict() == {"must": [{"key": "topic", "match": {"value": "index"}},
    #                           {"key": "score", "range": {"gte": 0.8}}]}

"""
from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, List, Optional


# ---------------------------------------------------------------------------
# Leaf value types
# ---------------------------------------------------------------------------


@dataclass
class FilterMatch:
    """Exact-value or membership match sub-object.

    Set exactly one of ``value`` (exact match) or ``any`` (membership):

    - ``{"value": <any>}``   — field must equal ``value``
    - ``{"any": [<any>...]}`` — field must equal any element of ``any``
    """

    value: Optional[Any] = None
    any: Optional[List[Any]] = None

    def to_dict(self) -> dict:
        """Serialise to the wire-shape dict, omitting unset fields."""
        out: dict = {}
        if self.value is not None:
            out["value"] = self.value
        if self.any is not None:
            out["any"] = list(self.any)
        return out


@dataclass
class FilterRange:
    """Numeric range bounds sub-object.

    Both bounds are optional and combined with AND semantics:

    - ``{"gte": <float>}``              — lower bound (inclusive)
    - ``{"lte": <float>}``              — upper bound (inclusive)
    - ``{"gte": <float>, "lte": <float>}`` — closed interval
    """

    gte: Optional[float] = None
    lte: Optional[float] = None

    def to_dict(self) -> dict:
        """Serialise to the wire-shape dict, omitting unset bounds."""
        out: dict = {}
        if self.gte is not None:
            out["gte"] = self.gte
        if self.lte is not None:
            out["lte"] = self.lte
        return out


# ---------------------------------------------------------------------------
# Condition
# ---------------------------------------------------------------------------


@dataclass
class FilterCondition:
    """A single filter condition attached to a payload field key.

    Exactly one of ``match``, ``range``, or ``filter`` should be set:

    - ``match``  — exact value or membership check on ``key``
    - ``range``  — numeric bounds check on ``key``
    - ``filter`` — nested compound sub-filter (``key`` is ignored by the server)

    Wire shape examples::

        {"key": "topic", "match": {"value": "index"}}
        {"key": "score", "range": {"gte": 0.5, "lte": 0.9}}
        {"key": "__nested__", "filter": {<QdrantFilter>}}
    """

    key: str
    match: Optional[FilterMatch] = None
    range: Optional[FilterRange] = None
    filter: Optional["QdrantFilter"] = None

    def to_dict(self) -> dict:
        """Serialise to the wire-shape dict, omitting unset sub-objects."""
        out: dict = {"key": self.key}
        if self.match is not None:
            out["match"] = self.match.to_dict()
        if self.range is not None:
            out["range"] = self.range.to_dict()
        if self.filter is not None:
            out["filter"] = self.filter.to_dict()
        return out


# ---------------------------------------------------------------------------
# Top-level filter
# ---------------------------------------------------------------------------


@dataclass
class QdrantFilter:
    """Top-level Qdrant-style filter accepted by ``delete_by_filter`` and
    ``bulk_update_metadata``.

    All three clause arrays are optional; omit any you don't need.  At least
    one clause with at least one condition must be present — the server rejects
    an all-absent filter with ``400 validation_error`` (message: "filter has no
    conditions").

    Wire shape::

        {"must": [...], "should": [...], "must_not": [...]}

    Use :func:`is_empty` to validate client-side before issuing the request.
    """

    must: Optional[List[FilterCondition]] = None
    should: Optional[List[FilterCondition]] = None
    must_not: Optional[List[FilterCondition]] = None

    def to_dict(self) -> dict:
        """Serialise to the wire-shape dict, omitting empty/absent clauses."""
        out: dict = {}
        if self.must:
            out["must"] = [c.to_dict() for c in self.must]
        if self.should:
            out["should"] = [c.to_dict() for c in self.should]
        if self.must_not:
            out["must_not"] = [c.to_dict() for c in self.must_not]
        return out

    def is_empty(self) -> bool:
        """Return ``True`` when none of the three clause lists has any conditions."""
        return not (self.must or self.should or self.must_not)


# ---------------------------------------------------------------------------
# Builder helpers
# ---------------------------------------------------------------------------


def filter_eq(key: str, value: Any) -> FilterCondition:
    """Build an exact-match condition.

    Wire: ``{"key": "<key>", "match": {"value": <value>}}``

    Args:
        key:   Payload field path (dot-separated for nested fields).
        value: Value the field must equal.
    """
    return FilterCondition(key=key, match=FilterMatch(value=value))


def filter_in(key: str, values: List[Any]) -> FilterCondition:
    """Build a multi-value membership condition (field IN [...]).

    Wire: ``{"key": "<key>", "match": {"any": [...]}}``

    Args:
        key:    Payload field path.
        values: List of values; the field must equal any one of them.
    """
    return FilterCondition(key=key, match=FilterMatch(any=list(values)))


def filter_range(
    key: str,
    gte: Optional[float] = None,
    lte: Optional[float] = None,
) -> FilterCondition:
    """Build a numeric range condition.

    Wire: ``{"key": "<key>", "range": {"gte": <num>?, "lte": <num>?}}``

    Args:
        key: Payload field path.
        gte: Lower bound (inclusive).  Omit to leave unbounded.
        lte: Upper bound (inclusive).  Omit to leave unbounded.
    """
    return FilterCondition(key=key, range=FilterRange(gte=gte, lte=lte))


def filter_nested(sub: QdrantFilter) -> FilterCondition:
    """Wrap a sub-filter as a nested condition.

    Wire: ``{"key": "__nested__", "filter": <QdrantFilter>}``

    Args:
        sub: Nested compound filter to embed.
    """
    return FilterCondition(key="__nested__", filter=sub)


def must_filter(*conditions: FilterCondition) -> QdrantFilter:
    """Build a filter requiring ALL given conditions to be true (AND semantics).

    Wire: ``{"must": [...]}``

    Args:
        *conditions: One or more :class:`FilterCondition` instances.
    """
    return QdrantFilter(must=list(conditions))


def should_filter(*conditions: FilterCondition) -> QdrantFilter:
    """Build a filter requiring AT LEAST ONE condition to be true (OR semantics).

    Wire: ``{"should": [...]}``

    Args:
        *conditions: One or more :class:`FilterCondition` instances.
    """
    return QdrantFilter(should=list(conditions))


def must_not_filter(*conditions: FilterCondition) -> QdrantFilter:
    """Build a filter requiring ALL conditions to be false (NOT semantics).

    Wire: ``{"must_not": [...]}``

    Args:
        *conditions: One or more :class:`FilterCondition` instances.
    """
    return QdrantFilter(must_not=list(conditions))
