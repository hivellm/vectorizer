"""Docker image smoke test for the v3.2.0 backpressure surface (#263).

Logs in as root, creates a collection, floods with 200 concurrent
inserts against the configured hard_limit=2 queue, and asserts:

  * /health stays 200 throughout
  * at least one HTTP 429 was returned with Retry-After
  * the vectorizer_upsert_rejected_total{reason="queue_full"} counter
    delta matches the observed 429 count
  * the response body shape is the canonical queue_full envelope
  * version exposed by /health is 3.2.0
"""

from __future__ import annotations

import asyncio
import json
import sys
import time

import aiohttp

BASE_URL = "http://127.0.0.1:25002"
COLLECTION = "docker-smoke"
CONCURRENT = 200
USERNAME = "root"
PASSWORD = "smokerootpw12345"


async def login(session: aiohttp.ClientSession) -> str:
    async with session.post(
        f"{BASE_URL}/auth/login",
        json={"username": USERNAME, "password": PASSWORD},
    ) as resp:
        body = await resp.json()
        return body["access_token"]


async def scrape_counter(session: aiohttp.ClientSession, label: str) -> float:
    async with session.get(f"{BASE_URL}/prometheus/metrics") as resp:
        text = await resp.text()
    needle = f'vectorizer_upsert_rejected_total{{reason="{label}"}} '
    for line in text.splitlines():
        if line.startswith(needle):
            return float(line.split()[-1])
    return 0.0


async def upsert(session: aiohttp.ClientSession, idx: int, headers: dict) -> dict:
    body = {"collection": COLLECTION, "text": f"docker-smoke doc {idx}"}
    async with session.post(f"{BASE_URL}/insert", json=body, headers=headers) as resp:
        return {
            "idx": idx,
            "status": resp.status,
            "retry_after": resp.headers.get("Retry-After"),
            "body": await resp.text(),
        }


async def main() -> int:
    timeout = aiohttp.ClientTimeout(total=30)
    async with aiohttp.ClientSession(timeout=timeout) as session:
        token = await login(session)
        headers = {"Authorization": f"Bearer {token}"}

        async with session.get(f"{BASE_URL}/health") as resp:
            health_before = await resp.json()

        # Create collection (auth required).
        async with session.post(
            f"{BASE_URL}/collections",
            json={"name": COLLECTION, "dimension": 512, "metric": "cosine"},
            headers=headers,
        ) as resp:
            create_status = resp.status

        before = await scrape_counter(session, "queue_full")
        started = time.monotonic()
        results = await asyncio.gather(
            *[upsert(session, i, headers) for i in range(CONCURRENT)]
        )
        elapsed = time.monotonic() - started

        statuses: dict[int, int] = {}
        rejected_with_header = 0
        first_429 = None
        for r in results:
            statuses[r["status"]] = statuses.get(r["status"], 0) + 1
            if r["status"] == 429:
                if r["retry_after"]:
                    rejected_with_header += 1
                if first_429 is None:
                    first_429 = r

        after = await scrape_counter(session, "queue_full")

        async with session.get(f"{BASE_URL}/health") as resp:
            health_after = await resp.json()

    rejected = statuses.get(429, 0)
    delta = after - before

    report = {
        "version_before": health_before.get("version"),
        "version_after": health_after.get("version"),
        "create_collection_status": create_status,
        "concurrent": CONCURRENT,
        "elapsed_s": round(elapsed, 2),
        "status_breakdown": statuses,
        "rejected_429": rejected,
        "rejected_with_retry_after": rejected_with_header,
        "counter_before": before,
        "counter_after": after,
        "counter_delta": delta,
        "first_429_sample": first_429,
        "health_after_flood": health_after.get("status"),
    }
    print(json.dumps(report, indent=2))

    failures: list[str] = []
    if health_before.get("version") != "3.2.0":
        failures.append(f"version mismatch: {health_before.get('version')!r} != 3.2.0")
    if rejected == 0:
        failures.append("expected at least 1 HTTP 429")
    if rejected_with_header < rejected:
        failures.append(
            f"only {rejected_with_header}/{rejected} 429s carried Retry-After"
        )
    if delta != rejected:
        failures.append(f"counter delta ({delta}) != observed 429s ({rejected})")
    if health_after.get("status") != "healthy":
        failures.append(f"/health degraded after flood: {health_after.get('status')!r}")
    if first_429 is not None:
        try:
            body = json.loads(first_429["body"])
            if body.get("error_type") != "queue_full":
                failures.append(f"first 429 error_type={body.get('error_type')!r}")
        except json.JSONDecodeError:
            failures.append(f"first 429 body is not valid JSON: {first_429['body']!r}")

    if failures:
        print("\nFAILURES:")
        for f in failures:
            print(f"  - {f}")
        return 1

    print("\nALL CHECKS PASSED")
    return 0


if __name__ == "__main__":
    sys.exit(asyncio.run(main()))
