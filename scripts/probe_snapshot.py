"""Probe 2.1: Snapshot round-trip.

Create collection, insert 100 short texts via POST /insert (the real
chunk-and-embed handler), force-save, restart server externally,
verify collection + vectors survive + search reproducible.

Run phases:
  python scripts/probe_snapshot.py seed      # before server restart
  python scripts/probe_snapshot.py verify    # after server restart
"""

import json
import sys
import urllib.error
import urllib.request

BASE = "http://127.0.0.1:15002"
COLL = "probe_snapshot"
N = 100
SEED_QUERY = "probe document number forty two"


def _req(method, path, body=None):
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(
        f"{BASE}{path}",
        data=data,
        headers={"Content-Type": "application/json"} if data else {},
        method=method,
    )
    try:
        with urllib.request.urlopen(req, timeout=30) as r:
            raw = r.read()
            return r.status, (json.loads(raw) if raw else None)
    except urllib.error.HTTPError as e:
        return e.code, (json.loads(e.read()) if e.fp else None)


def seed():
    _req("DELETE", f"/collections/{COLL}")
    s, b = _req(
        "POST",
        "/collections",
        {"name": COLL, "dimension": 512, "metric": "cosine"},
    )
    assert s in (200, 201, 409), f"create_collection failed: {s} {b}"

    texts = [
        {
            "text": (
                f"probe document number {i:03d}. "
                f"unique phrase alpha-{i:03d} beta-{(i * 37) % 101} gamma-{(i * 13) % 251}. "
                "synthetic seed content for the v3 snapshot round-trip probe."
            ),
            "metadata": {"idx": str(i), "tag": "probe_snapshot"},
        }
        for i in range(N)
    ]
    s, b = _req("POST", "/batch_insert", {"collection": COLL, "texts": texts})
    assert s == 200, f"batch_insert failed: {s} {b}"
    assert b.get("inserted") == N and b.get("failed") == 0, f"batch result: {b}"
    print("batch_insert:", b.get("inserted"), "inserted,", b.get("failed"), "failed")

    s, b = _req("POST", f"/collections/{COLL}/force-save", {})
    print("force_save:", s, b)
    assert b.get("success") and b.get("flushed"), f"force-save did not flush: {b}"

    s, b = _req(
        "POST",
        f"/collections/{COLL}/search/text",
        {"query": SEED_QUERY, "limit": 5},
    )
    assert s == 200, f"search failed: {s} {b}"
    pre = [
        (r.get("id") or r.get("vector_id"), round(r.get("score", 0.0), 6))
        for r in (b.get("results") or [])
    ]
    with open("scripts/probe_snapshot_pre.json", "w") as f:
        json.dump(pre, f, indent=2)
    print("pre-restart top-5:", pre)

    s, b = _req("GET", f"/collections/{COLL}", None)
    print("collection meta pre-restart:", s, b)


def verify():
    s, b = _req("GET", "/collections", None)
    assert s == 200, f"list_collections failed: {s} {b}"
    names = [c.get("name") for c in (b.get("collections") or [])]
    assert COLL in names, f"{COLL} missing after restart: {names}"

    s, b = _req("GET", f"/collections/{COLL}", None)
    print("collection meta post-restart:", s, b)

    s, b = _req(
        "POST",
        f"/collections/{COLL}/search/text",
        {"query": SEED_QUERY, "limit": 5},
    )
    assert s == 200, f"search after restart failed: {s} {b}"
    post = [
        (r.get("id") or r.get("vector_id"), round(r.get("score", 0.0), 6))
        for r in (b.get("results") or [])
    ]
    print("post-restart top-5:", post)

    with open("scripts/probe_snapshot_pre.json") as f:
        pre = [tuple(x) for x in json.load(f)]

    pre_ids = [x[0] for x in pre]
    post_ids = [x[0] for x in post]
    assert pre_ids == post_ids, f"top-5 order diverged: pre={pre_ids} post={post_ids}"
    print("PASS: snapshot round-trip, top-5 id order preserved.")


if __name__ == "__main__":
    mode = sys.argv[1] if len(sys.argv) > 1 else "seed"
    if mode == "seed":
        seed()
    elif mode == "verify":
        verify()
    else:
        raise SystemExit("usage: probe_snapshot.py seed|verify")
