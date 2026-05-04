package vectorizer

import (
	"encoding/json"
	"testing"
)

// marshalJSON is a test helper that marshals v to a generic map for assertion.
func marshalJSON(t *testing.T, v interface{}) map[string]interface{} {
	t.Helper()
	b, err := json.Marshal(v)
	if err != nil {
		t.Fatalf("json.Marshal failed: %v", err)
	}
	var m map[string]interface{}
	if err := json.Unmarshal(b, &m); err != nil {
		t.Fatalf("json.Unmarshal failed: %v", err)
	}
	return m
}

// TestFilterEqWireShape verifies that FilterEq serialises to the expected
// wire shape: {"key":"topic","match":{"value":"index"}}.
func TestFilterEqWireShape(t *testing.T) {
	cond := FilterEq("topic", "index")
	m := marshalJSON(t, cond)

	if m["key"] != "topic" {
		t.Errorf("key should be 'topic' but got %v", m["key"])
	}
	match, ok := m["match"].(map[string]interface{})
	if !ok {
		t.Fatalf("match should be an object but got %T", m["match"])
	}
	if match["value"] != "index" {
		t.Errorf("match.value should be 'index' but got %v", match["value"])
	}
	if _, exists := m["range"]; exists {
		t.Error("range key should be absent for FilterEq")
	}
	if _, exists := m["filter"]; exists {
		t.Error("filter key should be absent for FilterEq")
	}
}

// TestFilterInWireShape verifies that FilterIn serialises to
// {"key":"tier","match":{"any":["hot","warm"]}}.
func TestFilterInWireShape(t *testing.T) {
	cond := FilterIn("tier", []interface{}{"hot", "warm"})
	m := marshalJSON(t, cond)

	if m["key"] != "tier" {
		t.Errorf("key should be 'tier' but got %v", m["key"])
	}
	match, ok := m["match"].(map[string]interface{})
	if !ok {
		t.Fatalf("match should be an object but got %T", m["match"])
	}
	anyArr, ok := match["any"].([]interface{})
	if !ok {
		t.Fatalf("match.any should be an array but got %T", match["any"])
	}
	if len(anyArr) != 2 {
		t.Fatalf("match.any should have 2 elements but got %d", len(anyArr))
	}
	if anyArr[0] != "hot" {
		t.Errorf("match.any[0] should be 'hot' but got %v", anyArr[0])
	}
	if anyArr[1] != "warm" {
		t.Errorf("match.any[1] should be 'warm' but got %v", anyArr[1])
	}
	if _, exists := match["value"]; exists {
		t.Error("match.value key should be absent for FilterIn")
	}
}

// TestFilterRangeWireShape verifies that FilterRangeGteLte serialises to
// {"key":"score","range":{"gte":0.5,"lte":0.9}}.
func TestFilterRangeWireShape(t *testing.T) {
	gte := 0.5
	lte := 0.9
	cond := FilterRangeGteLte("score", &gte, &lte)
	m := marshalJSON(t, cond)

	if m["key"] != "score" {
		t.Errorf("key should be 'score' but got %v", m["key"])
	}
	rng, ok := m["range"].(map[string]interface{})
	if !ok {
		t.Fatalf("range should be an object but got %T", m["range"])
	}
	if rng["gte"] != 0.5 {
		t.Errorf("range.gte should be 0.5 but got %v", rng["gte"])
	}
	if rng["lte"] != 0.9 {
		t.Errorf("range.lte should be 0.9 but got %v", rng["lte"])
	}
	if _, exists := m["match"]; exists {
		t.Error("match key should be absent for FilterRangeGteLte")
	}
}

// TestQdrantFilterMustOnly verifies that MustFilter serialises with a "must"
// key and omits the empty "should" and "must_not" keys.
func TestQdrantFilterMustOnly(t *testing.T) {
	f := MustFilter(FilterEq("topic", "index"))
	b, err := json.Marshal(f)
	if err != nil {
		t.Fatalf("json.Marshal failed: %v", err)
	}
	var m map[string]interface{}
	if err := json.Unmarshal(b, &m); err != nil {
		t.Fatalf("json.Unmarshal failed: %v", err)
	}

	if _, ok := m["must"]; !ok {
		t.Error("must key should be present")
	}
	if _, ok := m["should"]; ok {
		t.Error("should key should be absent when empty")
	}
	if _, ok := m["must_not"]; ok {
		t.Error("must_not key should be absent when empty")
	}

	mustArr, ok := m["must"].([]interface{})
	if !ok || len(mustArr) == 0 {
		t.Fatalf("must should be a non-empty array but got %v", m["must"])
	}
	cond, ok := mustArr[0].(map[string]interface{})
	if !ok {
		t.Fatalf("must[0] should be an object but got %T", mustArr[0])
	}
	if cond["key"] != "topic" {
		t.Errorf("must[0].key should be 'topic' but got %v", cond["key"])
	}
}

// TestQdrantFilterCompound verifies that a filter with both must and must_not
// conditions serialises both keys while omitting the empty should key.
func TestQdrantFilterCompound(t *testing.T) {
	f := QdrantFilter{
		Must:    []FilterCondition{FilterEq("status", "active")},
		MustNot: []FilterCondition{FilterEq("tier", "cold")},
	}
	b, err := json.Marshal(f)
	if err != nil {
		t.Fatalf("json.Marshal failed: %v", err)
	}
	var m map[string]interface{}
	if err := json.Unmarshal(b, &m); err != nil {
		t.Fatalf("json.Unmarshal failed: %v", err)
	}

	if _, ok := m["must"]; !ok {
		t.Error("must key should be present")
	}
	if _, ok := m["must_not"]; !ok {
		t.Error("must_not key should be present")
	}
	if _, ok := m["should"]; ok {
		t.Error("should key should be absent when empty")
	}
}

// TestNestedFilter verifies that FilterNested produces a condition with a
// "filter" key containing the sub-filter object.
func TestNestedFilter(t *testing.T) {
	inner := MustFilter(FilterEq("inner_key", "value"))
	cond := FilterNested(inner)
	m := marshalJSON(t, cond)

	filterObj, ok := m["filter"].(map[string]interface{})
	if !ok {
		t.Fatalf("filter key should be an object but got %T", m["filter"])
	}
	mustArr, ok := filterObj["must"].([]interface{})
	if !ok || len(mustArr) == 0 {
		t.Fatalf("nested filter.must should be non-empty but got %v", filterObj["must"])
	}
	innerCond, ok := mustArr[0].(map[string]interface{})
	if !ok {
		t.Fatalf("nested must[0] should be an object but got %T", mustArr[0])
	}
	if innerCond["key"] != "inner_key" {
		t.Errorf("nested must[0].key should be 'inner_key' but got %v", innerCond["key"])
	}
}

// TestIsEmpty verifies the IsEmpty method covers all cases.
func TestIsEmpty(t *testing.T) {
	// Zero-value struct must be empty.
	var zero QdrantFilter
	if !zero.IsEmpty() {
		t.Error("zero-value QdrantFilter should be empty")
	}

	// Struct with all explicit empty slices must be empty.
	withEmptySlices := QdrantFilter{
		Must:    []FilterCondition{},
		Should:  []FilterCondition{},
		MustNot: []FilterCondition{},
	}
	if !withEmptySlices.IsEmpty() {
		t.Error("QdrantFilter with all empty slices should be empty")
	}

	// Struct with one populated Must array must NOT be empty.
	withMust := MustFilter(FilterEq("k", "v"))
	if withMust.IsEmpty() {
		t.Error("QdrantFilter with must conditions should not be empty")
	}

	// Struct with one populated Should array must NOT be empty.
	withShould := ShouldFilter(FilterEq("k", "v"))
	if withShould.IsEmpty() {
		t.Error("QdrantFilter with should conditions should not be empty")
	}

	// Struct with one populated MustNot array must NOT be empty.
	withMustNot := MustNotFilter(FilterEq("k", "v"))
	if withMustNot.IsEmpty() {
		t.Error("QdrantFilter with must_not conditions should not be empty")
	}
}
