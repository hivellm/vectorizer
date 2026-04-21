//! `vectorizer-cli` — command-line interface for managing a Vectorizer
//! deployment.
//!
//! Sub-phase 5 of `phase4_split-vectorizer-workspace` extracts the
//! CLI from the umbrella `vectorizer` crate. The two binaries
//! (`vectorizer-cli` for daemon / config / setup commands and
//! `create_mcp_key` for one-shot API-key minting) live under
//! [`bin/`]; the supporting modules (commands, config, setup,
//! utilities) live under [`cli`] and stay re-exportable for callers
//! that still build the CLI surface in-process.

#![allow(warnings)]

pub mod cli;

pub use cli::{Cli, Commands, run};
