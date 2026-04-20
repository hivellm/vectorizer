//! REST/MCP parity test, registry-driven.
//!
//! For every entry in `vectorizer_server::server::capabilities::inventory()`
//! tagged `Transport::Both`, this suite validates the **structural**
//! parity contract:
//!
//! - The MCP tool exists in `get_mcp_tools()` with the registered name.
//! - The registry-derived MCP tool's `input_schema` matches the legacy
//!   hand-written one byte-for-byte (these two assertions also live in
//!   the unit tests at `src/server/mcp/tools.rs`; we re-run them at the
//!   integration tier so a CI failure here surfaces them too).
//! - For every `RestOnly` / `McpOnly` entry, the Transport tag is
//!   self-consistent with its `mcp_*` and `rest` fields.
//!
//! What this suite does **not** yet do (tracked as follow-up work):
//!
//! - Boot the full Axum server and issue real HTTP requests for each
//!   `Both` entry, then issue the same call over MCP, then compare
//!   response bodies. That requires server boot + JWT minting + dual
//!   request crafting, which is the next task slot
//!   (`phase4_rest-mcp-parity-runtime-suite`).
//!
//! Until the runtime suite lands, the structural assertions here keep
//! the registry honest and detect schema drift in CI.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashSet;

use vectorizer_server::server::capabilities::{Transport, inventory};
use vectorizer_server::server::mcp_tools::{get_mcp_tools, tools_from_inventory};

#[test]
fn registry_invariants_hold() {
    vectorizer_server::server::capabilities::assert_inventory_invariants()
        .expect("registry invariants must hold");
}

#[test]
fn every_both_entry_is_in_legacy_mcp_tool_list() {
    let legacy_names: HashSet<String> = get_mcp_tools()
        .into_iter()
        .map(|t| t.name.into_owned())
        .collect();

    let mut missing: Vec<&'static str> = Vec::new();
    for cap in inventory() {
        if matches!(cap.transport, Transport::Both | Transport::McpOnly)
            && let Some(name) = cap.mcp_tool_name
            && !legacy_names.contains(name)
        {
            missing.push(name);
        }
    }
    assert!(
        missing.is_empty(),
        "registry MCP tool names with no legacy entry in get_mcp_tools(): {missing:?}"
    );
}

#[test]
fn registry_tool_schemas_match_legacy_byte_for_byte() {
    let legacy: std::collections::HashMap<String, serde_json::Value> = get_mcp_tools()
        .into_iter()
        .map(|t| {
            let name = t.name.into_owned();
            let schema = serde_json::Value::Object((*t.input_schema).clone());
            (name, schema)
        })
        .collect();

    let mut diffs: Vec<String> = Vec::new();
    for tool in tools_from_inventory() {
        let name = tool.name.clone().into_owned();
        let registry_schema = serde_json::Value::Object((*tool.input_schema).clone());
        match legacy.get(&name) {
            None => diffs.push(format!("{name}: missing legacy entry")),
            Some(legacy_schema) if legacy_schema != &registry_schema => {
                diffs.push(format!(
                    "{name}: schema diverged\n  registry: {registry_schema}\n  legacy:   {legacy_schema}"
                ));
            }
            Some(_) => {}
        }
    }
    assert!(
        diffs.is_empty(),
        "registry/legacy MCP schema drift detected:\n{}",
        diffs.join("\n")
    );
}

#[test]
fn rest_only_entries_have_a_rest_route_and_no_mcp_fields() {
    let bad: Vec<&'static str> = inventory()
        .into_iter()
        .filter(|c| c.transport == Transport::RestOnly)
        .filter(|c| c.rest.is_none() || c.mcp_tool_name.is_some() || c.mcp_input_schema.is_some())
        .map(|c| c.id)
        .collect();
    assert!(
        bad.is_empty(),
        "RestOnly entries with malformed transport fields: {bad:?}"
    );
}

#[test]
fn mcp_only_entries_have_mcp_fields_and_no_rest_route() {
    let bad: Vec<&'static str> = inventory()
        .into_iter()
        .filter(|c| c.transport == Transport::McpOnly)
        .filter(|c| c.rest.is_some() || c.mcp_tool_name.is_none() || c.mcp_input_schema.is_none())
        .map(|c| c.id)
        .collect();
    assert!(
        bad.is_empty(),
        "McpOnly entries with malformed transport fields: {bad:?}"
    );
}

#[test]
fn both_entries_carry_full_dual_registration() {
    let bad: Vec<&'static str> = inventory()
        .into_iter()
        .filter(|c| c.transport == Transport::Both)
        .filter(|c| c.rest.is_none() || c.mcp_tool_name.is_none() || c.mcp_input_schema.is_none())
        .map(|c| c.id)
        .collect();
    assert!(
        bad.is_empty(),
        "Both entries missing either an MCP or REST registration: {bad:?}"
    );
}
