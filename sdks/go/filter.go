package vectorizer

// QdrantFilter mirrors the server's filter wire shape (phase23).
//
// All three clause arrays are optional; omit any you don't need. At least
// one clause with at least one condition must be present — the server
// rejects an all-absent filter with 400 validation_error.
//
// Wire shape:
//
//	{"must": [...], "should": [...], "must_not": [...]}
type QdrantFilter struct {
	Must    []FilterCondition `json:"must,omitempty"`
	Should  []FilterCondition `json:"should,omitempty"`
	MustNot []FilterCondition `json:"must_not,omitempty"`
}

// FilterCondition is a single Qdrant-style filter predicate. Exactly one of
// Match, Range, or Filter should be set alongside Key (or Key may be empty
// for nested sub-filter conditions).
type FilterCondition struct {
	Key    string        `json:"key"`
	Match  *FilterMatch  `json:"match,omitempty"`
	Range  *FilterRange  `json:"range,omitempty"`
	Filter *QdrantFilter `json:"filter,omitempty"`
}

// FilterMatch represents an exact-value or multi-value match predicate.
// Set Value for a single-value equality check, or Any for a set-membership
// ("in") check. Setting both is undefined behaviour.
type FilterMatch struct {
	Value interface{}   `json:"value,omitempty"`
	Any   []interface{} `json:"any,omitempty"`
}

// FilterRange represents a numeric range predicate. Both bounds are optional;
// omitting a bound removes that side of the range.
type FilterRange struct {
	Gte *float64 `json:"gte,omitempty"`
	Lte *float64 `json:"lte,omitempty"`
}

// IsEmpty reports true when ALL three condition arrays are empty or absent.
// An empty filter is rejected by the server with 400 validation_error; the
// typed DeleteByFilter / BulkUpdateMetadata helpers also reject it
// client-side before issuing an HTTP request.
func (f *QdrantFilter) IsEmpty() bool {
	return len(f.Must) == 0 && len(f.Should) == 0 && len(f.MustNot) == 0
}

// FilterEq returns a single-equality match condition.
//
// Example: FilterEq("topic", "index") serialises to
//
//	{"key":"topic","match":{"value":"index"}}
func FilterEq(key string, value interface{}) FilterCondition {
	return FilterCondition{
		Key:   key,
		Match: &FilterMatch{Value: value},
	}
}

// FilterIn returns a multi-value match condition ("value is one of ...").
//
// Example: FilterIn("tier", []interface{}{"hot","warm"}) serialises to
//
//	{"key":"tier","match":{"any":["hot","warm"]}}
func FilterIn(key string, values []interface{}) FilterCondition {
	return FilterCondition{
		Key:   key,
		Match: &FilterMatch{Any: values},
	}
}

// FilterRangeGteLte returns a range condition with optional lower and upper
// bounds. Pass nil for either bound to leave that side open.
//
// Example:
//
//	gte, lte := 0.5, 0.9
//	FilterRangeGteLte("score", &gte, &lte)
//
// serialises to {"key":"score","range":{"gte":0.5,"lte":0.9}}.
func FilterRangeGteLte(key string, gte, lte *float64) FilterCondition {
	return FilterCondition{
		Key:   key,
		Range: &FilterRange{Gte: gte, Lte: lte},
	}
}

// FilterNested wraps a sub-filter under a condition's filter slot.
// The key field is left empty, matching the Qdrant nested-filter convention.
//
// Example: FilterNested(MustFilter(FilterEq("k","v"))) serialises to
//
//	{"key":"","filter":{"must":[{"key":"k","match":{"value":"v"}}]}}
func FilterNested(sub QdrantFilter) FilterCondition {
	return FilterCondition{
		Filter: &sub,
	}
}

// MustFilter builds a QdrantFilter that requires ALL given conditions to be
// true (logical AND). The Should and MustNot arrays are omitted.
func MustFilter(conditions ...FilterCondition) QdrantFilter {
	return QdrantFilter{Must: conditions}
}

// ShouldFilter builds a QdrantFilter that requires AT LEAST ONE of the given
// conditions to be true (logical OR). The Must and MustNot arrays are
// omitted.
func ShouldFilter(conditions ...FilterCondition) QdrantFilter {
	return QdrantFilter{Should: conditions}
}

// MustNotFilter builds a QdrantFilter that requires ALL given conditions to
// be false (logical NOT). The Must and Should arrays are omitted.
func MustNotFilter(conditions ...FilterCondition) QdrantFilter {
	return QdrantFilter{MustNot: conditions}
}
