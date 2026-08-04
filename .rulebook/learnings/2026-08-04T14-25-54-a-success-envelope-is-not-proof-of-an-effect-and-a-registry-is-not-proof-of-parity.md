# A success envelope is not proof of an effect — and a registry is not proof of parity
**Source**: manual
**Date**: 2026-08-04
**Related Task**: phase3_rpc-full-capability-parity
**Tags**: testing, rpc, parity, durability
Sweeping all 102 RPC commands and finding no errors proved only that dispatch and argument parsing worked. Two classes of bug survived that sweep:

1. collections.force_save answered {success: true} without writing anything. Only a follow-up pass that asserted the *effect* of each mutating command (write, then read it back) would have caught it — and the decisive proof was a container test: write over RPC, force_save, `docker kill` (SIGKILL, so no graceful shutdown force-save), restart on the same volume, read the vectors back.

2. graph.* answered "graph not enabled" for every collection an RPC client could create, because collections.create pinned graph: None and only a REST route could turn a graph on. The command worked; it was unreachable. Comparing the capability registry to the dispatch table did not reveal it either, because the registry marked graph.enable as RestOnly — an intentional-looking tag that hid a hole in the default protocol.

Practical consequences for future test passes here:
- Classify results as OK / feature-disabled / wrong-argument-shape / unexpected. Wrong-argument-shape means the *test* is wrong; conflating it with a server error wastes a cycle.
- Order matters destructively: collections.cleanup_empty takes [Map{dry_run}], and a bare Bool falls through to unwrap_or(false), i.e. a real deletion. It wiped three collections mid-sweep and every later failure looked like a server bug.
- Assertions must account for engine behaviour: a cosine collection stores L2-normalized vectors, so a read never returns the input verbatim; vectors.get returns the payload as a JSON-encoded Str, not a Map.
- Parity claims need a machine check. rpc_command_for + the boot assertion now fail startup when a capability has no RPC command, which is stronger than any sweep.