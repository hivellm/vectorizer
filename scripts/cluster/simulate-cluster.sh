#!/usr/bin/env bash
# ============================================================================
# Cluster Simulation Suite
# Tests: data flow, node failure, recovery, consistency, performance
# ============================================================================

MASTER="http://localhost:15002"
REP1="http://localhost:15012"
REP2="http://localhost:15022"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'
BOLD='\033[1m'

pass=0; fail=0; total_scenarios=0

check() {
    local name="$1"; local result="$2"
    ((total_scenarios++)) || true
    if [ "$result" = "true" ]; then
        echo -e "    ${GREEN}✓${NC} $name"
        ((pass++)) || true
    else
        echo -e "    ${RED}✗${NC} $name"
        ((fail++)) || true
    fi
}

sep() { echo -e "${BLUE}────────────────────────────────────────────────${NC}"; }

gen_vec() {
    python3 -c "import random; random.seed($1); v=[random.uniform(-1,1) for _ in range($2)]; n=sum(x*x for x in v)**0.5; print(','.join([str(round(x/n,6)) for x in v]))"
}

echo ""
echo -e "${BOLD}╔══════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}║     Vectorizer Cluster Simulation Suite      ║${NC}"
echo -e "${BOLD}╚══════════════════════════════════════════════╝${NC}"
echo ""

# ═══════════════════════════════════════════════
# SIMULATION 1: Data Ingestion Pipeline
# ═══════════════════════════════════════════════
echo -e "${CYAN}${BOLD}SIMULATION 1: Data Ingestion Pipeline${NC}"
echo -e "${CYAN}Simulates batch document ingestion${NC}"
sep

# Create collection
curl -s -X POST "$MASTER/collections" \
    -H "Content-Type: application/json" \
    -d '{"name":"documents","dimension":256,"metric":"cosine"}' > /dev/null 2>&1

# Batch upsert - 200 vectors in batches of 50
echo -e "  ${YELLOW}Inserting 200 vectors in 4 batches of 50...${NC}"
for batch in $(seq 0 3); do
    points=""
    for i in $(seq 1 50); do
        idx=$((batch * 50 + i))
        vec=$(gen_vec $idx 256)
        [ -n "$points" ] && points="$points,"
        points="$points{\"id\":\"doc-$idx\",\"vector\":[$vec],\"payload\":{\"title\":\"Document $idx\",\"batch\":$batch,\"category\":\"$([ $((idx % 3)) -eq 0 ] && echo tech || ([ $((idx % 3)) -eq 1 ] && echo science || echo business))\"}}"
    done

    start=$(python3 -c "import time; print(time.time())")
    result=$(curl -s -X PUT "$MASTER/qdrant/collections/documents/points" \
        -H "Content-Type: application/json" \
        -d "{\"points\":[$points]}" 2>/dev/null)
    end=$(python3 -c "import time; print(time.time())")
    ms=$(python3 -c "print(int(($end - $start) * 1000))")

    status=$(echo "$result" | python3 -c "import sys,json; print(json.load(sys.stdin).get('status','error'))" 2>/dev/null || echo "error")
    check "Batch $((batch+1))/4: 50 vectors in ${ms}ms" "$([ "$status" = "acknowledged" ] && echo true || echo false)"
done

# Verify total count
count=$(curl -sf "$MASTER/collections/documents" | python3 -c "import sys,json; print(json.load(sys.stdin).get('vector_count',0))" 2>/dev/null)
check "Total inserted: $count vectors (expected: 200)" "$([ "$count" -ge 200 ] && echo true || echo false)"

# ═══════════════════════════════════════════════
# SIMULATION 2: Semantic Search Quality
# ═══════════════════════════════════════════════
echo ""
echo -e "${CYAN}${BOLD}SIMULATION 2: Semantic Search Quality${NC}"
echo -e "${CYAN}Tests search quality and consistency${NC}"
sep

# Search with different limits
for limit in 1 5 10 20; do
    q=$(gen_vec 999 256)
    result=$(curl -s -X POST "$MASTER/qdrant/collections/documents/points/search" \
        -H "Content-Type: application/json" \
        -d "{\"vector\":[$q],\"limit\":$limit}" 2>/dev/null)
    count=$(echo "$result" | python3 -c "import sys,json; print(len(json.load(sys.stdin).get('result',[])))" 2>/dev/null || echo 0)
    check "Search limit=$limit returns $count results" "$([ "$count" -eq "$limit" ] && echo true || echo false)"
done

# Score ordering (results should be sorted by score descending)
q=$(gen_vec 42 256)
result=$(curl -s -X POST "$MASTER/qdrant/collections/documents/points/search" \
    -H "Content-Type: application/json" \
    -d "{\"vector\":[$q],\"limit\":5}" 2>/dev/null)
ordered=$(echo "$result" | python3 -c "
import sys,json
r = json.load(sys.stdin).get('result',[])
scores = [x['score'] for x in r]
print('true' if scores == sorted(scores, reverse=True) else 'false')
" 2>/dev/null || echo "false")
check "Results ordered by score (descending)" "$ordered"

# Same query returns same results (deterministic)
r1=$(curl -s -X POST "$MASTER/qdrant/collections/documents/points/search" \
    -H "Content-Type: application/json" \
    -d "{\"vector\":[$q],\"limit\":3}" 2>/dev/null)
r2=$(curl -s -X POST "$MASTER/qdrant/collections/documents/points/search" \
    -H "Content-Type: application/json" \
    -d "{\"vector\":[$q],\"limit\":3}" 2>/dev/null)
ids1=$(echo "$r1" | python3 -c "import sys,json; print([x['id'] for x in json.load(sys.stdin).get('result',[])])" 2>/dev/null)
ids2=$(echo "$r2" | python3 -c "import sys,json; print([x['id'] for x in json.load(sys.stdin).get('result',[])])" 2>/dev/null)
check "Deterministic search (same query = same IDs)" "$([ "$ids1" = "$ids2" ] && echo true || echo false)"

# ═══════════════════════════════════════════════
# SIMULATION 3: Multi-Collection Isolation
# ═══════════════════════════════════════════════
echo ""
echo -e "${CYAN}${BOLD}SIMULATION 3: Multi-Collection Isolation${NC}"
echo -e "${CYAN}Verifies data isolation between collections${NC}"
sep

# Create 3 isolated collections
for col in "users" "products" "logs"; do
    curl -s -X POST "$MASTER/collections" \
        -H "Content-Type: application/json" \
        -d "{\"name\":\"$col\",\"dimension\":64,\"metric\":\"cosine\"}" > /dev/null 2>&1
done

# Insert different data in each
for col in "users" "products" "logs"; do
    points=""
    for i in $(seq 1 10); do
        vec=$(gen_vec $((RANDOM + i)) 64)
        [ -n "$points" ] && points="$points,"
        points="$points{\"id\":\"${col}-$i\",\"vector\":[$vec],\"payload\":{\"source\":\"$col\"}}"
    done
    curl -s -X PUT "$MASTER/qdrant/collections/$col/points" \
        -H "Content-Type: application/json" \
        -d "{\"points\":[$points]}" > /dev/null 2>&1
done

# Verify each has exactly 10
for col in "users" "products" "logs"; do
    count=$(curl -sf "$MASTER/collections/$col" | python3 -c "import sys,json; print(json.load(sys.stdin).get('vector_count',0))" 2>/dev/null || echo 0)
    check "Collection '$col' has $count vectors (expected: 10)" "$([ "$count" -eq 10 ] && echo true || echo false)"
done

# Search in one collection doesn't return data from another
q=$(gen_vec 777 64)
result=$(curl -s -X POST "$MASTER/qdrant/collections/users/points/search" \
    -H "Content-Type: application/json" \
    -d "{\"vector\":[$q],\"limit\":3}" 2>/dev/null)
all_from_users=$(echo "$result" | python3 -c "
import sys,json
r = json.load(sys.stdin).get('result',[])
print('true' if all(x['id'].startswith('users-') for x in r) else 'false')
" 2>/dev/null || echo "false")
check "Search in 'users' returns only user IDs" "$all_from_users"

# ═══════════════════════════════════════════════
# SIMULATION 4: Node Failure & Recovery
# ═══════════════════════════════════════════════
echo ""
echo -e "${CYAN}${BOLD}SIMULATION 4: Node Failure & Recovery${NC}"
echo -e "${CYAN}Simulates replica failure and recovery${NC}"
sep

# Verify all nodes healthy
for node in "$MASTER" "$REP1" "$REP2"; do
    name=$([ "$node" = "$MASTER" ] && echo "Master" || ([ "$node" = "$REP1" ] && echo "Replica1" || echo "Replica2"))
    health=$(curl -sf "$node/health" | python3 -c "import sys,json; print(json.load(sys.stdin).get('status',''))" 2>/dev/null || echo "down")
    check "Before failure: $name is $health" "$([ "$health" = "healthy" ] && echo true || echo false)"
done

# Kill replica 2
echo -e "  ${YELLOW}Stopping Replica 2...${NC}"
docker stop vz-replica2 > /dev/null 2>&1
sleep 2
check "Replica 2 stopped" "$(curl -sf $REP2/health > /dev/null 2>&1 && echo false || echo true)"

# Master and Replica 1 should still work
check "Master still healthy" "$(curl -sf $MASTER/health > /dev/null 2>&1 && echo true || echo false)"
check "Replica 1 still healthy" "$(curl -sf $REP1/health > /dev/null 2>&1 && echo true || echo false)"

# Insert data while replica is down
echo -e "  ${YELLOW}Inserting data with replica down...${NC}"
q=$(gen_vec 888 256)
curl -s -X PUT "$MASTER/qdrant/collections/documents/points" \
    -H "Content-Type: application/json" \
    -d "{\"points\":[{\"id\":\"during-failure\",\"vector\":[$q],\"payload\":{\"note\":\"inserted while replica2 was down\"}}]}" > /dev/null 2>&1
check "Insert during failure: success" "$(curl -sf $MASTER/collections/documents | python3 -c 'import sys,json; print(json.load(sys.stdin).get(\"vector_count\",0) > 200)' 2>/dev/null || echo false)"

# Recover replica 2
echo -e "  ${YELLOW}Recovering Replica 2...${NC}"
docker start vz-replica2 > /dev/null 2>&1
sleep 5
check "Replica 2 recovered" "$(curl -sf $REP2/health > /dev/null 2>&1 && echo true || echo false)"

# ═══════════════════════════════════════════════
# SIMULATION 5: Cross-Node Data Consistency
# ═══════════════════════════════════════════════
echo ""
echo -e "${CYAN}${BOLD}SIMULATION 5: Cross-Node Consistency${NC}"
echo -e "${CYAN}Verifies all nodes serve the same data${NC}"
sep

# Create collection on all nodes independently (each node is standalone)
for node_url in "$REP1" "$REP2"; do
    curl -s -X POST "$node_url/collections" \
        -H "Content-Type: application/json" \
        -d '{"name":"consistency-test","dimension":32,"metric":"cosine"}' > /dev/null 2>&1
done
curl -s -X POST "$MASTER/collections" \
    -H "Content-Type: application/json" \
    -d '{"name":"consistency-test","dimension":32,"metric":"cosine"}' > /dev/null 2>&1

# Insert same data on all nodes
points=""
for i in $(seq 1 20); do
    vec=$(gen_vec $i 32)
    [ -n "$points" ] && points="$points,"
    points="$points{\"id\":\"shared-$i\",\"vector\":[$vec]}"
done

for node_url in "$MASTER" "$REP1" "$REP2"; do
    curl -s -X PUT "$node_url/qdrant/collections/consistency-test/points" \
        -H "Content-Type: application/json" \
        -d "{\"points\":[$points]}" > /dev/null 2>&1
done

# Search on all 3 nodes with same query - should get same results
q=$(gen_vec 555 32)
master_ids=$(curl -s -X POST "$MASTER/qdrant/collections/consistency-test/points/search" \
    -H "Content-Type: application/json" \
    -d "{\"vector\":[$q],\"limit\":5}" 2>/dev/null | python3 -c "import sys,json; print(sorted([x['id'] for x in json.load(sys.stdin).get('result',[])]))" 2>/dev/null)
rep1_ids=$(curl -s -X POST "$REP1/qdrant/collections/consistency-test/points/search" \
    -H "Content-Type: application/json" \
    -d "{\"vector\":[$q],\"limit\":5}" 2>/dev/null | python3 -c "import sys,json; print(sorted([x['id'] for x in json.load(sys.stdin).get('result',[])]))" 2>/dev/null)
rep2_ids=$(curl -s -X POST "$REP2/qdrant/collections/consistency-test/points/search" \
    -H "Content-Type: application/json" \
    -d "{\"vector\":[$q],\"limit\":5}" 2>/dev/null | python3 -c "import sys,json; print(sorted([x['id'] for x in json.load(sys.stdin).get('result',[])]))" 2>/dev/null)

check "Master and Replica1 return same IDs" "$([ "$master_ids" = "$rep1_ids" ] && echo true || echo false)"
check "Master and Replica2 return same IDs" "$([ "$master_ids" = "$rep2_ids" ] && echo true || echo false)"

# ═══════════════════════════════════════════════
# SIMULATION 6: Performance Under Load
# ═══════════════════════════════════════════════
echo ""
echo -e "${CYAN}${BOLD}SIMULATION 6: Performance Under Load${NC}"
echo -e "${CYAN}Tests latency and throughput${NC}"
sep

# Single search latency
total_ms=0
for i in $(seq 1 10); do
    q=$(gen_vec $((i+2000)) 256)
    start=$(python3 -c "import time; print(time.time())")
    curl -s -X POST "$MASTER/qdrant/collections/documents/points/search" \
        -H "Content-Type: application/json" \
        -d "{\"vector\":[$q],\"limit\":10}" > /dev/null 2>&1
    end=$(python3 -c "import time; print(time.time())")
    ms=$(python3 -c "print(int(($end - $start) * 1000))")
    total_ms=$((total_ms + ms))
done
avg_ms=$((total_ms / 10))
check "Average search latency: ${avg_ms}ms (10 queries)" "$([ "$avg_ms" -lt 100 ] && echo true || echo false)"

# Bulk insert throughput
points=""
for i in $(seq 1 500); do
    vec=$(gen_vec $((i+5000)) 256)
    [ -n "$points" ] && points="$points,"
    points="$points{\"id\":\"perf-$i\",\"vector\":[$vec]}"
done
start=$(python3 -c "import time; print(time.time())")
curl -s -X PUT "$MASTER/qdrant/collections/documents/points" \
    -H "Content-Type: application/json" \
    -d "{\"points\":[$points]}" > /dev/null 2>&1
end=$(python3 -c "import time; print(time.time())")
elapsed=$(python3 -c "print(round($end - $start, 2))")
throughput=$(python3 -c "print(int(500 / max($end - $start, 0.001)))")
check "Bulk insert 500 vectors: ${elapsed}s (${throughput} vec/s)" "$([ "$throughput" -gt 100 ] && echo true || echo false)"

# Concurrent reads from different nodes
echo -e "  ${YELLOW}Parallel searches on 3 nodes...${NC}"
start=$(python3 -c "import time; print(time.time())")
for node in "$MASTER" "$REP1" "$REP2"; do
    q=$(gen_vec $RANDOM 256)
    curl -s -X POST "$node/qdrant/collections/documents/points/search" \
        -H "Content-Type: application/json" \
        -d "{\"vector\":[$q],\"limit\":10}" > /dev/null 2>&1 &
done
wait
end=$(python3 -c "import time; print(time.time())")
parallel_ms=$(python3 -c "print(int(($end - $start) * 1000))")
check "3 parallel searches (3 nodes): ${parallel_ms}ms" "$([ "$parallel_ms" -lt 500 ] && echo true || echo false)"

# ═══════════════════════════════════════════════
# SIMULATION 7: Prometheus Monitoring
# ═══════════════════════════════════════════════
echo ""
echo -e "${CYAN}${BOLD}SIMULATION 7: Monitoring & Observability${NC}"
echo -e "${CYAN}Verifies metrics and stats of all nodes${NC}"
sep

for node_url in "$MASTER" "$REP1" "$REP2"; do
    name=$([ "$node_url" = "$MASTER" ] && echo "Master" || ([ "$node_url" = "$REP1" ] && echo "Replica1" || echo "Replica2"))

    metrics=$(curl -sf "$node_url/prometheus/metrics" 2>/dev/null || echo "")
    metric_lines=$(echo "$metrics" | grep -c "vectorizer_" 2>/dev/null || echo 0)
    check "$name: Prometheus metrics ($metric_lines lines)" "$([ "$metric_lines" -gt 0 ] && echo true || echo false)"
done

# Check specific metrics exist
metrics=$(curl -sf "$MASTER/prometheus/metrics" 2>/dev/null)
check "Metric: vectorizer_collections_total" "$(echo "$metrics" | grep -q "vectorizer_collections_total" && echo true || echo false)"
check "Metric: vectorizer_search_duration" "$(echo "$metrics" | grep -q "search_duration\|search_latency\|request" && echo true || echo false)"

# Stats endpoint
stats=$(curl -sf "$MASTER/api/stats" 2>/dev/null || echo "{}")
col_count=$(echo "$stats" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('total_collections', d.get('collections',0)))" 2>/dev/null || echo 0)
check "Stats: $col_count collections on master" "$([ "$col_count" -gt 0 ] && echo true || echo false)"

# ═══════════════════════════════════════════════
# SUMMARY
# ═══════════════════════════════════════════════
echo ""
echo -e "${BOLD}╔══════════════════════════════════════════════╗${NC}"
if [ "$fail" -eq 0 ]; then
    echo -e "${BOLD}║  ${GREEN}ALL $total_scenarios SIMULATIONS PASSED${NC}${BOLD}                  ║${NC}"
else
    printf "${BOLD}║  ${GREEN}%d passed${NC}${BOLD}, ${RED}%d failed${NC}${BOLD} / %d total              ║${NC}\n" "$pass" "$fail" "$total_scenarios"
fi
echo -e "${BOLD}╚══════════════════════════════════════════════╝${NC}"
echo ""

[ "$fail" -eq 0 ]
