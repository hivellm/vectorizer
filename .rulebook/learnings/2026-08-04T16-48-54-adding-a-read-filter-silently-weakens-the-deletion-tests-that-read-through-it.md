# Adding a read filter silently weakens the deletion tests that read through it
**Source**: manual
**Date**: 2026-08-04
**Related Task**: phase1_filter-expired-vectors-on-read
**Tags**: testing, ttl, read-path, false-green
The TTL reaper's tests asserted deletion like this:

    assert!(store.get_vector("reap", "gone").is_err(), "an expired vector must be deleted");

That was sound until the very next task made `get_vector` filter expired vectors. From then on the assertion passed whether or not the sweep deleted anything — the filter alone satisfies it. Five green tests, one of which no longer tested its subject.

The fix is to assert through a path the new filter does not touch. `get_all_vectors` is the raw accessor (it must stay raw so the reaper can find expired vectors and a save does not silently drop one), so the tests now go:

    fn is_stored(store, collection, id) -> bool {
        store.get_collection(collection)
            .map(|c| c.get_all_vectors().iter().any(|v| v.id == id))
            .unwrap_or(false)
    }

And the new filter tests assert the complement — `get_vector` errors while `is_stored` is still true — so each test states which mechanism it is exercising.

Generalisation for this codebase: when adding a filter, guard or short-circuit to a read path, grep the test suite for assertions that go through that path to prove a *write* happened. Deletion, expiry, eviction and invalidation tests are the usual victims, because "not readable" is their proxy for "gone". A filter turns that proxy into a tautology.

Related trap in the same area: `Payload::is_expired` had exactly one caller before this work, so the read paths returned expired data. One caller for a predicate that ought to be consulted on every read is itself a smell worth grepping for.