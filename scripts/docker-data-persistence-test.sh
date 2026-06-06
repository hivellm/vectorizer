#!/usr/bin/env bash
# docker-data-persistence-test.sh — phase32 / issue #300
#
# Pins the regression that motivated phase32: the 3.3.0 image wrote
# persistent state to `/.local/share/vectorizer/` (the XDG path),
# which lived on the container's writable layer. Mounting only
# `/data` looked correct from the README but silently lost every
# collection on `docker compose up -d --force-recreate`.
#
# Test flow:
#   1. Build a local image from the current Dockerfile.
#   2. Start a container with a single `vec-data-test:/data` volume.
#   3. Create a collection + insert vectors via REST.
#   4. Stop + remove the container.
#   5. Start a fresh container from the same volume.
#   6. Assert the collection survives + vector counts match.
#
# Run from repo root:
#   bash scripts/docker-data-persistence-test.sh
#
# Override the image tag / port:
#   IMAGE=local/vectorizer:phase32 PORT=15010 \
#     bash scripts/docker-data-persistence-test.sh
#
# Exits non-zero on the first red case.

set -euo pipefail

IMAGE="${IMAGE:-local/vectorizer:phase32-test}"
PORT="${PORT:-15010}"
CONTAINER="vec-phase32-test"
VOLUME="vec-data-phase32-test"
URL="http://127.0.0.1:${PORT}"

fail() { echo "FAIL: $*" >&2; cleanup; exit 1; }
pass() { echo "PASS: $*"; }

cleanup() {
  docker rm -f "$CONTAINER" >/dev/null 2>&1 || true
}

wait_for_health() {
  local deadline=$((SECONDS + 60))
  while (( SECONDS < deadline )); do
    if curl -fsS "${URL}/health" >/dev/null 2>&1; then
      return 0
    fi
    sleep 2
  done
  return 1
}

trap cleanup EXIT

echo "=== phase32 / #300: container data persistence regression test ==="
echo "    image=$IMAGE container=$CONTAINER volume=$VOLUME url=$URL"

# Step 1: build the image from the current Dockerfile. Use the
# release-docker profile (the same one the CI publishes) so the test
# exercises the actual production layout, not a dev build.
echo
echo "--- Step 1: build image"
docker build -t "$IMAGE" -f Dockerfile . \
  --build-arg PROFILE=release-docker \
  --build-arg NO_DEFAULT_FEATURES=1 \
  --build-arg FEATURES= \
  || fail "docker build failed"
pass "image built"

# Step 2: start container with a single volume on /data.
echo
echo "--- Step 2: first container (writes baseline state)"
cleanup
docker volume create "$VOLUME" >/dev/null
docker run -d --rm \
  --name "$CONTAINER" \
  -p "${PORT}:15002" \
  -v "${VOLUME}:/data" \
  -e VECTORIZER_AUTH_ENABLED=false \
  "$IMAGE" >/dev/null \
  || fail "docker run #1 failed"

if ! wait_for_health; then
  docker logs "$CONTAINER" || true
  fail "/health never responded within 60s on first boot"
fi
pass "first boot — /health green"

# Step 3: create a collection + insert a few texts. We use the
# default embedding provider (bm25 in a no-default-features build),
# so dimension stays at 512 regardless of which providers were
# compiled in.
echo
echo "--- Step 3: write baseline collections"
COLL="phase32_persist_$(date +%s)"

curl -fsS -X POST "${URL}/collections" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"${COLL}\",\"dimension\":512,\"metric\":\"cosine\"}" \
  > /tmp/phase32_create.json \
  || fail "create collection"
pass "collection ${COLL} created"

curl -fsS -X POST "${URL}/batch_insert" \
  -H "Content-Type: application/json" \
  -d "{\"collection\":\"${COLL}\",\"texts\":[
        {\"text\":\"phase32 persistence guard alpha\",\"metadata\":{\"i\":\"0\"}},
        {\"text\":\"phase32 persistence guard beta\",\"metadata\":{\"i\":\"1\"}},
        {\"text\":\"phase32 persistence guard gamma\",\"metadata\":{\"i\":\"2\"}}
      ]}" \
  > /tmp/phase32_insert.json \
  || fail "batch_insert"
grep -q '"inserted":3' /tmp/phase32_insert.json \
  || fail "batch_insert did not report 3 ($(cat /tmp/phase32_insert.json))"
pass "3 vectors inserted"

# Force-save flushes to disk so the next container reload picks up
# the latest WAL position deterministically.
curl -fsS -X POST "${URL}/collections/${COLL}/force-save" -o /tmp/phase32_fs.json \
  || fail "force-save"
grep -q '"flushed":true' /tmp/phase32_fs.json \
  || fail "force-save flushed != true ($(cat /tmp/phase32_fs.json))"
pass "force-save flushed=true"

# Read back the collection metadata so we can compare across the
# recreate boundary.
curl -fsS "${URL}/collections/${COLL}" -o /tmp/phase32_meta_before.json \
  || fail "read meta before recreate"
grep -q '"vector_count":3' /tmp/phase32_meta_before.json \
  || fail "vector_count != 3 before recreate"
pass "metadata shows 3 vectors before recreate"

# Step 4: stop + remove the container.
echo
echo "--- Step 4: docker stop && docker rm (simulates --force-recreate)"
docker stop "$CONTAINER" >/dev/null
pass "container stopped"

# Step 5: recreate from the same volume — this is the exact step
# that wiped collections on 3.3.0.
echo
echo "--- Step 5: second container (must inherit state)"
docker run -d --rm \
  --name "$CONTAINER" \
  -p "${PORT}:15002" \
  -v "${VOLUME}:/data" \
  -e VECTORIZER_AUTH_ENABLED=false \
  "$IMAGE" >/dev/null \
  || fail "docker run #2 failed"

if ! wait_for_health; then
  docker logs "$CONTAINER" || true
  fail "/health never responded within 60s on second boot"
fi
pass "second boot — /health green"

# Step 6: assertions.
echo
echo "--- Step 6: assertions"

# 6a. The collection still exists on the recreated container.
status=$(curl -s -o /tmp/phase32_meta_after.json -w "%{http_code}" \
  "${URL}/collections/${COLL}")
[[ "$status" == "200" ]] || fail "collection lost after recreate (status $status)"
pass "GET /collections/${COLL} returns 200"

# 6b. Vector count survives.
grep -q '"vector_count":3' /tmp/phase32_meta_after.json \
  || fail "vector_count != 3 after recreate (data wiped — phase32 regression!): $(cat /tmp/phase32_meta_after.json | head -c 400)"
pass "vector_count = 3 after recreate"

# 6c. /data inside the running container contains the real
# vectorizer.vecdb — guards against a future regression where the
# binary writes to a different path while the volume mount stays at
# `/data`.
docker exec "$CONTAINER" ls -la /data | tee /tmp/phase32_ls.txt
grep -q "vectorizer.vecdb" /tmp/phase32_ls.txt \
  || fail "/data does not contain vectorizer.vecdb after recreate (resolver pointed elsewhere)"
pass "/data contains vectorizer.vecdb"

# 6d. Search still hits the inserted vectors (end-to-end smoke).
curl -fsS -X POST "${URL}/search" \
  -H "Content-Type: application/json" \
  -d "{\"collection\":\"${COLL}\",\"query\":\"phase32 persistence\",\"limit\":3}" \
  > /tmp/phase32_search.json \
  || fail "search after recreate"
python3 -c "
import json, sys
d = json.load(open('/tmp/phase32_search.json'))
n = len(d.get('results', []))
assert n > 0, 'no search results after recreate: ' + str(d)
print(f'  search hit count: {n}')
" || fail "search returned empty after recreate"
pass "search returns results after recreate"

# Step 7: cleanup the test volume (best effort).
echo
docker volume rm "$VOLUME" >/dev/null 2>&1 || true

echo
echo "=== phase32 (#300) regression test green: collections survive recreate ==="
