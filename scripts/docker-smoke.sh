#!/usr/bin/env bash
# docker-smoke.sh — Manual smoke tests against a running vectorizer
# container on localhost:15002. Exits non-zero on the first red case.
#
# Usage:
#   VECTORIZER_URL=http://127.0.0.1:15002 bash scripts/docker-smoke.sh
#
# Covers every ship-blocker fix landed on release/v3.0.0:
# - F8 file-upload nested metadata + dual-shape fallback reader
# - F1 batch_insert actually persists
# - F2 /search returns real results (not placeholder empty)
# - F3 force-save flushes to disk
# - F4 BM25 fallback produces diverse vectors
# - F5 /embed + /batch_* run real logic
# - F7 auth gate (verified OFF for smoke)
# - Qdrant-compat minimal request shape (probe 3.6)

set -euo pipefail

URL="${VECTORIZER_URL:-http://127.0.0.1:15002}"
fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

echo "=== Vectorizer Docker smoke ($URL) ==="

# 1. Health
status=$(curl -s -o /tmp/health.json -w "%{http_code}" "$URL/health")
[[ "$status" == "200" ]] || fail "health $status"
grep -q '"status":"healthy"' /tmp/health.json || fail "health not healthy"
grep -q '"version":"3.0.0"' /tmp/health.json || fail "health version != 3.0.0"
pass "health healthy + v3.0.0"

# 2. Auth off — /collections anonymous must succeed
status=$(curl -s -o /tmp/cols.json -w "%{http_code}" "$URL/collections")
[[ "$status" == "200" ]] || fail "/collections anonymous $status (auth gate still on?)"
pass "/collections anonymous OK"

# 3. Create collection
COLL="smoke_$(date +%s)"
status=$(curl -s -o /tmp/create.json -w "%{http_code}" -X POST "$URL/collections" \
  -H "Content-Type: application/json" \
  -d "{\"name\":\"$COLL\",\"dimension\":512,\"metric\":\"cosine\"}")
[[ "$status" =~ ^2 ]] || fail "create $COLL $status"
pass "create $COLL"

# 4. Insert single text (F1 — must actually persist)
curl -s -X POST "$URL/insert" \
  -H "Content-Type: application/json" \
  -d "{\"collection\":\"$COLL\",\"text\":\"Vectorizer v3.0.0 smoke test insert\",\"metadata\":{\"src\":\"docker-smoke\"}}" \
  -o /tmp/insert.json -w "%{http_code}\n" | grep -q "^2" \
  || fail "insert"
pass "POST /insert"

# 5. Batch insert (F1 — previously silent-dropped)
cat > /tmp/batch.json <<EOF
{"collection":"$COLL","texts":[
  {"text":"alpha batch smoke doc one","metadata":{"idx":"0"}},
  {"text":"beta batch smoke doc two","metadata":{"idx":"1"}},
  {"text":"gamma batch smoke doc three","metadata":{"idx":"2"}}
]}
EOF
curl -s -X POST "$URL/batch_insert" -H "Content-Type: application/json" \
  -d @/tmp/batch.json -o /tmp/batch_resp.json
grep -q '"inserted":3' /tmp/batch_resp.json || fail "batch_insert inserted != 3 ($(cat /tmp/batch_resp.json))"
pass "POST /batch_insert persisted 3"

# 6. Vector count reflects 4 inserts (1 + 3)
curl -s "$URL/collections/$COLL" -o /tmp/meta.json
grep -q '"vector_count":4' /tmp/meta.json || fail "vector_count != 4 ($(cat /tmp/meta.json | head -c 200))"
pass "collection has 4 vectors"

# 7. Search (F2 — previously returned placeholder empty)
curl -s -X POST "$URL/search" -H "Content-Type: application/json" \
  -d "{\"collection\":\"$COLL\",\"query\":\"batch smoke\",\"limit\":3}" \
  -o /tmp/search.json
grep -q '"query_type"' /tmp/search.json || fail "search missing query_type field (F2 regression)"
python -c "import json; d=json.load(open('/tmp/search.json')); assert len(d.get('results',[]))>0, 'empty results: '+str(d)" \
  || fail "search returned empty"
pass "POST /search returned results with query_type"

# 8. Embed (F5 — previously returned [0.1; 512] placeholder)
curl -s -X POST "$URL/embed" -H "Content-Type: application/json" \
  -d '{"text":"embed smoke test"}' -o /tmp/embed.json
python -c "
import json
d = json.load(open('/tmp/embed.json'))
emb = d.get('embedding', [])
assert len(emb) > 0, 'empty embedding'
# F5 guard: not all 0.1 (placeholder)
assert not all(abs(v - 0.1) < 1e-6 for v in emb), 'F5 regression: placeholder 0.1 vector returned'
assert len(set(round(v, 4) for v in emb)) > 10, 'embedding lacks diversity (F4 regression?)'
print(f'  embedding dim={len(emb)}, unique_rounded={len(set(round(v, 4) for v in emb))}')
" || fail "/embed diversity check"
pass "POST /embed real vector, diverse components"

# 9. force-save (F3 — previously no-op on disk)
curl -s -X POST "$URL/collections/$COLL/force-save" -o /tmp/fs.json
grep -q '"flushed":true' /tmp/fs.json || fail "force-save flushed != true (F3 regression)"
pass "POST /force-save flushed=true"

# 10. Qdrant-compat minimal flat shape (probe 3.6)
QCOLL="qdrant_smoke_$(date +%s)"
status=$(curl -s -o /tmp/qdrant.json -w "%{http_code}" -X PUT "$URL/qdrant/collections/$QCOLL" \
  -H "Content-Type: application/json" \
  -d '{"vectors":{"size":4,"distance":"Cosine"}}')
[[ "$status" =~ ^2 ]] || fail "qdrant flat shape $status (probe 3.6 regression: $(cat /tmp/qdrant.json))"
pass "PUT /qdrant/collections flat {vectors: ...} shape"

# 11. File upload + /file/list + /file/chunks (F8)
F8_COLL="f8_smoke_$(date +%s)"
curl -s -X POST "$URL/collections" -H "Content-Type: application/json" \
  -d "{\"name\":\"$F8_COLL\",\"dimension\":512,\"metric\":\"cosine\"}" -o /dev/null
cat > /tmp/f8_probe.md <<'EOF'
# F8 probe
Docker smoke regression guard for phase8_fix-file-upload-payload-schema.
EOF
curl -s -X POST "$URL/files/upload" \
  -F "collection_name=$F8_COLL" \
  -F "file=@/tmp/f8_probe.md" \
  -o /tmp/upload.json
grep -q '"success":true' /tmp/upload.json || fail "upload (F8): $(cat /tmp/upload.json)"
curl -s -X POST "$URL/file/list" -H "Content-Type: application/json" \
  -d "{\"collection\":\"$F8_COLL\"}" -o /tmp/list.json
python -c "
import json
d = json.load(open('/tmp/list.json'))
files = d.get('files', [])
assert len(files) == 1, f'expected 1 file, got {len(files)}: {d}'
assert files[0].get('path') == 'f8_probe.md', f'unexpected path: {files[0]}'
" || fail "F8 /file/list regression"
pass "POST /files/upload + /file/list (F8 nested-metadata)"

# 12. Cleanup
curl -s -X DELETE "$URL/collections/$COLL" -o /dev/null
curl -s -X DELETE "$URL/collections/$F8_COLL" -o /dev/null
curl -s -X DELETE "$URL/qdrant/collections/$QCOLL" -o /dev/null
pass "cleanup"

echo "=== All smoke tests green ==="
