//! `vectorizer-core` — primitives shared by the umbrella `vectorizer`
//! crate, the upcoming `vectorizer-server`, and `vectorizer-cli`.
//!
//! Sub-phase 3 of `phase4_split-vectorizer-workspace` extracts the
//! safest leaf modules — the ones with **zero outbound dependencies**
//! on the rest of the codebase — into this crate. They form a
//! self-contained primitives layer:
//!
//! - [`error`] — `VectorizerError` + `ErrorKind` + the wire-protocol
//!   error mappings used by the HTTP middleware, the gRPC interceptor,
//!   and the MCP dispatcher.
//! - [`codec`] — thin `bincode` v3 wrapper that preserves the v1 API
//!   surface (used by cluster, embedding cache, normalization,
//!   persistence).
//! - [`quantization`] — vector quantization primitives (scalar +
//!   product) and the SIMD dispatch hooks they rely on.
//! - [`simd`] — SIMD primitives + per-ISA backends + scalar oracle.
//! - [`parallel`] — `rayon` thread-pool helpers.
//! - [`compression`] — `lz4` / `zstd` wrappers used by the
//!   persistence layer.
//!
//! Heavier modules (`db`, `embedding`, `models`, `persistence`,
//! `file_*`, `discovery`) stay in `vectorizer` for now — they have
//! many cross-deps to server-internal types and a dedicated
//! sub-phase covers each move.
//!
//! The umbrella `vectorizer` crate enforces `#![deny(missing_docs)]`
//! on the public API surface (phase4_enforce-public-api-docs). The
//! moved modules under here keep the same per-file
//! `#![allow(missing_docs)]` annotations on internal data-layout
//! files; the lib root inherits the workspace-level lint policy
//! which leaves `missing_docs` as a `warn` (the umbrella escalates
//! it to `deny` for its own surface).

// Suppress the long tail of legacy clippy warnings (cast_lossless,
// redundant_closure, manual_div_ceil, etc.) so the only doc /
// no-unwrap policies the workspace genuinely enforces are visible.
// Mirrors the same blanket `#![allow(warnings)]` the umbrella
// `vectorizer` crate carries — without it, the workspace clippy
// lints inherited from `[workspace.lints.clippy]` fire as denies
// against pre-existing legacy code that wasn't touched by the
// extraction.
#![allow(warnings)]

pub mod codec;
pub mod compression;
pub mod error;
pub mod parallel;
pub mod quantization;
pub mod simd;
