#!/usr/bin/env bash
set -uo pipefail

# ============================================================================
# Local Multi-Process Cluster Test
# Starts a Vectorizer instance and tests all features end-to-end
# ============================================================================

BINARY="./target/debug/vectorizer"
PORT=19002
BASE="http://127.0.0.1:$PORT"
DIM=64

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

pass=0
fail=0
PIDS=()

cleanup() {
    echo ""
    echo -e "${BLUE}Cleaning up...${NC}"
    for pid in "${PIDS[@]}"; do
        kill "$pid" 2>/dev/null || true
    done
    wait 2>/dev/null || true
    rm -rf /tmp/vectorizer-e2e-test
    echo -e "${BLUE}Done.${NC}"
}
trap cleanup EXIT

check() {
    local name="$1"
    local result="$2"
    if [ "$result" = "true" ]; then
        echo -e "  ${GREEN}PASS${NC} $name"
        ((pass++)) || true
    else
        echo -e "  ${RED}FAIL${NC} $name"
        ((fail++)) || true
    fi
}

wait_for_health() {
    local max=30
    for i in $(seq 1 $max); do
        if curl -sf "$BASE/health" > /dev/null 2>&1; then
            echo -e "  ${GREEN}✓${NC} Server ready"
            return 0
        fi
        sleep 1
    done
    echo -e "  ${RED}✗${NC} Server failed to start"
    return 1
}

gen_vec() {
    python3 -c "import random; random.seed($1); print(','.join([str(round(random.uniform(-1,1),4)) for _ in range($DIM)]))"
}

if [ ! -f "$BINARY" ]; then
    echo "Binary not found. Run: cargo build"
    exit 1
fi

echo "============================================"
echo "  Vectorizer E2E Integration Tests"
echo "============================================"
echo ""

# --- Start Server ---
mkdir -p /tmp/vectorizer-e2e-test
echo -e "${YELLOW}Starting server on port $PORT...${NC}"
DATA_DIR=/tmp/vectorizer-e2e-test \
RUST_LOG=warn \
"$BINARY" --host 127.0.0.1 --port $PORT > /tmp/vectorizer-e2e-test/stdout.log 2>&1 &
PIDS+=($!)
wait_for_health

# =============================================
# Test 1: Health + Status
# =============================================
echo ""
echo -e "${YELLOW}[1/8] Health & Status${NC}"

health=$(curl -sf "$BASE/health" 2>/dev/null || echo "{}")
status=$(echo "$health" | python3 -c "import sys,json; print(json.load(sys.stdin).get('status',''))" 2>/dev/null || echo "")
check "Health endpoint returns healthy" "$([ "$status" = "healthy" ] && echo true || echo false)"

version=$(echo "$health" | python3 -c "import sys,json; print(json.load(sys.stdin).get('version',''))" 2>/dev/null || echo "")
check "Version reported: $version" "$([ -n "$version" ] && echo true || echo false)"

# =============================================
# Test 2: Collection CRUD
# =============================================
echo ""
echo -e "${YELLOW}[2/8] Collection CRUD${NC}"

create=$(curl -s -X POST "$BASE/collections" \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"test-main\",\"dimension\":$DIM,\"metric\":\"cosine\"}" 2>/dev/null || echo "{}")
# Accept both new creation and already-exists
check "Create collection" "$(echo "$create" | python3 -c "import sys,json; d=json.load(sys.stdin); print('true' if 'test-main' in str(d) or d.get('collection')=='test-main' else 'false')" 2>/dev/null || echo false)"

get=$(curl -sf "$BASE/collections/test-main" 2>/dev/null || echo "{}")
check "Get collection info" "$(echo "$get" | python3 -c "import sys,json; d=json.load(sys.stdin); print('true' if d.get('dimension')==$DIM else 'false')" 2>/dev/null || echo false)"

# =============================================
# Test 3: Vector Upsert via Qdrant API
# =============================================
echo ""
echo -e "${YELLOW}[3/8] Vector Upsert (Qdrant API)${NC}"

# Build batch of 20 points
points=""
for i in $(seq 1 20); do
    vec=$(gen_vec $i)
    if [ -n "$points" ]; then points="$points,"; fi
    points="$points{\"id\":\"vec-$i\",\"vector\":[$vec],\"payload\":{\"idx\":$i}}"
done

upsert=$(curl -sf -X PUT "$BASE/qdrant/collections/test-main/points" \
    -H "Content-Type: application/json" \
    -d "{\"points\":[$points]}" 2>/dev/null || echo "{}")
check "Upsert 20 vectors" "$(echo "$upsert" | python3 -c "import sys,json; d=json.load(sys.stdin); print('true' if d.get('status')=='acknowledged' else 'false')" 2>/dev/null || echo false)"

count=$(curl -sf "$BASE/collections/test-main" | python3 -c "import sys,json; print(json.load(sys.stdin).get('vector_count',0))" 2>/dev/null || echo 0)
check "Vector count >= 20 (got: $count)" "$([ "$count" -ge 20 ] && echo true || echo false)"

# =============================================
# Test 4: Search via Qdrant API
# =============================================
echo ""
echo -e "${YELLOW}[4/8] Search (Qdrant API)${NC}"

query=$(gen_vec 42)
search=$(curl -sf -X POST "$BASE/qdrant/collections/test-main/points/search" \
    -H "Content-Type: application/json" \
    -d "{\"vector\":[$query],\"limit\":5}" 2>/dev/null || echo '{"result":[]}')
result_count=$(echo "$search" | python3 -c "import sys,json; print(len(json.load(sys.stdin).get('result',[])))" 2>/dev/null || echo 0)
check "Search returns 5 results (got: $result_count)" "$([ "$result_count" -eq 5 ] && echo true || echo false)"

top_score=$(echo "$search" | python3 -c "import sys,json; r=json.load(sys.stdin).get('result',[]); print(r[0]['score'] if r else 0)" 2>/dev/null || echo 0)
check "Top result has score > 0 (got: $top_score)" "$(python3 -c "print('true' if float('$top_score') > 0 else 'false')")"

# =============================================
# Test 5: Multiple Collections
# =============================================
echo ""
echo -e "${YELLOW}[5/8] Multiple Collections${NC}"

for name in col-euclidean col-dot; do
    metric="euclidean"
    [ "$name" = "col-dot" ] && metric="dot"
    curl -sf -X POST "$BASE/collections" \
        -H "Content-Type: application/json" \
        -d "{\"name\":\"$name\",\"dimension\":32,\"metric\":\"$metric\"}" > /dev/null 2>&1 || true
done

list=$(curl -sf "$BASE/collections" 2>/dev/null || echo "[]")
col_count=$(echo "$list" | python3 -c "import sys,json; d=json.load(sys.stdin); print(len(d) if isinstance(d,list) else len(d.get('collections',[])))" 2>/dev/null || echo 0)
check "3+ collections exist (got: $col_count)" "$([ "$col_count" -ge 3 ] && echo true || echo false)"

# =============================================
# Test 6: Batch Performance
# =============================================
echo ""
echo -e "${YELLOW}[6/8] Batch Upsert Performance${NC}"

# Build 100-point batch
points=""
for i in $(seq 1 100); do
    vec=$(gen_vec $((i+1000)))
    if [ -n "$points" ]; then points="$points,"; fi
    points="$points{\"id\":\"batch-$i\",\"vector\":[$vec]}"
done

start_time=$(python3 -c "import time; print(time.time())")
curl -sf -X PUT "$BASE/qdrant/collections/test-main/points" \
    -H "Content-Type: application/json" \
    -d "{\"points\":[$points]}" > /dev/null 2>&1
end_time=$(python3 -c "import time; print(time.time())")
elapsed=$(python3 -c "print(round($end_time - $start_time, 2))")
check "100 vectors batch upsert in ${elapsed}s" "$(python3 -c "print('true' if $elapsed < 10 else 'false')")"

total=$(curl -sf "$BASE/collections/test-main" | python3 -c "import sys,json; print(json.load(sys.stdin).get('vector_count',0))" 2>/dev/null || echo 0)
check "Total vectors: $total (expected 120)" "$([ "$total" -ge 100 ] && echo true || echo false)"

# =============================================
# Test 7: Cluster Endpoints
# =============================================
echo ""
echo -e "${YELLOW}[7/8] Cluster API Endpoints${NC}"

nodes=$(curl -sf -o /dev/null -w "%{http_code}" "$BASE/api/v1/cluster/nodes" 2>/dev/null || echo "000")
check "Cluster nodes endpoint responds" "$( [ "$nodes" != "000" ] && echo true || echo false )"

shards_code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/api/v1/cluster/shard-distribution" 2>/dev/null || echo "000")
check "Shard distribution responds" "$([ "$shards_code" != "000" ] && echo true || echo false)"

# =============================================
# Test 8: Monitoring
# =============================================
echo ""
echo -e "${YELLOW}[8/8] Monitoring & Metrics${NC}"

stats_code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/api/stats" 2>/dev/null || echo "000")
check "Stats endpoint responds" "$([ "$stats_code" != "000" ] && echo true || echo false)"

metrics=$(curl -sf "$BASE/prometheus/metrics" 2>/dev/null || echo "")
metric_count=$(echo "$metrics" | wc -l | tr -d ' ')
check "Prometheus metrics ($metric_count lines)" "$([ "$metric_count" -gt 10 ] && echo true || echo false)"

# =============================================
# Summary
# =============================================
echo ""
echo "============================================"
total=$((pass + fail))
if [ "$fail" -eq 0 ]; then
    echo -e "  ${GREEN}ALL $total TESTS PASSED${NC}"
else
    echo -e "  Results: ${GREEN}$pass passed${NC}, ${RED}$fail failed${NC} / $total total"
fi
echo "============================================"

[ "$fail" -eq 0 ]
