# Swap an SDK's transport without touching its typed command surface

**Category**: sdk
**Tags**: rpc, thunder, sdk, migration, wire-protocol

## Description

Keep the SDK's own value type and its constructor/accessor names; introduce ONE total conversion seam to the new library's value model and rewrite only the client. The 2000-4000 line command catalogs then compile untouched.

Per language, the cheapest form of "keep the type" differs:
- Rust: `pub use thunder::Value as VectorizerValue` (a plain alias worked because the accessor names already matched).
- Python: subclass `thunder_rpc.Value` and re-add only the trailing-underscore factories (`str_`, `int_`) the SDK used; accessors are inherited.
- TypeScript: alias the type, then re-implement the `Value` factory and the `asX`/`mapGet` helpers as one-line adapters (Thunder returns `undefined`, this SDK returned `null`).
- Go / C#: keep the SDK's own struct and add `toWire`/`fromWire` (Go) or `ToThunder`/`FromThunder` (C#). Direct field access in the command code (`.Kind`, `MapPair{}`) rules out an alias, and a total conversion is cheaper than rewriting the call sites.

Verify the seam by round-tripping through the new library's REAL frame codec in a test, not just through the conversion functions — that catches encoding drift, not only mapping mistakes.
