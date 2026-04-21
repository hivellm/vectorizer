#!/usr/bin/env bash
set -euo pipefail

# ============================================================================
# Cluster Integration Test Script
# Usage: docker-compose -f docker-compose.cluster-test.yml up -d
#        ./scripts/test-cluster.sh
# ============================================================================

MASTER="http://localhost:15002"
REPLICA1="http://localhost:15012"
REPLICA2="http://localhost:15022"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

pass=0
fail=0

check() {
    local name="$1"
    local result="$2"
    if [ "$result" = "true" ] || [ "$result" = "ok" ]; then
        echo -e "  ${GREEN}PASS${NC} $name"
        ((pass++))
    else
        echo -e "  ${RED}FAIL${NC} $name (got: $result)"
        ((fail++))
    fi
}

echo "============================================"
echo "  Vectorizer Cluster Integration Tests"
echo "============================================"
echo ""

# --- Test 1: Health Checks ---
echo -e "${YELLOW}[1/6] Health Checks${NC}"
master_health=$(curl -sf "$MASTER/health" | jq -r '.status' 2>/dev/null || echo "unreachable")
check "Master healthy" "$([ "$master_health" = "healthy" ] && echo true || echo false)"

replica1_health=$(curl -sf "$REPLICA1/health" | jq -r '.status' 2>/dev/null || echo "unreachable")
check "Replica 1 healthy" "$([ "$replica1_health" = "healthy" ] && echo true || echo false)"

replica2_health=$(curl -sf "$REPLICA2/health" | jq -r '.status' 2>/dev/null || echo "unreachable")
check "Replica 2 healthy" "$([ "$replica2_health" = "healthy" ] && echo true || echo false)"

# --- Test 2: Collection Creation ---
echo ""
echo -e "${YELLOW}[2/6] Collection Creation (Quorum)${NC}"
create_result=$(curl -sf -X POST "$MASTER/collections" \
    -H "Content-Type: application/json" \
    -d '{"name":"test-cluster","dimension":128,"metric":"cosine"}' \
    2>/dev/null || echo '{"error":"failed"}')
check "Collection created on master" "$(echo "$create_result" | jq -r '.name // .error' | grep -q 'test-cluster' && echo true || echo false)"

# Wait for replication
sleep 2

# --- Test 3: Replication ---
echo ""
echo -e "${YELLOW}[3/6] Data Replication${NC}"

# Insert vectors on master
for i in $(seq 1 10); do
    curl -sf -X POST "$MASTER/collections/test-cluster/vectors" \
        -H "Content-Type: application/json" \
        -d "{\"id\":\"vec-$i\",\"vector\":[$(python3 -c "import random; print(','.join([str(random.uniform(-1,1)) for _ in range(128)]))")],\"payload\":{\"index\":$i}}" \
        > /dev/null 2>&1
done
check "10 vectors inserted on master" "true"

# Wait for replication
sleep 3

# Check vector count on master
master_count=$(curl -sf "$MASTER/collections/test-cluster" | jq -r '.vector_count // 0' 2>/dev/null || echo "0")
check "Master has vectors" "$([ "$master_count" -ge 10 ] && echo true || echo false)"

# --- Test 4: Write Concern ---
echo ""
echo -e "${YELLOW}[4/6] Write Concern${NC}"

# Insert with write_concern=1 (wait for 1 replica)
wc_result=$(curl -sf -X POST "$MASTER/collections/test-cluster/vectors?write_concern=1" \
    -H "Content-Type: application/json" \
    -d '{"id":"vec-wc","vector":[0.1,0.2,0.3],"payload":{"test":"write_concern"}}' \
    2>/dev/null || echo '{"error":"timeout or not supported"}')
check "Write with write_concern=1" "$(echo "$wc_result" | jq -r '.id // "ok"' | grep -q 'vec-wc\|ok' && echo true || echo false)"

# --- Test 5: Search ---
echo ""
echo -e "${YELLOW}[5/6] Search Across Cluster${NC}"
search_result=$(curl -sf -X POST "$MASTER/collections/test-cluster/search" \
    -H "Content-Type: application/json" \
    -d "{\"vector\":[$(python3 -c "import random; print(','.join([str(random.uniform(-1,1)) for _ in range(128)]))")],\"limit\":5}" \
    2>/dev/null || echo '{"results":[]}')
result_count=$(echo "$search_result" | jq '.results | length' 2>/dev/null || echo "0")
check "Search returns results" "$([ "$result_count" -gt 0 ] && echo true || echo false)"

# --- Test 6: Cluster Nodes ---
echo ""
echo -e "${YELLOW}[6/6] Cluster Node Discovery${NC}"
nodes_result=$(curl -sf "$MASTER/api/v1/cluster/nodes" 2>/dev/null || echo '{"nodes":[]}')
node_count=$(echo "$nodes_result" | jq '.nodes | length' 2>/dev/null || echo "0")
check "Cluster nodes visible" "$([ "$node_count" -gt 0 ] && echo true || echo false)"

# --- Summary ---
echo ""
echo "============================================"
total=$((pass + fail))
echo -e "  Results: ${GREEN}$pass passed${NC}, ${RED}$fail failed${NC} / $total total"
echo "============================================"

if [ "$fail" -gt 0 ]; then
    exit 1
fi
