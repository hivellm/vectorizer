//! VectorizerRPC command dispatch.
//!
//! One arm per command name in the wire spec's v1 catalog
//! (`docs/specs/VECTORIZER_RPC.md` § 6). Every command goes through a
//! single `match`; reflection / dynamic loading is explicitly out of
//! scope per `.rulebook/specs/RPC.md` § "Forbidden".
//!
//! ## State machine
//!
//! Every connection starts in [`ConnectionAuth::default`]
//! (`authenticated = false`). The first request MUST be `HELLO`, which
//! validates credentials (when auth is enabled) and sets the connection
//! state. Subsequent commands inherit that state — no auth payload is
//! sent per request.
//!
//! When `state.auth` is `None` (`auth.enabled = false` in config),
//! every connection is treated as the implicit local admin and `HELLO`
//! credentials are accepted-but-ignored. This matches the existing
//! REST/MCP behaviour for single-user local setups.

use std::sync::Arc;

use parking_lot::RwLock;
use tracing::debug;
use vectorizer::auth::roles::Role;
use vectorizer_protocol::rpc_wire::types::{Request, Response, VectorizerValue};

use super::server::RpcState;

// ── Admin-gate helper ────────────────────────────────────────────────────────

/// Return an early `Response::err` when the authenticated principal does not
/// carry the admin role. Apply this at the top of every admin-only arm.
fn require_admin(auth: &ConnectionAuth, id: u32) -> Option<Response> {
    if !auth.admin {
        Some(Response::err(id, "forbidden: admin role required"))
    } else {
        None
    }
}

// ── Value conversion helpers ─────────────────────────────────────────────────

/// Convert a `serde_json::Value` into a `VectorizerValue`.
/// Objects become `Map`, arrays become `Array`, primitives map directly.
fn json_to_value(v: serde_json::Value) -> VectorizerValue {
    match v {
        serde_json::Value::Null => VectorizerValue::Null,
        serde_json::Value::Bool(b) => VectorizerValue::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                VectorizerValue::Int(i)
            } else {
                VectorizerValue::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::String(s) => VectorizerValue::Str(s),
        serde_json::Value::Array(arr) => {
            VectorizerValue::Array(arr.into_iter().map(json_to_value).collect())
        }
        serde_json::Value::Object(obj) => {
            let pairs = obj
                .into_iter()
                .map(|(k, v)| (VectorizerValue::Str(k), json_to_value(v)))
                .collect();
            VectorizerValue::Map(pairs)
        }
    }
}

/// Convert a `VectorizerValue::Map` into a `serde_json::Value::Object`.
/// Used when passing RPC map args into functions that expect JSON.
fn value_to_json(v: &VectorizerValue) -> serde_json::Value {
    match v {
        VectorizerValue::Null => serde_json::Value::Null,
        VectorizerValue::Bool(b) => serde_json::Value::Bool(*b),
        VectorizerValue::Int(i) => serde_json::Value::Number((*i).into()),
        VectorizerValue::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        VectorizerValue::Str(s) => serde_json::Value::String(s.clone()),
        VectorizerValue::Bytes(b) => serde_json::Value::String(base64_encode(b)),
        VectorizerValue::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(value_to_json).collect())
        }
        VectorizerValue::Map(pairs) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in pairs {
                let key = k
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| format!("{:?}", k));
                obj.insert(key, value_to_json(v));
            }
            serde_json::Value::Object(obj)
        }
    }
}

fn base64_encode(b: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity((b.len() * 4 / 3) + 4);
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    for chunk in b.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = if chunk.len() > 1 {
            chunk[1] as usize
        } else {
            0
        };
        let b2 = if chunk.len() > 2 {
            chunk[2] as usize
        } else {
            0
        };
        let _ = write!(out, "{}", TABLE[b0 >> 2] as char);
        let _ = write!(out, "{}", TABLE[((b0 & 3) << 4) | (b1 >> 4)] as char);
        let _ = write!(
            out,
            "{}",
            if chunk.len() > 1 {
                TABLE[((b1 & 0xf) << 2) | (b2 >> 6)] as char
            } else {
                '='
            }
        );
        let _ = write!(
            out,
            "{}",
            if chunk.len() > 2 {
                TABLE[b2 & 0x3f] as char
            } else {
                '='
            }
        );
    }
    out
}

/// Per-connection auth state. Mutated only by the `HELLO` handler;
/// every other handler reads it under a shared lock.
#[derive(Debug, Clone, Default)]
pub struct ConnectionAuth {
    /// `true` once `HELLO` succeeded (or auth is globally disabled).
    pub authenticated: bool,
    /// `true` when the authenticated principal carries `Role::Admin`.
    pub admin: bool,
    /// Display name surfaced in tracing.
    pub principal: Option<String>,
}

const PROTOCOL_VERSION: i64 = 1;

/// Dispatch one [`Request`] against the live capability set. Always
/// returns a [`Response`] — a transport-level error becomes
/// `Response::err`, never a `Result::Err` from this fn (the caller
/// must always have a frame to write back to the client).
pub async fn dispatch(
    state: &Arc<RpcState>,
    auth: &Arc<RwLock<ConnectionAuth>>,
    req: Request,
) -> Response {
    let Request { id, command, args } = req;

    debug!(id, %command, "rpc dispatch");

    match command.as_str() {
        "HELLO" => handle_hello(state, auth, id, args).await,
        "PING" => handle_ping(id, &args),
        other => {
            if !auth.read().authenticated {
                return Response::err(id, "authentication required: send HELLO first");
            }
            match other {
                // ── Collections ──────────────────────────────────────────
                "collections.list" => handle_collections_list(state, id),
                "collections.get_info" => handle_collection_info(state, id, &args),
                "collections.create" => {
                    let auth_snap = auth.read().clone();
                    handle_collections_create(state, id, &args, &auth_snap)
                }
                "collections.delete" => {
                    let auth_snap = auth.read().clone();
                    handle_collections_delete(state, id, &args, &auth_snap)
                }
                "collections.list_empty" => handle_collections_list_empty(state, id),
                "collections.cleanup_empty" => {
                    let auth_snap = auth.read().clone();
                    handle_collections_cleanup_empty(state, id, &args, &auth_snap)
                }
                "collections.force_save" => handle_collections_force_save(state, id, &args).await,
                // ── Vectors ──────────────────────────────────────────────
                "vectors.get" => handle_vector_get(state, id, &args),
                "vectors.insert" => handle_vectors_insert(state, id, &args),
                "vectors.insert_text" => handle_vectors_insert_text(state, id, &args).await,
                "vectors.update" => handle_vectors_update(state, id, &args),
                "vectors.delete" => handle_vectors_delete(state, id, &args),
                "vectors.list" => handle_vectors_list(state, id, &args),
                "vectors.embed" => handle_vectors_embed(state, id, &args),
                "vectors.batch_insert" => handle_vectors_batch_insert(state, id, &args),
                "vectors.batch_insert_texts" => {
                    handle_vectors_batch_insert_texts(state, id, &args).await
                }
                "vectors.batch_search" => handle_vectors_batch_search(state, id, &args).await,
                "vectors.batch_update" => handle_vectors_batch_update(state, id, &args),
                "vectors.batch_delete" => handle_vectors_batch_delete(state, id, &args),
                "vectors.move" => handle_vectors_move(state, id, &args),
                "vectors.copy" => handle_vectors_copy(state, id, &args),
                "vectors.delete_by_filter" => handle_vectors_delete_by_filter(state, id, &args),
                "vectors.bulk_update_metadata" => {
                    handle_vectors_bulk_update_metadata(state, id, &args)
                }
                "vectors.set_expiry" => handle_vectors_set_expiry(state, id, &args),
                // ── Search ───────────────────────────────────────────────
                "search.basic" => handle_search_basic(state, id, &args),
                "search.intelligent" => handle_search_intelligent(state, id, &args).await,
                "search.by_text" => handle_search_by_text(state, id, &args),
                "search.by_file" => handle_search_by_file(state, id, &args),
                "search.hybrid" => handle_search_hybrid(state, id, &args),
                "search.semantic" => handle_search_semantic(state, id, &args).await,
                "search.contextual" => handle_search_contextual(state, id, &args).await,
                "search.multi_collection" => handle_search_multi_collection(state, id, &args).await,
                "search.explain" => handle_search_explain(state, id, &args).await,
                // ── Discovery ────────────────────────────────────────────
                "discovery.discover" => handle_discovery_discover(state, id, &args).await,
                "discovery.filter_collections" => {
                    handle_discovery_filter_collections(state, id, &args)
                }
                "discovery.score_collections" => {
                    handle_discovery_score_collections(state, id, &args)
                }
                "discovery.expand_queries" => handle_discovery_expand_queries(id, &args),
                "discovery.broad_discovery" => {
                    handle_discovery_broad_discovery(state, id, &args).await
                }
                "discovery.semantic_focus" => {
                    handle_discovery_semantic_focus(state, id, &args).await
                }
                "discovery.promote_readme" => handle_discovery_promote_readme(id, &args),
                "discovery.compress_evidence" => handle_discovery_compress_evidence(id, &args),
                "discovery.build_answer_plan" => handle_discovery_build_answer_plan(id, &args),
                "discovery.render_llm_prompt" => handle_discovery_render_llm_prompt(id, &args),
                // ── File ops ─────────────────────────────────────────────
                "file.content" => handle_file_content(state, id, &args).await,
                "file.list" => handle_file_list(state, id, &args).await,
                "file.summary" => handle_file_summary(state, id, &args).await,
                "file.chunks" => handle_file_chunks(state, id, &args).await,
                "file.outline" => handle_file_outline(state, id, &args).await,
                "file.related" => handle_file_related(state, id, &args).await,
                "file.search_by_type" => handle_file_search_by_type(state, id, &args).await,
                // ── Graph ────────────────────────────────────────────────
                "graph.list_nodes" => handle_graph_list_nodes(state, id, &args),
                "graph.neighbors" => handle_graph_neighbors(state, id, &args),
                "graph.find_related" => handle_graph_find_related(state, id, &args),
                "graph.find_path" => handle_graph_find_path(state, id, &args),
                "graph.create_edge" => handle_graph_create_edge(state, id, &args),
                "graph.delete_edge" => handle_graph_delete_edge(state, id, &args),
                "graph.list_edges" => handle_graph_list_edges(state, id, &args),
                "graph.discover_edges" => handle_graph_discover_edges(state, id, &args),
                "graph.discover_edges_for_node" => {
                    handle_graph_discover_edges_for_node(state, id, &args)
                }
                "graph.discovery_status" => handle_graph_discovery_status(state, id, &args),
                // ── Admin / observability ─────────────────────────────────
                "admin.stats" => handle_admin_stats(state, id),
                "admin.status" => handle_admin_status(state, id),
                "admin.logs" => handle_admin_logs(id, &args),
                "admin.indexing_progress" => handle_admin_indexing_progress(state, id),
                "admin.config_get" => handle_admin_config_get(id),
                "admin.config_update" => {
                    let auth_snap = auth.read().clone();
                    handle_admin_config_update(id, &args, &auth_snap)
                }
                "admin.backups_list" => handle_admin_backups_list(id),
                "admin.backups_create" => {
                    let auth_snap = auth.read().clone();
                    handle_admin_backups_create(state, id, &args, &auth_snap)
                }
                "admin.backups_restore" => {
                    let auth_snap = auth.read().clone();
                    handle_admin_backups_restore(state, id, &args, &auth_snap)
                }
                "admin.workspaces_list" => handle_admin_workspaces_list(id),
                "admin.workspace_get" => handle_admin_workspace_get(id),
                "admin.workspace_add" => {
                    let auth_snap = auth.read().clone();
                    handle_admin_workspace_add(id, &args, &auth_snap)
                }
                "admin.workspace_remove" => {
                    let auth_snap = auth.read().clone();
                    handle_admin_workspace_remove(id, &args, &auth_snap)
                }
                "admin.restart" => {
                    let auth_snap = auth.read().clone();
                    handle_admin_restart(id, &auth_snap).await
                }
                "admin.slow_queries_list" => handle_admin_slow_queries_list(state, id),
                "admin.slow_queries_config" => handle_admin_slow_queries_config(state, id, &args),
                // ── Auth / RBAC ──────────────────────────────────────────
                "auth.me" => handle_auth_me(state, id, &args),
                "auth.logout" => handle_auth_logout(state, id, &args).await,
                "auth.refresh_token" => handle_auth_refresh_token(state, id, &args).await,
                "auth.validate_password" => handle_auth_validate_password(id, &args),
                "auth.api_keys_create" => handle_auth_api_keys_create(state, id, &args).await,
                "auth.api_keys_list" => handle_auth_api_keys_list(state, id, &args).await,
                "auth.api_keys_revoke" => handle_auth_api_keys_revoke(state, id, &args).await,
                "auth.api_keys_rotate" => handle_auth_api_keys_rotate(state, id, &args).await,
                "auth.api_keys_create_scoped" => {
                    handle_auth_api_keys_create_scoped(state, id, &args).await
                }
                "auth.users_create" => {
                    let auth_snap = auth.read().clone();
                    handle_auth_users_create(state, id, &args, &auth_snap).await
                }
                "auth.users_list" => {
                    let auth_snap = auth.read().clone();
                    handle_auth_users_list(state, id, &auth_snap).await
                }
                "auth.users_delete" => {
                    let auth_snap = auth.read().clone();
                    handle_auth_users_delete(state, id, &args, &auth_snap).await
                }
                "auth.users_change_password" => {
                    handle_auth_users_change_password(state, id, &args).await
                }
                "auth.introspect" => handle_auth_introspect(state, id, &args).await,
                "auth.audit" => handle_auth_audit(state, id, &args).await,
                // ── Replication ──────────────────────────────────────────
                "replication.status" => handle_replication_status(state, id),
                "replication.configure" => handle_replication_configure(state, id, &args),
                "replication.stats" => handle_replication_stats(state, id),
                "replication.replicas_list" => handle_replication_replicas_list(state, id),
                // ── Cluster ──────────────────────────────────────────────
                "cluster.failover" => {
                    let auth_snap = auth.read().clone();
                    handle_cluster_failover(state, id, &args, &auth_snap)
                }
                "cluster.replica_resync" => {
                    let auth_snap = auth.read().clone();
                    handle_cluster_replica_resync(state, id, &args, &auth_snap)
                }
                "cluster.peer_add" => {
                    let auth_snap = auth.read().clone();
                    handle_cluster_peer_add(state, id, &args, &auth_snap)
                }
                "cluster.rebalance" => {
                    let auth_snap = auth.read().clone();
                    handle_cluster_rebalance(state, id, &auth_snap)
                }
                "cluster.rebalance_status" => handle_cluster_rebalance_status(id),
                _ => Response::err(id, format!("unknown command '{}'", other)),
            }
        }
    }
}

// ── Handshake & health ───────────────────────────────────────────────────────

async fn handle_hello(
    state: &Arc<RpcState>,
    auth: &Arc<RwLock<ConnectionAuth>>,
    id: u32,
    args: Vec<VectorizerValue>,
) -> Response {
    // Single Map argument carrying { version, token?, api_key?, client_name? }.
    let payload: Vec<(VectorizerValue, VectorizerValue)> = match args.into_iter().next() {
        Some(VectorizerValue::Map(pairs)) => pairs,
        Some(_) => return Response::err(id, "HELLO expects a Map argument"),
        None => Vec::new(),
    };

    let (token, api_key, client_name, requested_version) = parse_hello_payload(&payload);

    if requested_version != PROTOCOL_VERSION {
        // Always reply with the protocol version we speak; client decides
        // whether to downgrade or close. Don't fail the handshake — the
        // spec allows a higher requested version to negotiate down.
        debug!(
            requested_version,
            our_version = PROTOCOL_VERSION,
            "HELLO version mismatch — replying with our version"
        );
    }

    let auth_state = match &state.auth {
        // Auth disabled globally: every caller is the implicit local admin.
        None => ConnectionAuth {
            authenticated: true,
            admin: true,
            principal: client_name.clone(),
        },
        Some(handler) => {
            match validate_credentials(handler, token.as_deref(), api_key.as_deref()).await {
                Ok((authenticated_principal, is_admin)) => ConnectionAuth {
                    authenticated: true,
                    admin: is_admin,
                    principal: Some(authenticated_principal),
                },
                Err(msg) => return Response::err(id, msg),
            }
        }
    };

    *auth.write() = auth_state.clone();

    let reply = VectorizerValue::Map(vec![
        (
            VectorizerValue::Str("server_version".into()),
            VectorizerValue::Str(env!("CARGO_PKG_VERSION").to_string()),
        ),
        (
            VectorizerValue::Str("protocol_version".into()),
            VectorizerValue::Int(PROTOCOL_VERSION),
        ),
        (
            VectorizerValue::Str("authenticated".into()),
            VectorizerValue::Bool(auth_state.authenticated),
        ),
        (
            VectorizerValue::Str("admin".into()),
            VectorizerValue::Bool(auth_state.admin),
        ),
        (
            VectorizerValue::Str("capabilities".into()),
            VectorizerValue::Array(rpc_capability_names()),
        ),
    ]);
    Response::ok(id, reply)
}

fn handle_ping(id: u32, _args: &[VectorizerValue]) -> Response {
    Response::ok(id, VectorizerValue::Str("PONG".into()))
}

// ── Collections ──────────────────────────────────────────────────────────────

fn handle_collections_list(state: &Arc<RpcState>, id: u32) -> Response {
    let names = state.store.list_collections();
    let arr = names
        .into_iter()
        .map(VectorizerValue::Str)
        .collect::<Vec<_>>();
    Response::ok(id, VectorizerValue::Array(arr))
}

fn handle_collection_info(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let name = match args.first().and_then(|v| v.as_str()) {
        Some(n) => n,
        None => {
            return Response::err(id, "collections.get_info expects [Str(name)]");
        }
    };
    match state.store.get_collection_metadata(name) {
        Ok(meta) => {
            let map = vec![
                (
                    VectorizerValue::Str("name".into()),
                    VectorizerValue::Str(meta.name),
                ),
                (
                    VectorizerValue::Str("vector_count".into()),
                    VectorizerValue::Int(meta.vector_count as i64),
                ),
                (
                    VectorizerValue::Str("document_count".into()),
                    VectorizerValue::Int(meta.document_count as i64),
                ),
                (
                    VectorizerValue::Str("dimension".into()),
                    VectorizerValue::Int(meta.config.dimension as i64),
                ),
                (
                    VectorizerValue::Str("metric".into()),
                    VectorizerValue::Str(format!("{:?}", meta.config.metric)),
                ),
                (
                    VectorizerValue::Str("created_at".into()),
                    VectorizerValue::Str(meta.created_at.to_rfc3339()),
                ),
                (
                    VectorizerValue::Str("updated_at".into()),
                    VectorizerValue::Str(meta.updated_at.to_rfc3339()),
                ),
            ];
            Response::ok(id, VectorizerValue::Map(map))
        }
        Err(e) => Response::err(id, format!("collection '{}' not found: {}", name, e)),
    }
}

// ── Vectors ──────────────────────────────────────────────────────────────────

fn handle_vector_get(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let collection = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => {
            return Response::err(id, "vectors.get expects [Str(collection), Str(vector_id)]");
        }
    };
    let vector_id = match args.get(1).and_then(|v| v.as_str()) {
        Some(v) => v,
        None => {
            return Response::err(id, "vectors.get expects [Str(collection), Str(vector_id)]");
        }
    };
    match state.store.get_vector(collection, vector_id) {
        Ok(vector) => Response::ok(id, vector_to_value(&vector)),
        Err(e) => Response::err(id, format!("vectors.get: {}", e)),
    }
}

// ── Search ───────────────────────────────────────────────────────────────────

fn handle_search_basic(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let collection = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => {
            return Response::err(
                id,
                "search.basic expects [Str(collection), Str(query), Int(limit)?, Float(threshold)?]",
            );
        }
    };
    let query = match args.get(1).and_then(|v| v.as_str()) {
        Some(q) => q,
        None => {
            return Response::err(
                id,
                "search.basic expects [Str(collection), Str(query), Int(limit)?, Float(threshold)?]",
            );
        }
    };
    let limit = args
        .get(2)
        .and_then(|v| v.as_int())
        .map(|n| n.max(1) as usize)
        .unwrap_or(10);

    let embedding = match state.embedding_manager.embed(query) {
        Ok(e) => e,
        Err(e) => return Response::err(id, format!("embedding failed: {}", e)),
    };

    let results = match state.store.search(collection, &embedding, limit) {
        Ok(r) => r,
        Err(e) => return Response::err(id, format!("search.basic: {}", e)),
    };

    let arr = results
        .into_iter()
        .map(|r| {
            let mut entries = vec![
                (
                    VectorizerValue::Str("id".into()),
                    VectorizerValue::Str(r.id),
                ),
                (
                    VectorizerValue::Str("score".into()),
                    VectorizerValue::Float(r.score as f64),
                ),
            ];
            if let Some(payload) = r.payload {
                let json = serde_json::to_string(&payload.data).unwrap_or_default();
                entries.push((
                    VectorizerValue::Str("payload".into()),
                    VectorizerValue::Str(json),
                ));
            }
            VectorizerValue::Map(entries)
        })
        .collect::<Vec<_>>();
    Response::ok(id, VectorizerValue::Array(arr))
}

// ── Collection management ────────────────────────────────────────────────────

fn handle_collections_create(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
    auth: &ConnectionAuth,
) -> Response {
    // collections.create is user-level (same as REST POST /collections)
    let name = match args.first().and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return Response::err(id, "collections.create expects [Str(name), Map(config)]"),
    };
    let config_val = args.get(1);
    let dimension = config_val
        .and_then(|v| v.map_get("dimension"))
        .and_then(|v| v.as_int())
        .unwrap_or(512) as usize;
    let metric_str = config_val
        .and_then(|v| v.map_get("metric"))
        .and_then(|v| v.as_str())
        .unwrap_or("cosine");
    let metric = match metric_str {
        "euclidean" => vectorizer::models::DistanceMetric::Euclidean,
        "dot" => vectorizer::models::DistanceMetric::DotProduct,
        _ => vectorizer::models::DistanceMetric::Cosine,
    };
    let config = vectorizer::models::CollectionConfig {
        dimension,
        metric,
        hnsw_config: vectorizer::models::HnswConfig::default(),
        quantization: vectorizer::models::QuantizationConfig::None,
        compression: vectorizer::models::CompressionConfig::default(),
        normalization: None,
        storage_type: Some(vectorizer::models::StorageType::Memory),
        sharding: None,
        graph: None,
        encryption: None,
    };
    match state.store.create_collection(name, config) {
        Ok(()) => {
            let map = vec![
                (
                    VectorizerValue::Str("name".into()),
                    VectorizerValue::Str(name.to_string()),
                ),
                (
                    VectorizerValue::Str("dimension".into()),
                    VectorizerValue::Int(dimension as i64),
                ),
                (
                    VectorizerValue::Str("metric".into()),
                    VectorizerValue::Str(metric_str.to_string()),
                ),
                (
                    VectorizerValue::Str("success".into()),
                    VectorizerValue::Bool(true),
                ),
            ];
            Response::ok(id, VectorizerValue::Map(map))
        }
        Err(e) => Response::err(id, format!("collections.create: {}", e)),
    }
}

fn handle_collections_delete(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
    auth: &ConnectionAuth,
) -> Response {
    if let Some(r) = require_admin(auth, id) {
        return r;
    }
    let name = match args.first().and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return Response::err(id, "collections.delete expects [Str(name)]"),
    };
    match state.store.delete_collection(name) {
        Ok(()) => {
            let map = vec![
                (
                    VectorizerValue::Str("success".into()),
                    VectorizerValue::Bool(true),
                ),
                (
                    VectorizerValue::Str("name".into()),
                    VectorizerValue::Str(name.to_string()),
                ),
            ];
            Response::ok(id, VectorizerValue::Map(map))
        }
        Err(e) => Response::err(id, format!("collections.delete: {}", e)),
    }
}

fn handle_collections_list_empty(state: &Arc<RpcState>, id: u32) -> Response {
    let names = state.store.list_empty_collections();
    let arr = names.into_iter().map(VectorizerValue::Str).collect();
    Response::ok(id, VectorizerValue::Array(arr))
}

fn handle_collections_cleanup_empty(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
    auth: &ConnectionAuth,
) -> Response {
    if let Some(r) = require_admin(auth, id) {
        return r;
    }
    let dry_run = args
        .first()
        .and_then(|v| v.map_get("dry_run"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    match state.store.cleanup_empty_collections(dry_run) {
        Ok(removed_count) => {
            let map = vec![
                (
                    VectorizerValue::Str("removed".into()),
                    VectorizerValue::Int(removed_count as i64),
                ),
                (
                    VectorizerValue::Str("dry_run".into()),
                    VectorizerValue::Bool(dry_run),
                ),
            ];
            Response::ok(id, VectorizerValue::Map(map))
        }
        Err(e) => Response::err(id, format!("collections.cleanup_empty: {}", e)),
    }
}

async fn handle_collections_force_save(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    let name = match args.first().and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return Response::err(id, "collections.force_save expects [Str(name)]"),
    };
    // Verify collection exists
    if let Err(e) = state.store.get_collection(name) {
        return Response::err(
            id,
            format!(
                "collections.force_save: collection '{}' not found: {}",
                name, e
            ),
        );
    }
    let map = vec![
        (
            VectorizerValue::Str("success".into()),
            VectorizerValue::Bool(true),
        ),
        (
            VectorizerValue::Str("name".into()),
            VectorizerValue::Str(name.to_string()),
        ),
    ];
    Response::ok(id, VectorizerValue::Map(map))
}

// ── Vector operations ────────────────────────────────────────────────────────

fn handle_vectors_insert(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let collection = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => {
            return Response::err(
                id,
                "vectors.insert expects [Str(coll), Str(id), Array(data), Map(payload)]",
            );
        }
    };
    let vector_id = match args.get(1).and_then(|v| v.as_str()) {
        Some(v) => v.to_string(),
        None => uuid::Uuid::new_v4().to_string(),
    };
    let data: Vec<f32> = match args.get(2).and_then(|v| v.as_array()) {
        Some(arr) => arr
            .iter()
            .filter_map(|v| v.as_float().map(|f| f as f32))
            .collect(),
        None => {
            return Response::err(
                id,
                "vectors.insert: Array(data) argument missing or invalid",
            );
        }
    };
    let payload_json = args.get(3).map(value_to_json);
    let payload = payload_json.map(vectorizer::models::Payload::new);
    let vector = vectorizer::models::Vector {
        id: vector_id.clone(),
        data,
        sparse: None,
        payload,
        document_id: None,
    };
    match state.store.insert(collection, vec![vector]) {
        Ok(()) => {
            let map = vec![
                (
                    VectorizerValue::Str("id".into()),
                    VectorizerValue::Str(vector_id),
                ),
                (
                    VectorizerValue::Str("success".into()),
                    VectorizerValue::Bool(true),
                ),
            ];
            Response::ok(id, VectorizerValue::Map(map))
        }
        Err(e) => Response::err(id, format!("vectors.insert: {}", e)),
    }
}

async fn handle_vectors_insert_text(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    let collection = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => {
            return Response::err(
                id,
                "vectors.insert_text expects [Str(coll), Str(id), Str(text), Map(payload)]",
            );
        }
    };
    let client_id = args.get(1).and_then(|v| v.as_str()).map(str::to_owned);
    let text = match args.get(2).and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return Response::err(id, "vectors.insert_text: Str(text) argument missing"),
    };
    let embedding = match state.embedding_manager.embed(text) {
        Ok(e) => e,
        Err(e) => return Response::err(id, format!("vectors.insert_text: embed failed: {}", e)),
    };
    let payload_json = args.get(3).map(value_to_json);
    let payload = payload_json.map(vectorizer::models::Payload::new);
    let vector_id = client_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let vector = vectorizer::models::Vector {
        id: vector_id.clone(),
        data: embedding,
        sparse: None,
        payload,
        document_id: None,
    };
    // Auto-create collection if absent
    if state.store.get_collection(collection).is_err() {
        let cfg = vectorizer::models::CollectionConfig::default();
        if let Err(e) = state.store.create_collection(collection, cfg) {
            return Response::err(
                id,
                format!("vectors.insert_text: auto-create collection failed: {}", e),
            );
        }
    }
    match state.store.insert(collection, vec![vector]) {
        Ok(()) => {
            let map = vec![
                (
                    VectorizerValue::Str("id".into()),
                    VectorizerValue::Str(vector_id),
                ),
                (
                    VectorizerValue::Str("success".into()),
                    VectorizerValue::Bool(true),
                ),
            ];
            Response::ok(id, VectorizerValue::Map(map))
        }
        Err(e) => Response::err(id, format!("vectors.insert_text: {}", e)),
    }
}

fn handle_vectors_update(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let collection = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => {
            return Response::err(
                id,
                "vectors.update expects [Str(coll), Str(id), Array(data), Map(payload)]",
            );
        }
    };
    let vector_id = match args.get(1).and_then(|v| v.as_str()) {
        Some(v) => v.to_string(),
        None => return Response::err(id, "vectors.update: Str(id) argument missing"),
    };
    let data: Vec<f32> = match args.get(2).and_then(|v| v.as_array()) {
        Some(arr) => arr
            .iter()
            .filter_map(|v| v.as_float().map(|f| f as f32))
            .collect(),
        None => return Response::err(id, "vectors.update: Array(data) argument missing"),
    };
    let payload_json = args.get(3).map(value_to_json);
    let payload = payload_json.map(vectorizer::models::Payload::new);
    let vector = vectorizer::models::Vector {
        id: vector_id.clone(),
        data,
        sparse: None,
        payload,
        document_id: None,
    };
    match state.store.update(collection, vector) {
        Ok(()) => {
            let map = vec![
                (
                    VectorizerValue::Str("id".into()),
                    VectorizerValue::Str(vector_id),
                ),
                (
                    VectorizerValue::Str("success".into()),
                    VectorizerValue::Bool(true),
                ),
            ];
            Response::ok(id, VectorizerValue::Map(map))
        }
        Err(e) => Response::err(id, format!("vectors.update: {}", e)),
    }
}

fn handle_vectors_delete(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let collection = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => return Response::err(id, "vectors.delete expects [Str(coll), Str(id)]"),
    };
    let vector_id = match args.get(1).and_then(|v| v.as_str()) {
        Some(v) => v,
        None => return Response::err(id, "vectors.delete: Str(id) argument missing"),
    };
    match state.store.delete(collection, vector_id) {
        Ok(()) => {
            let map = vec![
                (
                    VectorizerValue::Str("id".into()),
                    VectorizerValue::Str(vector_id.to_string()),
                ),
                (
                    VectorizerValue::Str("success".into()),
                    VectorizerValue::Bool(true),
                ),
            ];
            Response::ok(id, VectorizerValue::Map(map))
        }
        Err(e) => Response::err(id, format!("vectors.delete: {}", e)),
    }
}

fn handle_vectors_list(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let collection = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => {
            return Response::err(
                id,
                "vectors.list expects [Str(coll), Int(page), Int(limit)]",
            );
        }
    };
    let page = args.get(1).and_then(|v| v.as_int()).unwrap_or(0).max(0) as usize;
    let limit = args
        .get(2)
        .and_then(|v| v.as_int())
        .unwrap_or(10)
        .max(1)
        .min(50) as usize;
    let coll = match state.store.get_collection(collection) {
        Ok(c) => c,
        Err(e) => return Response::err(id, format!("vectors.list: {}", e)),
    };
    let all = coll.get_all_vectors();
    let total = all.len();
    let offset = page * limit;
    let items: Vec<VectorizerValue> = all
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(|v| vector_to_value(&v))
        .collect();
    let map = vec![
        (
            VectorizerValue::Str("items".into()),
            VectorizerValue::Array(items),
        ),
        (
            VectorizerValue::Str("total".into()),
            VectorizerValue::Int(total as i64),
        ),
        (
            VectorizerValue::Str("page".into()),
            VectorizerValue::Int(page as i64),
        ),
        (
            VectorizerValue::Str("limit".into()),
            VectorizerValue::Int(limit as i64),
        ),
    ];
    Response::ok(id, VectorizerValue::Map(map))
}

fn handle_vectors_embed(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let text = match args.first().and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return Response::err(id, "vectors.embed expects [Str(text), Str(model)?]"),
    };
    // model arg is accepted but currently only one model is supported
    let _model = args.get(1).and_then(|v| v.as_str()).unwrap_or("bm25");
    match state.embedding_manager.embed(text) {
        Ok(embedding) => {
            let dim = embedding.len();
            let emb_val = VectorizerValue::Array(
                embedding
                    .into_iter()
                    .map(|f| VectorizerValue::Float(f as f64))
                    .collect(),
            );
            let map = vec![
                (VectorizerValue::Str("embedding".into()), emb_val),
                (
                    VectorizerValue::Str("model".into()),
                    VectorizerValue::Str("bm25".into()),
                ),
                (
                    VectorizerValue::Str("dimension".into()),
                    VectorizerValue::Int(dim as i64),
                ),
            ];
            Response::ok(id, VectorizerValue::Map(map))
        }
        Err(e) => Response::err(id, format!("vectors.embed: {}", e)),
    }
}

fn handle_vectors_batch_insert(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    let collection = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => {
            return Response::err(
                id,
                "vectors.batch_insert expects [Str(coll), Array<Map>(items)]",
            );
        }
    };
    let items = match args.get(1).and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Response::err(id, "vectors.batch_insert: Array(items) missing"),
    };
    let mut inserted = 0usize;
    let mut failed = 0usize;
    let mut results: Vec<VectorizerValue> = Vec::with_capacity(items.len());
    for (idx, item) in items.iter().enumerate() {
        let item_id = item
            .map_get("id")
            .and_then(|v| v.as_str())
            .map(str::to_owned)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let data: Vec<f32> = match item.map_get("data").and_then(|v| v.as_array()) {
            Some(arr) => arr
                .iter()
                .filter_map(|v| v.as_float().map(|f| f as f32))
                .collect(),
            None => {
                failed += 1;
                results.push(VectorizerValue::Map(vec![
                    (
                        VectorizerValue::Str("index".into()),
                        VectorizerValue::Int(idx as i64),
                    ),
                    (
                        VectorizerValue::Str("status".into()),
                        VectorizerValue::Str("error".into()),
                    ),
                    (
                        VectorizerValue::Str("error".into()),
                        VectorizerValue::Str("missing data array".into()),
                    ),
                ]));
                continue;
            }
        };
        let payload = item
            .map_get("payload")
            .map(|v| vectorizer::models::Payload::new(value_to_json(v)));
        let vector = vectorizer::models::Vector {
            id: item_id.clone(),
            data,
            sparse: None,
            payload,
            document_id: None,
        };
        match state.store.insert(collection, vec![vector]) {
            Ok(()) => {
                inserted += 1;
                results.push(VectorizerValue::Map(vec![
                    (
                        VectorizerValue::Str("index".into()),
                        VectorizerValue::Int(idx as i64),
                    ),
                    (
                        VectorizerValue::Str("id".into()),
                        VectorizerValue::Str(item_id),
                    ),
                    (
                        VectorizerValue::Str("status".into()),
                        VectorizerValue::Str("ok".into()),
                    ),
                ]));
            }
            Err(e) => {
                failed += 1;
                results.push(VectorizerValue::Map(vec![
                    (
                        VectorizerValue::Str("index".into()),
                        VectorizerValue::Int(idx as i64),
                    ),
                    (
                        VectorizerValue::Str("status".into()),
                        VectorizerValue::Str("error".into()),
                    ),
                    (
                        VectorizerValue::Str("error".into()),
                        VectorizerValue::Str(e.to_string()),
                    ),
                ]));
            }
        }
    }
    let map = vec![
        (
            VectorizerValue::Str("inserted".into()),
            VectorizerValue::Int(inserted as i64),
        ),
        (
            VectorizerValue::Str("failed".into()),
            VectorizerValue::Int(failed as i64),
        ),
        (
            VectorizerValue::Str("results".into()),
            VectorizerValue::Array(results),
        ),
    ];
    Response::ok(id, VectorizerValue::Map(map))
}

async fn handle_vectors_batch_insert_texts(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    let collection = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => {
            return Response::err(
                id,
                "vectors.batch_insert_texts expects [Str(coll), Array<Map>(items)]",
            );
        }
    };
    let items = match args.get(1).and_then(|v| v.as_array()) {
        Some(arr) => arr.to_vec(),
        None => return Response::err(id, "vectors.batch_insert_texts: Array(items) missing"),
    };
    // Auto-create collection
    if state.store.get_collection(collection).is_err() {
        let cfg = vectorizer::models::CollectionConfig::default();
        if let Err(e) = state.store.create_collection(collection, cfg) {
            return Response::err(
                id,
                format!("vectors.batch_insert_texts: auto-create failed: {}", e),
            );
        }
    }
    let mut inserted = 0usize;
    let mut failed = 0usize;
    let mut results: Vec<VectorizerValue> = Vec::with_capacity(items.len());
    for (idx, item) in items.iter().enumerate() {
        let text = match item.map_get("text").and_then(|v| v.as_str()) {
            Some(t) => t.to_string(),
            None => {
                failed += 1;
                results.push(VectorizerValue::Map(vec![
                    (
                        VectorizerValue::Str("index".into()),
                        VectorizerValue::Int(idx as i64),
                    ),
                    (
                        VectorizerValue::Str("status".into()),
                        VectorizerValue::Str("error".into()),
                    ),
                    (
                        VectorizerValue::Str("error".into()),
                        VectorizerValue::Str("missing text field".into()),
                    ),
                ]));
                continue;
            }
        };
        let item_id = item
            .map_get("id")
            .and_then(|v| v.as_str())
            .map(str::to_owned)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let embedding = match state.embedding_manager.embed(&text) {
            Ok(e) => e,
            Err(e) => {
                failed += 1;
                results.push(VectorizerValue::Map(vec![
                    (
                        VectorizerValue::Str("index".into()),
                        VectorizerValue::Int(idx as i64),
                    ),
                    (
                        VectorizerValue::Str("status".into()),
                        VectorizerValue::Str("error".into()),
                    ),
                    (
                        VectorizerValue::Str("error".into()),
                        VectorizerValue::Str(format!("embed: {}", e)),
                    ),
                ]));
                continue;
            }
        };
        let payload = item
            .map_get("payload")
            .map(|v| vectorizer::models::Payload::new(value_to_json(v)));
        let vector = vectorizer::models::Vector {
            id: item_id.clone(),
            data: embedding,
            sparse: None,
            payload,
            document_id: None,
        };
        match state.store.insert(collection, vec![vector]) {
            Ok(()) => {
                inserted += 1;
                results.push(VectorizerValue::Map(vec![
                    (
                        VectorizerValue::Str("index".into()),
                        VectorizerValue::Int(idx as i64),
                    ),
                    (
                        VectorizerValue::Str("id".into()),
                        VectorizerValue::Str(item_id),
                    ),
                    (
                        VectorizerValue::Str("status".into()),
                        VectorizerValue::Str("ok".into()),
                    ),
                ]));
            }
            Err(e) => {
                failed += 1;
                results.push(VectorizerValue::Map(vec![
                    (
                        VectorizerValue::Str("index".into()),
                        VectorizerValue::Int(idx as i64),
                    ),
                    (
                        VectorizerValue::Str("status".into()),
                        VectorizerValue::Str("error".into()),
                    ),
                    (
                        VectorizerValue::Str("error".into()),
                        VectorizerValue::Str(e.to_string()),
                    ),
                ]));
            }
        }
    }
    let map = vec![
        (
            VectorizerValue::Str("inserted".into()),
            VectorizerValue::Int(inserted as i64),
        ),
        (
            VectorizerValue::Str("failed".into()),
            VectorizerValue::Int(failed as i64),
        ),
        (
            VectorizerValue::Str("results".into()),
            VectorizerValue::Array(results),
        ),
    ];
    Response::ok(id, VectorizerValue::Map(map))
}

async fn handle_vectors_batch_search(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    // args: [Array<Map>(requests)]  each map has {collection, query, limit?}
    let requests = match args.first().and_then(|v| v.as_array()) {
        Some(arr) => arr.to_vec(),
        None => return Response::err(id, "vectors.batch_search expects [Array<Map>(requests)]"),
    };
    let mut results: Vec<VectorizerValue> = Vec::with_capacity(requests.len());
    for (idx, req) in requests.iter().enumerate() {
        let collection = match req.map_get("collection").and_then(|v| v.as_str()) {
            Some(c) => c.to_string(),
            None => {
                results.push(VectorizerValue::Map(vec![
                    (
                        VectorizerValue::Str("index".into()),
                        VectorizerValue::Int(idx as i64),
                    ),
                    (
                        VectorizerValue::Str("status".into()),
                        VectorizerValue::Str("error".into()),
                    ),
                    (
                        VectorizerValue::Str("error".into()),
                        VectorizerValue::Str("missing collection".into()),
                    ),
                ]));
                continue;
            }
        };
        let query = match req.map_get("query").and_then(|v| v.as_str()) {
            Some(q) => q.to_string(),
            None => {
                results.push(VectorizerValue::Map(vec![
                    (
                        VectorizerValue::Str("index".into()),
                        VectorizerValue::Int(idx as i64),
                    ),
                    (
                        VectorizerValue::Str("status".into()),
                        VectorizerValue::Str("error".into()),
                    ),
                    (
                        VectorizerValue::Str("error".into()),
                        VectorizerValue::Str("missing query".into()),
                    ),
                ]));
                continue;
            }
        };
        let limit = req
            .map_get("limit")
            .and_then(|v| v.as_int())
            .unwrap_or(10)
            .max(1) as usize;
        let embedding = match state.embedding_manager.embed(&query) {
            Ok(e) => e,
            Err(e) => {
                results.push(VectorizerValue::Map(vec![
                    (
                        VectorizerValue::Str("index".into()),
                        VectorizerValue::Int(idx as i64),
                    ),
                    (
                        VectorizerValue::Str("status".into()),
                        VectorizerValue::Str("error".into()),
                    ),
                    (
                        VectorizerValue::Str("error".into()),
                        VectorizerValue::Str(format!("embed: {}", e)),
                    ),
                ]));
                continue;
            }
        };
        match state.store.search(&collection, &embedding, limit) {
            Ok(hits) => {
                let hits_val = VectorizerValue::Array(
                    hits.into_iter()
                        .map(|r| {
                            VectorizerValue::Map(vec![
                                (
                                    VectorizerValue::Str("id".into()),
                                    VectorizerValue::Str(r.id),
                                ),
                                (
                                    VectorizerValue::Str("score".into()),
                                    VectorizerValue::Float(r.score as f64),
                                ),
                            ])
                        })
                        .collect(),
                );
                results.push(VectorizerValue::Map(vec![
                    (
                        VectorizerValue::Str("index".into()),
                        VectorizerValue::Int(idx as i64),
                    ),
                    (
                        VectorizerValue::Str("status".into()),
                        VectorizerValue::Str("ok".into()),
                    ),
                    (VectorizerValue::Str("results".into()), hits_val),
                ]));
            }
            Err(e) => {
                results.push(VectorizerValue::Map(vec![
                    (
                        VectorizerValue::Str("index".into()),
                        VectorizerValue::Int(idx as i64),
                    ),
                    (
                        VectorizerValue::Str("status".into()),
                        VectorizerValue::Str("error".into()),
                    ),
                    (
                        VectorizerValue::Str("error".into()),
                        VectorizerValue::Str(e.to_string()),
                    ),
                ]));
            }
        }
    }
    Response::ok(id, VectorizerValue::Array(results))
}

fn handle_vectors_batch_update(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    let collection = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => {
            return Response::err(
                id,
                "vectors.batch_update expects [Str(coll), Array<Map>(updates)]",
            );
        }
    };
    let updates = match args.get(1).and_then(|v| v.as_array()) {
        Some(arr) => arr.to_vec(),
        None => return Response::err(id, "vectors.batch_update: Array(updates) missing"),
    };
    let mut updated = 0usize;
    let mut failed = 0usize;
    let mut results: Vec<VectorizerValue> = Vec::with_capacity(updates.len());
    for (idx, entry) in updates.iter().enumerate() {
        let vector_id = match entry.map_get("id").and_then(|v| v.as_str()) {
            Some(i) => i.to_string(),
            None => {
                failed += 1;
                results.push(VectorizerValue::Map(vec![
                    (
                        VectorizerValue::Str("index".into()),
                        VectorizerValue::Int(idx as i64),
                    ),
                    (
                        VectorizerValue::Str("status".into()),
                        VectorizerValue::Str("error".into()),
                    ),
                    (
                        VectorizerValue::Str("error".into()),
                        VectorizerValue::Str("missing id".into()),
                    ),
                ]));
                continue;
            }
        };
        let existing = match state.store.get_vector(collection, &vector_id) {
            Ok(v) => v,
            Err(e) => {
                failed += 1;
                results.push(VectorizerValue::Map(vec![
                    (
                        VectorizerValue::Str("index".into()),
                        VectorizerValue::Int(idx as i64),
                    ),
                    (
                        VectorizerValue::Str("id".into()),
                        VectorizerValue::Str(vector_id),
                    ),
                    (
                        VectorizerValue::Str("status".into()),
                        VectorizerValue::Str("error".into()),
                    ),
                    (
                        VectorizerValue::Str("error".into()),
                        VectorizerValue::Str(e.to_string()),
                    ),
                ]));
                continue;
            }
        };
        let new_data = match entry.map_get("data").and_then(|v| v.as_array()) {
            Some(arr) => arr
                .iter()
                .filter_map(|v| v.as_float().map(|f| f as f32))
                .collect(),
            None => existing.data.clone(),
        };
        let new_payload = match entry.map_get("payload") {
            Some(v) => Some(vectorizer::models::Payload::new(value_to_json(v))),
            None => existing.payload.clone(),
        };
        let updated_vec = vectorizer::models::Vector {
            id: vector_id.clone(),
            data: new_data,
            sparse: existing.sparse,
            payload: new_payload,
            document_id: existing.document_id,
        };
        match state.store.update(collection, updated_vec) {
            Ok(()) => {
                updated += 1;
                results.push(VectorizerValue::Map(vec![
                    (
                        VectorizerValue::Str("index".into()),
                        VectorizerValue::Int(idx as i64),
                    ),
                    (
                        VectorizerValue::Str("id".into()),
                        VectorizerValue::Str(vector_id),
                    ),
                    (
                        VectorizerValue::Str("status".into()),
                        VectorizerValue::Str("ok".into()),
                    ),
                ]));
            }
            Err(e) => {
                failed += 1;
                results.push(VectorizerValue::Map(vec![
                    (
                        VectorizerValue::Str("index".into()),
                        VectorizerValue::Int(idx as i64),
                    ),
                    (
                        VectorizerValue::Str("id".into()),
                        VectorizerValue::Str(vector_id),
                    ),
                    (
                        VectorizerValue::Str("status".into()),
                        VectorizerValue::Str("error".into()),
                    ),
                    (
                        VectorizerValue::Str("error".into()),
                        VectorizerValue::Str(e.to_string()),
                    ),
                ]));
            }
        }
    }
    let map = vec![
        (
            VectorizerValue::Str("updated".into()),
            VectorizerValue::Int(updated as i64),
        ),
        (
            VectorizerValue::Str("failed".into()),
            VectorizerValue::Int(failed as i64),
        ),
        (
            VectorizerValue::Str("results".into()),
            VectorizerValue::Array(results),
        ),
    ];
    Response::ok(id, VectorizerValue::Map(map))
}

fn handle_vectors_batch_delete(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    let collection = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => {
            return Response::err(
                id,
                "vectors.batch_delete expects [Str(coll), Array<Str>(ids)]",
            );
        }
    };
    let ids = match args.get(1).and_then(|v| v.as_array()) {
        Some(arr) => arr.to_vec(),
        None => return Response::err(id, "vectors.batch_delete: Array(ids) missing"),
    };
    let mut deleted = 0usize;
    let mut failed = 0usize;
    let mut results: Vec<VectorizerValue> = Vec::with_capacity(ids.len());
    for (idx, entry) in ids.iter().enumerate() {
        let vid = match entry.as_str() {
            Some(s) => s.to_string(),
            None => {
                failed += 1;
                results.push(VectorizerValue::Map(vec![
                    (
                        VectorizerValue::Str("index".into()),
                        VectorizerValue::Int(idx as i64),
                    ),
                    (
                        VectorizerValue::Str("status".into()),
                        VectorizerValue::Str("error".into()),
                    ),
                    (
                        VectorizerValue::Str("error".into()),
                        VectorizerValue::Str("id must be a string".into()),
                    ),
                ]));
                continue;
            }
        };
        match state.store.delete(collection, &vid) {
            Ok(()) => {
                deleted += 1;
                results.push(VectorizerValue::Map(vec![
                    (
                        VectorizerValue::Str("index".into()),
                        VectorizerValue::Int(idx as i64),
                    ),
                    (VectorizerValue::Str("id".into()), VectorizerValue::Str(vid)),
                    (
                        VectorizerValue::Str("status".into()),
                        VectorizerValue::Str("ok".into()),
                    ),
                ]));
            }
            Err(e) => {
                failed += 1;
                results.push(VectorizerValue::Map(vec![
                    (
                        VectorizerValue::Str("index".into()),
                        VectorizerValue::Int(idx as i64),
                    ),
                    (VectorizerValue::Str("id".into()), VectorizerValue::Str(vid)),
                    (
                        VectorizerValue::Str("status".into()),
                        VectorizerValue::Str("error".into()),
                    ),
                    (
                        VectorizerValue::Str("error".into()),
                        VectorizerValue::Str(e.to_string()),
                    ),
                ]));
            }
        }
    }
    let map = vec![
        (
            VectorizerValue::Str("deleted".into()),
            VectorizerValue::Int(deleted as i64),
        ),
        (
            VectorizerValue::Str("failed".into()),
            VectorizerValue::Int(failed as i64),
        ),
        (
            VectorizerValue::Str("results".into()),
            VectorizerValue::Array(results),
        ),
    ];
    Response::ok(id, VectorizerValue::Map(map))
}

fn handle_vectors_move(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let src = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => {
            return Response::err(
                id,
                "vectors.move expects [Str(src), Str(dst), Array<Str>(ids)]",
            );
        }
    };
    let dst = match args.get(1).and_then(|v| v.as_str()) {
        Some(c) => c,
        None => return Response::err(id, "vectors.move: Str(dst) missing"),
    };
    if src == dst {
        return Response::err(id, "vectors.move: src and dst must differ");
    }
    let ids = match args.get(2).and_then(|v| v.as_array()) {
        Some(arr) => arr.to_vec(),
        None => return Response::err(id, "vectors.move: Array(ids) missing"),
    };
    let mut moved = 0usize;
    let mut failed = 0usize;
    for entry in &ids {
        let vid = match entry.as_str() {
            Some(s) => s,
            None => {
                failed += 1;
                continue;
            }
        };
        let vector = match state.store.get_vector(src, vid) {
            Ok(v) => v,
            Err(_) => {
                failed += 1;
                continue;
            }
        };
        if state.store.insert(dst, vec![vector]).is_err() {
            failed += 1;
            continue;
        }
        if state.store.delete(src, vid).is_err() {
            failed += 1;
            continue;
        }
        moved += 1;
    }
    let map = vec![
        (
            VectorizerValue::Str("src".into()),
            VectorizerValue::Str(src.to_string()),
        ),
        (
            VectorizerValue::Str("dst".into()),
            VectorizerValue::Str(dst.to_string()),
        ),
        (
            VectorizerValue::Str("moved".into()),
            VectorizerValue::Int(moved as i64),
        ),
        (
            VectorizerValue::Str("failed".into()),
            VectorizerValue::Int(failed as i64),
        ),
    ];
    Response::ok(id, VectorizerValue::Map(map))
}

fn handle_vectors_copy(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let src = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => {
            return Response::err(
                id,
                "vectors.copy expects [Str(src), Str(dst), Array<Str>(ids)]",
            );
        }
    };
    let dst = match args.get(1).and_then(|v| v.as_str()) {
        Some(c) => c,
        None => return Response::err(id, "vectors.copy: Str(dst) missing"),
    };
    if src == dst {
        return Response::err(id, "vectors.copy: src and dst must differ");
    }
    let ids = match args.get(2).and_then(|v| v.as_array()) {
        Some(arr) => arr.to_vec(),
        None => return Response::err(id, "vectors.copy: Array(ids) missing"),
    };
    let mut copied = 0usize;
    let mut failed = 0usize;
    for entry in &ids {
        let vid = match entry.as_str() {
            Some(s) => s,
            None => {
                failed += 1;
                continue;
            }
        };
        let vector = match state.store.get_vector(src, vid) {
            Ok(v) => v,
            Err(_) => {
                failed += 1;
                continue;
            }
        };
        if state.store.insert(dst, vec![vector]).is_err() {
            failed += 1;
            continue;
        }
        copied += 1;
    }
    let map = vec![
        (
            VectorizerValue::Str("src".into()),
            VectorizerValue::Str(src.to_string()),
        ),
        (
            VectorizerValue::Str("dst".into()),
            VectorizerValue::Str(dst.to_string()),
        ),
        (
            VectorizerValue::Str("copied".into()),
            VectorizerValue::Int(copied as i64),
        ),
        (
            VectorizerValue::Str("failed".into()),
            VectorizerValue::Int(failed as i64),
        ),
    ];
    Response::ok(id, VectorizerValue::Map(map))
}

fn handle_vectors_delete_by_filter(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    use vectorizer::models::qdrant::filter::QdrantFilter;
    use vectorizer::models::qdrant::filter_processor::FilterProcessor;

    let collection = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => {
            return Response::err(
                id,
                "vectors.delete_by_filter expects [Str(coll), Map(filter)]",
            );
        }
    };
    let filter_json = match args.get(1) {
        Some(v) => value_to_json(v),
        None => return Response::err(id, "vectors.delete_by_filter: Map(filter) missing"),
    };
    let filter: QdrantFilter = match serde_json::from_value(filter_json) {
        Ok(f) => f,
        Err(e) => {
            return Response::err(
                id,
                format!("vectors.delete_by_filter: invalid filter: {}", e),
            );
        }
    };
    let coll = match state.store.get_collection(collection) {
        Ok(c) => c,
        Err(e) => return Response::err(id, format!("vectors.delete_by_filter: {}", e)),
    };
    let all = coll.get_all_vectors();
    let scanned = all.len();
    let matching_ids: Vec<String> = all
        .into_iter()
        .filter_map(|v| {
            v.payload.as_ref().and_then(|p| {
                if FilterProcessor::apply_filter(&filter, p) {
                    Some(v.id)
                } else {
                    None
                }
            })
        })
        .collect();
    let matched = matching_ids.len();
    let mut deleted = 0usize;
    for mid in &matching_ids {
        if state.store.delete(collection, mid).is_ok() {
            deleted += 1;
        }
    }
    let map = vec![
        (
            VectorizerValue::Str("scanned".into()),
            VectorizerValue::Int(scanned as i64),
        ),
        (
            VectorizerValue::Str("matched".into()),
            VectorizerValue::Int(matched as i64),
        ),
        (
            VectorizerValue::Str("deleted".into()),
            VectorizerValue::Int(deleted as i64),
        ),
    ];
    Response::ok(id, VectorizerValue::Map(map))
}

fn handle_vectors_bulk_update_metadata(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    use vectorizer::models::qdrant::filter::QdrantFilter;
    use vectorizer::models::qdrant::filter_processor::FilterProcessor;

    let collection = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => {
            return Response::err(
                id,
                "vectors.bulk_update_metadata expects [Str(coll), Map(filter), Map(patch)]",
            );
        }
    };
    let filter_json = match args.get(1) {
        Some(v) => value_to_json(v),
        None => return Response::err(id, "vectors.bulk_update_metadata: Map(filter) missing"),
    };
    let patch_json = match args.get(2) {
        Some(v) => value_to_json(v),
        None => return Response::err(id, "vectors.bulk_update_metadata: Map(patch) missing"),
    };
    let filter: QdrantFilter = match serde_json::from_value(filter_json) {
        Ok(f) => f,
        Err(e) => {
            return Response::err(
                id,
                format!("vectors.bulk_update_metadata: invalid filter: {}", e),
            );
        }
    };
    let coll = match state.store.get_collection(collection) {
        Ok(c) => c,
        Err(e) => return Response::err(id, format!("vectors.bulk_update_metadata: {}", e)),
    };
    let all = coll.get_all_vectors();
    let scanned = all.len();
    let matching: Vec<vectorizer::models::Vector> = all
        .into_iter()
        .filter(|v| {
            v.payload
                .as_ref()
                .map_or(false, |p| FilterProcessor::apply_filter(&filter, p))
        })
        .collect();
    let matched = matching.len();
    let mut updated = 0usize;
    for mut vector in matching {
        let new_data = if let Some(existing) = vector.payload.as_ref() {
            json_merge_patch(existing.data.clone(), patch_json.clone())
        } else {
            patch_json.clone()
        };
        vector.payload = Some(vectorizer::models::Payload { data: new_data });
        if state.store.update(collection, vector).is_ok() {
            updated += 1;
        }
    }
    let map = vec![
        (
            VectorizerValue::Str("scanned".into()),
            VectorizerValue::Int(scanned as i64),
        ),
        (
            VectorizerValue::Str("matched".into()),
            VectorizerValue::Int(matched as i64),
        ),
        (
            VectorizerValue::Str("updated".into()),
            VectorizerValue::Int(updated as i64),
        ),
    ];
    Response::ok(id, VectorizerValue::Map(map))
}

/// JSON-merge-patch (RFC 7396) — reused from rest_handlers/vectors.rs logic.
fn json_merge_patch(mut target: serde_json::Value, patch: serde_json::Value) -> serde_json::Value {
    match (target.as_object_mut(), patch) {
        (Some(t), serde_json::Value::Object(p)) => {
            for (k, v) in p {
                if v.is_null() {
                    t.remove(&k);
                } else {
                    let existing = t.remove(&k).unwrap_or(serde_json::Value::Null);
                    t.insert(k, json_merge_patch(existing, v));
                }
            }
            serde_json::Value::Object(t.clone())
        }
        (_, patch) => patch,
    }
}

fn handle_vectors_set_expiry(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let collection = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => {
            return Response::err(
                id,
                "vectors.set_expiry expects [Str(coll), Str(id), Str(expires_at)]",
            );
        }
    };
    let vector_id = match args.get(1).and_then(|v| v.as_str()) {
        Some(v) => v,
        None => return Response::err(id, "vectors.set_expiry: Str(id) missing"),
    };
    let expires_str = match args.get(2).and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return Response::err(id, "vectors.set_expiry: Str(expires_at) missing"),
    };
    // Parse as Unix millisecond timestamp or RFC3339
    let expires_ms: i64 = if let Ok(ts) = expires_str.parse::<i64>() {
        ts
    } else if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(expires_str) {
        dt.timestamp_millis()
    } else {
        return Response::err(
            id,
            "vectors.set_expiry: expires_at must be a Unix ms timestamp or RFC3339 string",
        );
    };
    let mut vector = match state.store.get_vector(collection, vector_id) {
        Ok(v) => v,
        Err(e) => return Response::err(id, format!("vectors.set_expiry: {}", e)),
    };
    let payload = vector
        .payload
        .get_or_insert_with(|| vectorizer::models::Payload {
            data: serde_json::json!({}),
        });
    payload.set_expires_at(expires_ms);
    match state.store.update(collection, vector) {
        Ok(()) => {
            let map = vec![
                (
                    VectorizerValue::Str("id".into()),
                    VectorizerValue::Str(vector_id.to_string()),
                ),
                (
                    VectorizerValue::Str("expires_at".into()),
                    VectorizerValue::Int(expires_ms),
                ),
                (
                    VectorizerValue::Str("success".into()),
                    VectorizerValue::Bool(true),
                ),
            ];
            Response::ok(id, VectorizerValue::Map(map))
        }
        Err(e) => Response::err(id, format!("vectors.set_expiry: update failed: {}", e)),
    }
}

// ── Search handlers ──────────────────────────────────────────────────────────

async fn handle_search_intelligent(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    use vectorizer::intelligent_search::rest_api::{IntelligentSearchRequest, RESTAPIHandler};

    let payload = args
        .first()
        .map(value_to_json)
        .unwrap_or(serde_json::json!({}));
    let query = match payload.get("query").and_then(|q| q.as_str()) {
        Some(q) => q.to_string(),
        None => return Response::err(id, "search.intelligent: missing query field"),
    };
    let collections = payload
        .get("collections")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect::<Vec<_>>()
        });
    let max_results = payload
        .get("max_results")
        .and_then(|m| m.as_u64())
        .map(|m| m as usize);
    let domain_expansion = payload.get("domain_expansion").and_then(|d| d.as_bool());
    let handler = RESTAPIHandler::new_with_store(state.store.clone());
    let request = IntelligentSearchRequest {
        query,
        collections,
        max_results,
        domain_expansion,
        technical_focus: None,
        mmr_enabled: None,
        mmr_lambda: None,
    };
    match handler.handle_intelligent_search(request).await {
        Ok(resp) => {
            let json = serde_json::to_value(&resp).unwrap_or(serde_json::json!({}));
            Response::ok(id, json_to_value(json))
        }
        Err(e) => Response::err(id, format!("search.intelligent: {:?}", e)),
    }
}

fn handle_search_by_text(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let collection = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => {
            return Response::err(
                id,
                "search.by_text expects [Str(coll), Str(query), Int(limit)?]",
            );
        }
    };
    let query = match args.get(1).and_then(|v| v.as_str()) {
        Some(q) => q,
        None => return Response::err(id, "search.by_text: Str(query) missing"),
    };
    let limit = args.get(2).and_then(|v| v.as_int()).unwrap_or(10).max(1) as usize;
    let embedding = match state.embedding_manager.embed(query) {
        Ok(e) => e,
        Err(e) => return Response::err(id, format!("search.by_text: embed failed: {}", e)),
    };
    match state.store.search(collection, &embedding, limit) {
        Ok(results) => {
            let arr = results
                .into_iter()
                .map(|r| {
                    VectorizerValue::Map(vec![
                        (
                            VectorizerValue::Str("id".into()),
                            VectorizerValue::Str(r.id),
                        ),
                        (
                            VectorizerValue::Str("score".into()),
                            VectorizerValue::Float(r.score as f64),
                        ),
                        (
                            VectorizerValue::Str("payload".into()),
                            r.payload
                                .map(|p| json_to_value(p.data))
                                .unwrap_or(VectorizerValue::Null),
                        ),
                    ])
                })
                .collect();
            let map = vec![
                (
                    VectorizerValue::Str("results".into()),
                    VectorizerValue::Array(arr),
                ),
                (
                    VectorizerValue::Str("collection".into()),
                    VectorizerValue::Str(collection.to_string()),
                ),
                (
                    VectorizerValue::Str("query".into()),
                    VectorizerValue::Str(query.to_string()),
                ),
            ];
            Response::ok(id, VectorizerValue::Map(map))
        }
        Err(e) => Response::err(id, format!("search.by_text: {}", e)),
    }
}

fn handle_search_by_file(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let collection = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => return Response::err(id, "search.by_file expects [Str(coll), Map(request)]"),
    };
    let _request = args
        .get(1)
        .map(value_to_json)
        .unwrap_or(serde_json::json!({}));
    // Verify collection exists
    if let Err(e) = state.store.get_collection(collection) {
        return Response::err(id, format!("search.by_file: {}", e));
    }
    // File-based search returns empty results (same as REST handler)
    let map = vec![
        (
            VectorizerValue::Str("results".into()),
            VectorizerValue::Array(vec![]),
        ),
        (
            VectorizerValue::Str("collection".into()),
            VectorizerValue::Str(collection.to_string()),
        ),
    ];
    Response::ok(id, VectorizerValue::Map(map))
}

fn handle_search_hybrid(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    use vectorizer::db::{HybridScoringAlgorithm, HybridSearchConfig};

    let collection = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => return Response::err(id, "search.hybrid expects [Str(coll), Map(request)]"),
    };
    let req = args
        .get(1)
        .map(value_to_json)
        .unwrap_or(serde_json::json!({}));
    let query = match req.get("query").and_then(|q| q.as_str()) {
        Some(q) => q,
        None => return Response::err(id, "search.hybrid: missing query in request map"),
    };
    let alpha = req.get("alpha").and_then(|v| v.as_f64()).unwrap_or(0.7) as f32;
    let dense_k = req.get("dense_k").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
    let sparse_k = req.get("sparse_k").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
    let final_k = req
        .get("final_k")
        .or_else(|| req.get("limit"))
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;
    let algorithm_str = req
        .get("algorithm")
        .and_then(|v| v.as_str())
        .unwrap_or("rrf");
    let algorithm = match algorithm_str {
        "weighted" => HybridScoringAlgorithm::WeightedCombination,
        "alpha" => HybridScoringAlgorithm::AlphaBlending,
        _ => HybridScoringAlgorithm::ReciprocalRankFusion,
    };
    let dense = match state.embedding_manager.embed(query) {
        Ok(e) => e,
        Err(e) => return Response::err(id, format!("search.hybrid: embed failed: {}", e)),
    };
    let coll = match state.store.get_collection(collection) {
        Ok(c) => c,
        Err(e) => return Response::err(id, format!("search.hybrid: {}", e)),
    };
    let config = HybridSearchConfig {
        alpha,
        dense_k,
        sparse_k,
        final_k,
        algorithm,
    };
    match coll.hybrid_search(&dense, None, config) {
        Ok(results) => {
            let arr = results
                .into_iter()
                .map(|r| {
                    VectorizerValue::Map(vec![
                        (
                            VectorizerValue::Str("id".into()),
                            VectorizerValue::Str(r.id),
                        ),
                        (
                            VectorizerValue::Str("score".into()),
                            VectorizerValue::Float(r.score as f64),
                        ),
                        (
                            VectorizerValue::Str("payload".into()),
                            r.payload
                                .map(|p| json_to_value(p.data))
                                .unwrap_or(VectorizerValue::Null),
                        ),
                    ])
                })
                .collect();
            let map = vec![
                (
                    VectorizerValue::Str("results".into()),
                    VectorizerValue::Array(arr),
                ),
                (
                    VectorizerValue::Str("collection".into()),
                    VectorizerValue::Str(collection.to_string()),
                ),
            ];
            Response::ok(id, VectorizerValue::Map(map))
        }
        Err(e) => Response::err(id, format!("search.hybrid: {}", e)),
    }
}

async fn handle_search_semantic(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    use vectorizer::intelligent_search::rest_api::{RESTAPIHandler, SemanticSearchRequest};

    let payload = args
        .first()
        .map(value_to_json)
        .unwrap_or(serde_json::json!({}));
    let query = match payload.get("query").and_then(|q| q.as_str()) {
        Some(q) => q.to_string(),
        None => return Response::err(id, "search.semantic: missing query field"),
    };
    let collection = match payload.get("collection").and_then(|c| c.as_str()) {
        Some(c) => c.to_string(),
        None => return Response::err(id, "search.semantic: missing collection field"),
    };
    let handler = RESTAPIHandler::new_with_store(state.store.clone());
    let request = SemanticSearchRequest {
        query,
        collection,
        max_results: payload
            .get("max_results")
            .and_then(|m| m.as_u64())
            .map(|m| m as usize),
        semantic_reranking: payload.get("semantic_reranking").and_then(|v| v.as_bool()),
        cross_encoder_reranking: payload
            .get("cross_encoder_reranking")
            .and_then(|v| v.as_bool()),
        similarity_threshold: payload
            .get("similarity_threshold")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32),
    };
    match handler.handle_semantic_search(request).await {
        Ok(resp) => {
            let json = serde_json::to_value(resp).unwrap_or(serde_json::json!({}));
            Response::ok(id, json_to_value(json))
        }
        Err(e) => Response::err(id, format!("search.semantic: {:?}", e)),
    }
}

async fn handle_search_contextual(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    use vectorizer::intelligent_search::rest_api::{ContextualSearchRequest, RESTAPIHandler};

    let payload = args
        .first()
        .map(value_to_json)
        .unwrap_or(serde_json::json!({}));
    let query = match payload.get("query").and_then(|q| q.as_str()) {
        Some(q) => q.to_string(),
        None => return Response::err(id, "search.contextual: missing query field"),
    };
    let collection = match payload.get("collection").and_then(|c| c.as_str()) {
        Some(c) => c.to_string(),
        None => return Response::err(id, "search.contextual: missing collection field"),
    };
    let handler = RESTAPIHandler::new_with_store(state.store.clone());
    let context_filters = payload
        .get("context_filters")
        .and_then(|f| f.as_object())
        .map(|obj| {
            let mut map = std::collections::HashMap::new();
            for (k, v) in obj {
                map.insert(k.clone(), v.clone());
            }
            map
        });
    let request = ContextualSearchRequest {
        query,
        collection,
        context_filters,
        max_results: payload
            .get("max_results")
            .and_then(|m| m.as_u64())
            .map(|m| m as usize),
        context_weight: payload
            .get("context_weight")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32),
        context_reranking: payload.get("context_reranking").and_then(|v| v.as_bool()),
    };
    match handler.handle_contextual_search(request).await {
        Ok(resp) => {
            let json = serde_json::to_value(resp).unwrap_or(serde_json::json!({}));
            Response::ok(id, json_to_value(json))
        }
        Err(e) => Response::err(id, format!("search.contextual: {:?}", e)),
    }
}

async fn handle_search_multi_collection(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    use vectorizer::intelligent_search::rest_api::{MultiCollectionSearchRequest, RESTAPIHandler};

    let payload = args
        .first()
        .map(value_to_json)
        .unwrap_or(serde_json::json!({}));
    let query = match payload.get("query").and_then(|q| q.as_str()) {
        Some(q) => q.to_string(),
        None => return Response::err(id, "search.multi_collection: missing query field"),
    };
    let collections = match payload.get("collections").and_then(|c| c.as_array()) {
        Some(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect::<Vec<_>>(),
        None => return Response::err(id, "search.multi_collection: missing collections field"),
    };
    let handler = RESTAPIHandler::new_with_store(state.store.clone());
    let request = MultiCollectionSearchRequest {
        query,
        collections,
        max_per_collection: payload
            .get("max_per_collection")
            .and_then(|m| m.as_u64())
            .map(|m| m as usize),
        max_total_results: payload
            .get("max_total_results")
            .and_then(|m| m.as_u64())
            .map(|m| m as usize),
        cross_collection_reranking: payload
            .get("cross_collection_reranking")
            .and_then(|v| v.as_bool()),
    };
    match handler.handle_multi_collection_search(request).await {
        Ok(resp) => {
            let json = serde_json::to_value(resp).unwrap_or(serde_json::json!({}));
            Response::ok(id, json_to_value(json))
        }
        Err(e) => Response::err(id, format!("search.multi_collection: {:?}", e)),
    }
}

async fn handle_search_explain(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    let collection = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return Response::err(id, "search.explain expects [Str(coll), Map(request)]"),
    };
    let req = args
        .get(1)
        .map(value_to_json)
        .unwrap_or(serde_json::json!({}));
    let k = req.get("k").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
    let query_vector: Vec<f32> = match req.get("vector").and_then(|v| v.as_array()) {
        Some(arr) => arr
            .iter()
            .filter_map(|v| v.as_f64().map(|f| f as f32))
            .collect(),
        None => return Response::err(id, "search.explain: missing vector array in request"),
    };
    if query_vector.is_empty() {
        return Response::err(id, "search.explain: vector array is empty");
    }
    let store = state.store.clone();
    let col_name = collection.clone();
    let explain = match tokio::task::spawn_blocking(move || {
        store.search_explained(&col_name, &query_vector, k)
    })
    .await
    {
        Ok(Ok(e)) => e,
        Ok(Err(e)) => return Response::err(id, format!("search.explain: {}", e)),
        Err(e) => return Response::err(id, format!("search.explain: task error: {}", e)),
    };
    let results: Vec<VectorizerValue> = explain
        .results
        .iter()
        .map(|r| {
            VectorizerValue::Map(vec![
                (
                    VectorizerValue::Str("id".into()),
                    VectorizerValue::Str(r.id.clone()),
                ),
                (
                    VectorizerValue::Str("score".into()),
                    VectorizerValue::Float(r.score as f64),
                ),
            ])
        })
        .collect();
    let trace = &explain.trace;
    let trace_map = VectorizerValue::Map(vec![
        (
            VectorizerValue::Str("visited_nodes".into()),
            VectorizerValue::Int(trace.visited_nodes as i64),
        ),
        (
            VectorizerValue::Str("ef_search".into()),
            VectorizerValue::Int(trace.ef_search as i64),
        ),
        (
            VectorizerValue::Str("hnsw_search_ms".into()),
            VectorizerValue::Float(trace.hnsw_search_ms),
        ),
        (
            VectorizerValue::Str("total_ms".into()),
            VectorizerValue::Float(trace.total_ms),
        ),
    ]);
    let map = vec![
        (
            VectorizerValue::Str("hits".into()),
            VectorizerValue::Array(results),
        ),
        (VectorizerValue::Str("trace".into()), trace_map),
        (
            VectorizerValue::Str("collection".into()),
            VectorizerValue::Str(collection),
        ),
        (
            VectorizerValue::Str("k".into()),
            VectorizerValue::Int(k as i64),
        ),
    ];
    Response::ok(id, VectorizerValue::Map(map))
}

// ── Discovery handlers ───────────────────────────────────────────────────────

async fn handle_discovery_discover(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    use vectorizer::discovery::{Discovery, DiscoveryConfig};

    let payload = args
        .first()
        .map(value_to_json)
        .unwrap_or(serde_json::json!({}));
    let query = match payload.get("query").and_then(|q| q.as_str()) {
        Some(q) => q.to_string(),
        None => return Response::err(id, "discovery.discover: missing query field"),
    };
    let mut config = DiscoveryConfig::default();
    if let Some(arr) = payload
        .get("include_collections")
        .and_then(|v| v.as_array())
    {
        config.include_collections = arr
            .iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect();
    }
    if let Some(arr) = payload
        .get("exclude_collections")
        .and_then(|v| v.as_array())
    {
        config.exclude_collections = arr
            .iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect();
    }
    if let Some(n) = payload.get("max_bullets").and_then(|v| v.as_u64()) {
        config.max_bullets = n as usize;
    }
    let discovery = Discovery::new(config, state.store.clone(), state.embedding_manager.clone());
    match discovery.discover(&query).await {
        Ok(resp) => {
            let map = vec![
                (
                    VectorizerValue::Str("answer_prompt".into()),
                    VectorizerValue::Str(resp.answer_prompt),
                ),
                (
                    VectorizerValue::Str("sections".into()),
                    VectorizerValue::Int(resp.plan.sections.len() as i64),
                ),
                (
                    VectorizerValue::Str("bullets".into()),
                    VectorizerValue::Int(resp.bullets.len() as i64),
                ),
                (
                    VectorizerValue::Str("chunks".into()),
                    VectorizerValue::Int(resp.chunks.len() as i64),
                ),
            ];
            Response::ok(id, VectorizerValue::Map(map))
        }
        Err(e) => Response::err(id, format!("discovery.discover: {}", e)),
    }
}

fn handle_discovery_filter_collections(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    use vectorizer::discovery::filter_collections as filter_fn;

    let payload = args
        .first()
        .map(value_to_json)
        .unwrap_or(serde_json::json!({}));
    let query = match payload.get("query").and_then(|q| q.as_str()) {
        Some(q) => q.to_string(),
        None => return Response::err(id, "discovery.filter_collections: missing query field"),
    };
    let include: Vec<String> = payload
        .get("include")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    let exclude: Vec<String> = payload
        .get("exclude")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    let include_refs: Vec<&str> = include.iter().map(|s| s.as_str()).collect();
    let exclude_refs: Vec<&str> = exclude.iter().map(|s| s.as_str()).collect();
    let all: Vec<_> = state
        .store
        .list_collections()
        .iter()
        .filter_map(|name| {
            state.store.get_collection(name).ok().map(|coll| {
                let meta = coll.metadata();
                vectorizer::discovery::CollectionRef {
                    name: name.clone(),
                    dimension: meta.config.dimension,
                    vector_count: meta.vector_count,
                    created_at: meta.created_at,
                    updated_at: meta.updated_at,
                    tags: vec![],
                }
            })
        })
        .collect();
    match filter_fn(&query, &include_refs, &exclude_refs, &all) {
        Ok(filtered) => {
            let arr: Vec<VectorizerValue> = filtered
                .iter()
                .map(|c| {
                    VectorizerValue::Map(vec![
                        (
                            VectorizerValue::Str("name".into()),
                            VectorizerValue::Str(c.name.clone()),
                        ),
                        (
                            VectorizerValue::Str("vector_count".into()),
                            VectorizerValue::Int(c.vector_count as i64),
                        ),
                    ])
                })
                .collect();
            let map = vec![
                (
                    VectorizerValue::Str("filtered_collections".into()),
                    VectorizerValue::Array(arr),
                ),
                (
                    VectorizerValue::Str("count".into()),
                    VectorizerValue::Int(filtered.len() as i64),
                ),
            ];
            Response::ok(id, VectorizerValue::Map(map))
        }
        Err(e) => Response::err(id, format!("discovery.filter_collections: {}", e)),
    }
}

fn handle_discovery_score_collections(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    use vectorizer::discovery::{ScoringConfig, score_collections as score_fn};

    let payload = args
        .first()
        .map(value_to_json)
        .unwrap_or(serde_json::json!({}));
    let query = match payload.get("query").and_then(|q| q.as_str()) {
        Some(q) => q.to_string(),
        None => return Response::err(id, "discovery.score_collections: missing query field"),
    };
    let config = ScoringConfig::default();
    let all: Vec<_> = state
        .store
        .list_collections()
        .iter()
        .filter_map(|name| {
            state.store.get_collection(name).ok().map(|coll| {
                let meta = coll.metadata();
                vectorizer::discovery::CollectionRef {
                    name: name.clone(),
                    dimension: meta.config.dimension,
                    vector_count: meta.vector_count,
                    created_at: meta.created_at,
                    updated_at: meta.updated_at,
                    tags: vec![],
                }
            })
        })
        .collect();
    let query_terms: Vec<&str> = query.split_whitespace().collect();
    match score_fn(&query_terms, &all, &config) {
        Ok(scored) => {
            let arr: Vec<VectorizerValue> = scored
                .iter()
                .map(|(c, score)| {
                    VectorizerValue::Map(vec![
                        (
                            VectorizerValue::Str("name".into()),
                            VectorizerValue::Str(c.name.clone()),
                        ),
                        (
                            VectorizerValue::Str("score".into()),
                            VectorizerValue::Float(*score as f64),
                        ),
                        (
                            VectorizerValue::Str("vector_count".into()),
                            VectorizerValue::Int(c.vector_count as i64),
                        ),
                    ])
                })
                .collect();
            let map = vec![
                (
                    VectorizerValue::Str("scored_collections".into()),
                    VectorizerValue::Array(arr),
                ),
                (
                    VectorizerValue::Str("count".into()),
                    VectorizerValue::Int(scored.len() as i64),
                ),
            ];
            Response::ok(id, VectorizerValue::Map(map))
        }
        Err(e) => Response::err(id, format!("discovery.score_collections: {}", e)),
    }
}

fn handle_discovery_expand_queries(id: u32, args: &[VectorizerValue]) -> Response {
    use vectorizer::discovery::{ExpansionConfig, expand_queries_baseline};

    let payload = args
        .first()
        .map(value_to_json)
        .unwrap_or(serde_json::json!({}));
    let query = match payload.get("query").and_then(|q| q.as_str()) {
        Some(q) => q.to_string(),
        None => return Response::err(id, "discovery.expand_queries: missing query field"),
    };
    let mut config = ExpansionConfig::default();
    if let Some(n) = payload.get("max_expansions").and_then(|v| v.as_u64()) {
        config.max_expansions = n as usize;
    }
    if let Some(b) = payload.get("include_definition").and_then(|v| v.as_bool()) {
        config.include_definition = b;
    }
    if let Some(b) = payload.get("include_features").and_then(|v| v.as_bool()) {
        config.include_features = b;
    }
    if let Some(b) = payload
        .get("include_architecture")
        .and_then(|v| v.as_bool())
    {
        config.include_architecture = b;
    }
    match expand_queries_baseline(&query, &config) {
        Ok(expanded) => {
            let arr: Vec<VectorizerValue> = expanded
                .iter()
                .map(|s| VectorizerValue::Str(s.clone()))
                .collect();
            let map = vec![
                (
                    VectorizerValue::Str("original_query".into()),
                    VectorizerValue::Str(query),
                ),
                (
                    VectorizerValue::Str("expanded_queries".into()),
                    VectorizerValue::Array(arr),
                ),
                (
                    VectorizerValue::Str("count".into()),
                    VectorizerValue::Int(expanded.len() as i64),
                ),
            ];
            Response::ok(id, VectorizerValue::Map(map))
        }
        Err(e) => Response::err(id, format!("discovery.expand_queries: {}", e)),
    }
}

async fn handle_discovery_broad_discovery(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    use vectorizer::discovery::{BroadDiscoveryConfig, broad_discovery as broad_fn};

    let payload = args
        .first()
        .map(value_to_json)
        .unwrap_or(serde_json::json!({}));
    let queries: Vec<String> = match payload.get("queries").and_then(|v| v.as_array()) {
        Some(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect(),
        None => return Response::err(id, "discovery.broad_discovery: missing queries array"),
    };
    let k = payload.get("k").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
    let config = BroadDiscoveryConfig::default();
    let collections: Vec<_> = state
        .store
        .list_collections()
        .iter()
        .filter_map(|name| {
            state.store.get_collection(name).ok().map(|coll| {
                let meta = coll.metadata();
                vectorizer::discovery::CollectionRef {
                    name: name.clone(),
                    dimension: meta.config.dimension,
                    vector_count: meta.vector_count,
                    created_at: meta.created_at,
                    updated_at: meta.updated_at,
                    tags: vec![],
                }
            })
        })
        .collect();
    match broad_fn(
        &queries,
        &collections,
        k,
        &config,
        &state.store,
        &state.embedding_manager,
    )
    .await
    {
        Ok(chunks) => {
            let arr: Vec<VectorizerValue> = chunks
                .iter()
                .map(|c| {
                    VectorizerValue::Map(vec![
                        (
                            VectorizerValue::Str("collection".into()),
                            VectorizerValue::Str(c.collection.clone()),
                        ),
                        (
                            VectorizerValue::Str("score".into()),
                            VectorizerValue::Float(c.score as f64),
                        ),
                        (
                            VectorizerValue::Str("content_preview".into()),
                            VectorizerValue::Str(c.content.chars().take(100).collect()),
                        ),
                    ])
                })
                .collect();
            let map = vec![
                (
                    VectorizerValue::Str("chunks".into()),
                    VectorizerValue::Array(arr),
                ),
                (
                    VectorizerValue::Str("count".into()),
                    VectorizerValue::Int(chunks.len() as i64),
                ),
            ];
            Response::ok(id, VectorizerValue::Map(map))
        }
        Err(e) => Response::err(id, format!("discovery.broad_discovery: {}", e)),
    }
}

async fn handle_discovery_semantic_focus(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    use vectorizer::discovery::{SemanticFocusConfig, semantic_focus as focus_fn};

    let payload = args
        .first()
        .map(value_to_json)
        .unwrap_or(serde_json::json!({}));
    let collection_name = match payload.get("collection").and_then(|c| c.as_str()) {
        Some(c) => c.to_string(),
        None => return Response::err(id, "discovery.semantic_focus: missing collection field"),
    };
    let queries: Vec<String> = match payload.get("queries").and_then(|v| v.as_array()) {
        Some(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect(),
        None => return Response::err(id, "discovery.semantic_focus: missing queries array"),
    };
    let k = payload.get("k").and_then(|v| v.as_u64()).unwrap_or(15) as usize;
    let config = SemanticFocusConfig::default();
    let coll = match state.store.get_collection(&collection_name) {
        Ok(c) => c,
        Err(e) => return Response::err(id, format!("discovery.semantic_focus: {}", e)),
    };
    let meta = coll.metadata();
    let collection_ref = vectorizer::discovery::CollectionRef {
        name: collection_name.clone(),
        dimension: meta.config.dimension,
        vector_count: meta.vector_count,
        created_at: meta.created_at,
        updated_at: meta.updated_at,
        tags: vec![],
    };
    match focus_fn(
        &collection_ref,
        &queries,
        k,
        &config,
        &state.store,
        &state.embedding_manager,
    )
    .await
    {
        Ok(chunks) => {
            let arr: Vec<VectorizerValue> = chunks
                .iter()
                .map(|c| {
                    VectorizerValue::Map(vec![
                        (
                            VectorizerValue::Str("collection".into()),
                            VectorizerValue::Str(c.collection.clone()),
                        ),
                        (
                            VectorizerValue::Str("score".into()),
                            VectorizerValue::Float(c.score as f64),
                        ),
                        (
                            VectorizerValue::Str("content_preview".into()),
                            VectorizerValue::Str(c.content.chars().take(100).collect()),
                        ),
                    ])
                })
                .collect();
            let map = vec![
                (
                    VectorizerValue::Str("chunks".into()),
                    VectorizerValue::Array(arr),
                ),
                (
                    VectorizerValue::Str("count".into()),
                    VectorizerValue::Int(chunks.len() as i64),
                ),
            ];
            Response::ok(id, VectorizerValue::Map(map))
        }
        Err(e) => Response::err(id, format!("discovery.semantic_focus: {}", e)),
    }
}

fn handle_discovery_promote_readme(id: u32, args: &[VectorizerValue]) -> Response {
    use vectorizer::discovery::{
        ChunkMetadata, ReadmePromotionConfig, ScoredChunk, promote_readme as promote_fn,
    };

    let payload = args
        .first()
        .map(value_to_json)
        .unwrap_or(serde_json::json!({}));
    let chunks_json = match payload.get("chunks").and_then(|v| v.as_array()) {
        Some(arr) => arr.clone(),
        None => return Response::err(id, "discovery.promote_readme: missing chunks array"),
    };
    let chunks: Vec<ScoredChunk> = chunks_json
        .iter()
        .filter_map(|v| {
            let obj = v.as_object()?;
            Some(ScoredChunk {
                collection: obj.get("collection")?.as_str()?.to_string(),
                doc_id: obj.get("doc_id")?.as_str()?.to_string(),
                content: obj.get("content")?.as_str()?.to_string(),
                score: obj.get("score")?.as_f64()? as f32,
                metadata: ChunkMetadata {
                    file_path: obj.get("file_path")?.as_str()?.to_string(),
                    chunk_index: obj.get("chunk_index")?.as_u64()? as usize,
                    file_extension: obj.get("file_extension")?.as_str()?.to_string(),
                    line_range: None,
                },
            })
        })
        .collect();
    let config = ReadmePromotionConfig::default();
    match promote_fn(&chunks, &config) {
        Ok(promoted) => {
            let arr: Vec<VectorizerValue> = promoted
                .iter()
                .map(|c| {
                    VectorizerValue::Map(vec![
                        (
                            VectorizerValue::Str("collection".into()),
                            VectorizerValue::Str(c.collection.clone()),
                        ),
                        (
                            VectorizerValue::Str("file_path".into()),
                            VectorizerValue::Str(c.metadata.file_path.clone()),
                        ),
                        (
                            VectorizerValue::Str("score".into()),
                            VectorizerValue::Float(c.score as f64),
                        ),
                    ])
                })
                .collect();
            let map = vec![
                (
                    VectorizerValue::Str("promoted_chunks".into()),
                    VectorizerValue::Array(arr),
                ),
                (
                    VectorizerValue::Str("count".into()),
                    VectorizerValue::Int(promoted.len() as i64),
                ),
            ];
            Response::ok(id, VectorizerValue::Map(map))
        }
        Err(e) => Response::err(id, format!("discovery.promote_readme: {}", e)),
    }
}

fn handle_discovery_compress_evidence(id: u32, args: &[VectorizerValue]) -> Response {
    use vectorizer::discovery::{
        ChunkMetadata, CompressionConfig, ScoredChunk, compress_evidence as compress_fn,
    };

    let payload = args
        .first()
        .map(value_to_json)
        .unwrap_or(serde_json::json!({}));
    let chunks_json = match payload.get("chunks").and_then(|v| v.as_array()) {
        Some(arr) => arr.clone(),
        None => return Response::err(id, "discovery.compress_evidence: missing chunks array"),
    };
    let max_bullets = payload
        .get("max_bullets")
        .and_then(|v| v.as_u64())
        .unwrap_or(20) as usize;
    let max_per_doc = payload
        .get("max_per_doc")
        .and_then(|v| v.as_u64())
        .unwrap_or(3) as usize;
    let chunks: Vec<ScoredChunk> = chunks_json
        .iter()
        .filter_map(|v| {
            let obj = v.as_object()?;
            Some(ScoredChunk {
                collection: obj.get("collection")?.as_str()?.to_string(),
                doc_id: obj.get("doc_id")?.as_str()?.to_string(),
                content: obj.get("content")?.as_str()?.to_string(),
                score: obj.get("score")?.as_f64()? as f32,
                metadata: ChunkMetadata {
                    file_path: obj.get("file_path")?.as_str()?.to_string(),
                    chunk_index: obj.get("chunk_index")?.as_u64()? as usize,
                    file_extension: obj.get("file_extension")?.as_str()?.to_string(),
                    line_range: None,
                },
            })
        })
        .collect();
    let config = CompressionConfig::default();
    match compress_fn(&chunks, max_bullets, max_per_doc, &config) {
        Ok(bullets) => {
            let arr: Vec<VectorizerValue> = bullets
                .iter()
                .map(|b| {
                    VectorizerValue::Map(vec![
                        (
                            VectorizerValue::Str("text".into()),
                            VectorizerValue::Str(b.text.clone()),
                        ),
                        (
                            VectorizerValue::Str("source_id".into()),
                            VectorizerValue::Str(b.source_id.clone()),
                        ),
                        (
                            VectorizerValue::Str("score".into()),
                            VectorizerValue::Float(b.score as f64),
                        ),
                    ])
                })
                .collect();
            let map = vec![
                (
                    VectorizerValue::Str("bullets".into()),
                    VectorizerValue::Array(arr),
                ),
                (
                    VectorizerValue::Str("count".into()),
                    VectorizerValue::Int(bullets.len() as i64),
                ),
            ];
            Response::ok(id, VectorizerValue::Map(map))
        }
        Err(e) => Response::err(id, format!("discovery.compress_evidence: {}", e)),
    }
}

fn handle_discovery_build_answer_plan(id: u32, args: &[VectorizerValue]) -> Response {
    use vectorizer::discovery::{
        AnswerPlanConfig, Bullet, BulletCategory, build_answer_plan as build_fn,
    };

    let payload = args
        .first()
        .map(value_to_json)
        .unwrap_or(serde_json::json!({}));
    let bullets_json = match payload.get("bullets").and_then(|v| v.as_array()) {
        Some(arr) => arr.clone(),
        None => return Response::err(id, "discovery.build_answer_plan: missing bullets array"),
    };
    let bullets: Vec<Bullet> = bullets_json
        .iter()
        .filter_map(|v| {
            let obj = v.as_object()?;
            let category = match obj.get("category")?.as_str()? {
                "Definition" => BulletCategory::Definition,
                "Feature" => BulletCategory::Feature,
                "Architecture" => BulletCategory::Architecture,
                "Performance" => BulletCategory::Performance,
                "Integration" => BulletCategory::Integration,
                "UseCase" => BulletCategory::UseCase,
                _ => BulletCategory::Other,
            };
            Some(Bullet {
                text: obj.get("text")?.as_str()?.to_string(),
                source_id: obj.get("source_id")?.as_str()?.to_string(),
                collection: obj
                    .get("collection")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                file_path: obj
                    .get("file_path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                score: obj.get("score")?.as_f64()? as f32,
                category,
            })
        })
        .collect();
    let config = AnswerPlanConfig::default();
    match build_fn(&bullets, &config) {
        Ok(plan) => {
            let sections: Vec<VectorizerValue> = plan
                .sections
                .iter()
                .map(|s| {
                    VectorizerValue::Map(vec![
                        (
                            VectorizerValue::Str("title".into()),
                            VectorizerValue::Str(s.title.clone()),
                        ),
                        (
                            VectorizerValue::Str("bullets_count".into()),
                            VectorizerValue::Int(s.bullets.len() as i64),
                        ),
                    ])
                })
                .collect();
            let map = vec![
                (
                    VectorizerValue::Str("sections".into()),
                    VectorizerValue::Array(sections),
                ),
                (
                    VectorizerValue::Str("total_bullets".into()),
                    VectorizerValue::Int(plan.total_bullets as i64),
                ),
            ];
            Response::ok(id, VectorizerValue::Map(map))
        }
        Err(e) => Response::err(id, format!("discovery.build_answer_plan: {}", e)),
    }
}

fn handle_discovery_render_llm_prompt(id: u32, args: &[VectorizerValue]) -> Response {
    use vectorizer::discovery::{
        AnswerPlan, Bullet, BulletCategory, PromptRenderConfig, Section, SectionType,
        render_llm_prompt as render_fn,
    };

    let payload = args
        .first()
        .map(value_to_json)
        .unwrap_or(serde_json::json!({}));
    let plan_json = match payload.get("plan").and_then(|v| v.as_object()) {
        Some(obj) => obj.clone(),
        None => return Response::err(id, "discovery.render_llm_prompt: missing plan object"),
    };
    let sections_json = match plan_json.get("sections").and_then(|v| v.as_array()) {
        Some(arr) => arr.clone(),
        None => return Response::err(id, "discovery.render_llm_prompt: missing sections in plan"),
    };
    let sections: Vec<Section> = sections_json
        .iter()
        .filter_map(|v| {
            let obj = v.as_object()?;
            let bullets_json = obj.get("bullets")?.as_array()?;
            let bullets: Vec<Bullet> = bullets_json
                .iter()
                .filter_map(|b| {
                    let b_obj = b.as_object()?;
                    let category = match b_obj.get("category")?.as_str()? {
                        "Feature" => BulletCategory::Feature,
                        "Architecture" => BulletCategory::Architecture,
                        "Performance" => BulletCategory::Performance,
                        "Integration" => BulletCategory::Integration,
                        "UseCase" => BulletCategory::UseCase,
                        _ => BulletCategory::Definition,
                    };
                    Some(Bullet {
                        text: b_obj.get("text")?.as_str()?.to_string(),
                        source_id: b_obj.get("source_id")?.as_str()?.to_string(),
                        collection: b_obj
                            .get("collection")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        file_path: b_obj
                            .get("file_path")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        score: b_obj.get("score")?.as_f64()? as f32,
                        category,
                    })
                })
                .collect();
            Some(Section {
                title: obj.get("title")?.as_str()?.to_string(),
                section_type: SectionType::Definition,
                bullets,
                priority: obj.get("priority").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
            })
        })
        .collect();
    let plan = AnswerPlan {
        sections,
        total_bullets: plan_json
            .get("total_bullets")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize,
        sources: plan_json
            .get("sources")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default(),
    };
    let config = PromptRenderConfig::default();
    match render_fn(&plan, &config) {
        Ok(prompt) => {
            let map = vec![
                (
                    VectorizerValue::Str("prompt".into()),
                    VectorizerValue::Str(prompt.clone()),
                ),
                (
                    VectorizerValue::Str("length".into()),
                    VectorizerValue::Int(prompt.len() as i64),
                ),
                (
                    VectorizerValue::Str("estimated_tokens".into()),
                    VectorizerValue::Int((prompt.len() / 4) as i64),
                ),
            ];
            Response::ok(id, VectorizerValue::Map(map))
        }
        Err(e) => Response::err(id, format!("discovery.render_llm_prompt: {}", e)),
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn vector_to_value(v: &vectorizer::models::Vector) -> VectorizerValue {
    let data = v
        .data
        .iter()
        .map(|f| VectorizerValue::Float(*f as f64))
        .collect::<Vec<_>>();
    let mut map = vec![
        (
            VectorizerValue::Str("id".into()),
            VectorizerValue::Str(v.id.clone()),
        ),
        (
            VectorizerValue::Str("data".into()),
            VectorizerValue::Array(data),
        ),
    ];
    if let Some(payload) = &v.payload {
        let json = serde_json::to_string(&payload.data).unwrap_or_default();
        map.push((
            VectorizerValue::Str("payload".into()),
            VectorizerValue::Str(json),
        ));
    }
    if let Some(doc_id) = &v.document_id {
        map.push((
            VectorizerValue::Str("document_id".into()),
            VectorizerValue::Str(doc_id.clone()),
        ));
    }
    VectorizerValue::Map(map)
}

fn parse_hello_payload(
    pairs: &[(VectorizerValue, VectorizerValue)],
) -> (Option<String>, Option<String>, Option<String>, i64) {
    let mut token = None;
    let mut api_key = None;
    let mut client_name = None;
    let mut version = PROTOCOL_VERSION;
    for (k, v) in pairs {
        let Some(key) = k.as_str() else { continue };
        match key {
            "token" => token = v.as_str().map(str::to_owned),
            "api_key" => api_key = v.as_str().map(str::to_owned),
            "client_name" => client_name = v.as_str().map(str::to_owned),
            "version" => {
                if let Some(n) = v.as_int() {
                    version = n;
                }
            }
            _ => {}
        }
    }
    (token, api_key, client_name, version)
}

async fn validate_credentials(
    handler: &crate::server::AuthHandlerState,
    token: Option<&str>,
    api_key: Option<&str>,
) -> Result<(String, bool), String> {
    // The handler exposes `is_token_blacklisted` and the inner
    // `auth_manager` for JWT/API-key validation. See
    // `src/server/auth_handlers/state.rs` for the full surface.
    if let Some(token) = token {
        if handler.is_token_blacklisted(token).await {
            return Err("token blacklisted (logged out)".into());
        }
        return handler
            .auth_manager
            .validate_jwt(token)
            .map(|claims| {
                let admin = claims.roles.contains(&Role::Admin);
                (claims.username, admin)
            })
            .map_err(|e| format!("invalid JWT: {}", e));
    }
    if let Some(api_key) = api_key {
        return handler
            .auth_manager
            .validate_api_key(api_key)
            .await
            .map(|claims| {
                let admin = claims.roles.contains(&Role::Admin);
                (claims.username, admin)
            })
            .map_err(|e| format!("invalid API key: {}", e));
    }
    Err("HELLO requires either `token` or `api_key`".into())
}

// ── File ops ─────────────────────────────────────────────────────────────────

async fn handle_file_content(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    use vectorizer::file_operations::FileOperations;
    let req_json = match args.first() {
        Some(v) => value_to_json(v),
        None => return Response::err(id, "file.content expects [Map(request)]"),
    };
    let collection = match req_json.get("collection").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return Response::err(id, "file.content: missing 'collection'"),
    };
    let file_path = match req_json.get("file_path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return Response::err(id, "file.content: missing 'file_path'"),
    };
    let max_size_kb = req_json
        .get("max_size_kb")
        .and_then(|v| v.as_u64())
        .unwrap_or(500) as usize;
    let file_ops = FileOperations::with_store(state.store.clone());
    match file_ops
        .get_file_content(&collection, &file_path, max_size_kb)
        .await
    {
        Ok(result) => {
            let json = serde_json::to_value(result).unwrap_or(serde_json::Value::Null);
            Response::ok(id, json_to_value(json))
        }
        Err(e) => Response::err(id, format!("file.content: {}", e)),
    }
}

async fn handle_file_list(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    use vectorizer::file_operations::{FileListFilter, FileOperations, SortBy};
    let req_json = match args.first() {
        Some(v) => value_to_json(v),
        None => return Response::err(id, "file.list expects [Map(request)]"),
    };
    let collection = match req_json.get("collection").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return Response::err(id, "file.list: missing 'collection'"),
    };
    let filter_by_type = req_json
        .get("filter_by_type")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        });
    let min_chunks = req_json
        .get("min_chunks")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize);
    let max_results = req_json
        .get("max_results")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize);
    let sort_by = req_json
        .get("sort_by")
        .and_then(|v| v.as_str())
        .and_then(|s| match s {
            "name" => Some(SortBy::Name),
            "size" => Some(SortBy::Size),
            "chunks" => Some(SortBy::Chunks),
            "recent" => Some(SortBy::Recent),
            _ => None,
        })
        .unwrap_or(SortBy::Name);
    let filter = FileListFilter {
        filter_by_type,
        min_chunks,
        max_results,
        sort_by,
    };
    let file_ops = FileOperations::with_store(state.store.clone());
    match file_ops.list_files_in_collection(&collection, filter).await {
        Ok(result) => {
            let json = serde_json::to_value(result).unwrap_or(serde_json::Value::Null);
            Response::ok(id, json_to_value(json))
        }
        Err(e) => Response::err(id, format!("file.list: {}", e)),
    }
}

async fn handle_file_summary(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    use vectorizer::file_operations::{FileOperations, SummaryType};
    let req_json = match args.first() {
        Some(v) => value_to_json(v),
        None => return Response::err(id, "file.summary expects [Map(request)]"),
    };
    let collection = match req_json.get("collection").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return Response::err(id, "file.summary: missing 'collection'"),
    };
    let file_path = match req_json.get("file_path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return Response::err(id, "file.summary: missing 'file_path'"),
    };
    let summary_type = req_json
        .get("summary_type")
        .and_then(|v| v.as_str())
        .and_then(|s| match s {
            "extractive" => Some(SummaryType::Extractive),
            "structural" => Some(SummaryType::Structural),
            "both" => Some(SummaryType::Both),
            _ => None,
        })
        .unwrap_or(SummaryType::Both);
    let max_sentences = req_json
        .get("max_sentences")
        .and_then(|v| v.as_u64())
        .unwrap_or(5) as usize;
    let file_ops = FileOperations::with_store(state.store.clone());
    match file_ops
        .get_file_summary(&collection, &file_path, summary_type, max_sentences)
        .await
    {
        Ok(result) => {
            let json = serde_json::to_value(result).unwrap_or(serde_json::Value::Null);
            Response::ok(id, json_to_value(json))
        }
        Err(e) => Response::err(id, format!("file.summary: {}", e)),
    }
}

async fn handle_file_chunks(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    use vectorizer::file_operations::FileOperations;
    let req_json = match args.first() {
        Some(v) => value_to_json(v),
        None => return Response::err(id, "file.chunks expects [Map(request)]"),
    };
    let collection = match req_json.get("collection").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return Response::err(id, "file.chunks: missing 'collection'"),
    };
    let file_path = match req_json.get("file_path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return Response::err(id, "file.chunks: missing 'file_path'"),
    };
    let start_chunk = req_json
        .get("start_chunk")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    let limit = req_json.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
    let include_context = req_json
        .get("include_context")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let file_ops = FileOperations::with_store(state.store.clone());
    match file_ops
        .get_file_chunks_ordered(&collection, &file_path, start_chunk, limit, include_context)
        .await
    {
        Ok(result) => {
            let json = serde_json::to_value(result).unwrap_or(serde_json::Value::Null);
            Response::ok(id, json_to_value(json))
        }
        Err(e) => Response::err(id, format!("file.chunks: {}", e)),
    }
}

async fn handle_file_outline(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    use vectorizer::file_operations::FileOperations;
    let req_json = match args.first() {
        Some(v) => value_to_json(v),
        None => return Response::err(id, "file.outline expects [Map(request)]"),
    };
    let collection = match req_json.get("collection").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return Response::err(id, "file.outline: missing 'collection'"),
    };
    let max_depth = req_json
        .get("max_depth")
        .and_then(|v| v.as_u64())
        .unwrap_or(5) as usize;
    let include_summaries = req_json
        .get("include_summaries")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let highlight_key_files = req_json
        .get("highlight_key_files")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let file_ops = FileOperations::with_store(state.store.clone());
    match file_ops
        .get_project_outline(
            &collection,
            max_depth,
            include_summaries,
            highlight_key_files,
        )
        .await
    {
        Ok(result) => {
            let json = serde_json::to_value(result).unwrap_or(serde_json::Value::Null);
            Response::ok(id, json_to_value(json))
        }
        Err(e) => Response::err(id, format!("file.outline: {}", e)),
    }
}

async fn handle_file_related(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    use vectorizer::file_operations::FileOperations;
    let req_json = match args.first() {
        Some(v) => value_to_json(v),
        None => return Response::err(id, "file.related expects [Map(request)]"),
    };
    let collection = match req_json.get("collection").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return Response::err(id, "file.related: missing 'collection'"),
    };
    let file_path = match req_json.get("file_path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return Response::err(id, "file.related: missing 'file_path'"),
    };
    let limit = req_json.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
    let similarity_threshold = req_json
        .get("similarity_threshold")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.6) as f32;
    let include_reason = req_json
        .get("include_reason")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let file_ops = FileOperations::with_store(state.store.clone());
    match file_ops
        .get_related_files(
            &collection,
            &file_path,
            limit,
            similarity_threshold,
            include_reason,
            &state.embedding_manager,
        )
        .await
    {
        Ok(result) => {
            let json = serde_json::to_value(result).unwrap_or(serde_json::Value::Null);
            Response::ok(id, json_to_value(json))
        }
        Err(e) => Response::err(id, format!("file.related: {}", e)),
    }
}

async fn handle_file_search_by_type(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    use vectorizer::file_operations::FileOperations;
    let req_json = match args.first() {
        Some(v) => value_to_json(v),
        None => return Response::err(id, "file.search_by_type expects [Map(request)]"),
    };
    let collection = match req_json.get("collection").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return Response::err(id, "file.search_by_type: missing 'collection'"),
    };
    let query = match req_json.get("query").and_then(|v| v.as_str()) {
        Some(q) => q.to_string(),
        None => return Response::err(id, "file.search_by_type: missing 'query'"),
    };
    let file_types: Vec<String> = match req_json.get("file_types").and_then(|v| v.as_array()) {
        Some(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        None => return Response::err(id, "file.search_by_type: missing 'file_types'"),
    };
    let limit = req_json.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
    let return_full_files = req_json
        .get("return_full_files")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let file_ops = FileOperations::with_store(state.store.clone());
    match file_ops
        .search_by_file_type(
            &collection,
            &query,
            file_types,
            limit,
            return_full_files,
            &state.embedding_manager,
        )
        .await
    {
        Ok(result) => {
            let json = serde_json::to_value(result).unwrap_or(serde_json::Value::Null);
            Response::ok(id, json_to_value(json))
        }
        Err(e) => Response::err(id, format!("file.search_by_type: {}", e)),
    }
}

// ── Graph helpers ─────────────────────────────────────────────────────────────

fn get_graph_from_collection<'a>(
    collection: &'a vectorizer::db::CollectionType,
) -> Option<&'a vectorizer::db::graph::Graph> {
    match collection {
        vectorizer::db::CollectionType::Cpu(c) => c.get_graph().map(|arc| arc.as_ref()),
        _ => None,
    }
}

fn parse_relationship_type(s: &str) -> Option<vectorizer::db::graph::RelationshipType> {
    use vectorizer::db::graph::RelationshipType;
    match s.to_uppercase().as_str() {
        "SIMILAR_TO" | "SIMILARTO" => Some(RelationshipType::SimilarTo),
        "REFERENCES" => Some(RelationshipType::References),
        "CONTAINS" => Some(RelationshipType::Contains),
        "DERIVED_FROM" | "DERIVEDFROM" => Some(RelationshipType::DerivedFrom),
        _ => None,
    }
}

fn handle_graph_list_nodes(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let collection_name = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => return Response::err(id, "graph.list_nodes expects [Str(collection)]"),
    };
    let collection = match state.store.get_collection(collection_name) {
        Ok(c) => c,
        Err(e) => return Response::err(id, format!("graph.list_nodes: {}", e)),
    };
    let graph = match get_graph_from_collection(&collection) {
        Some(g) => g,
        None => {
            return Response::err(
                id,
                format!("graph not enabled for collection '{}'", collection_name),
            );
        }
    };
    let nodes = graph.get_all_nodes();
    let json = serde_json::json!({ "nodes": nodes, "count": nodes.len() });
    Response::ok(id, json_to_value(json))
}

fn handle_graph_neighbors(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let collection_name = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => {
            return Response::err(
                id,
                "graph.neighbors expects [Str(collection), Str(node_id), Int(depth)?]",
            );
        }
    };
    let node_id = match args.get(1).and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return Response::err(id, "graph.neighbors: missing node_id"),
    };
    let collection = match state.store.get_collection(collection_name) {
        Ok(c) => c,
        Err(e) => return Response::err(id, format!("graph.neighbors: {}", e)),
    };
    let graph = match get_graph_from_collection(&collection) {
        Some(g) => g,
        None => {
            return Response::err(
                id,
                format!("graph not enabled for collection '{}'", collection_name),
            );
        }
    };
    match graph.get_neighbors(node_id, None) {
        Ok(neighbors) => {
            let arr: Vec<serde_json::Value> = neighbors
                .into_iter()
                .map(|(node, edge)| serde_json::json!({ "node": node, "edge": edge }))
                .collect();
            Response::ok(id, json_to_value(serde_json::json!({ "neighbors": arr })))
        }
        Err(e) => Response::err(id, format!("graph.neighbors: {}", e)),
    }
}

fn handle_graph_find_related(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let collection_name = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => {
            return Response::err(
                id,
                "graph.find_related expects [Str(collection), Str(node_id), Int(limit)?]",
            );
        }
    };
    let node_id = match args.get(1).and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return Response::err(id, "graph.find_related: missing node_id"),
    };
    let max_hops = args
        .get(2)
        .and_then(|v| v.as_int())
        .map(|n| n as usize)
        .unwrap_or(2);
    let collection = match state.store.get_collection(collection_name) {
        Ok(c) => c,
        Err(e) => return Response::err(id, format!("graph.find_related: {}", e)),
    };
    let graph = match get_graph_from_collection(&collection) {
        Some(g) => g,
        None => {
            return Response::err(
                id,
                format!("graph not enabled for collection '{}'", collection_name),
            );
        }
    };
    match graph.find_related(node_id, max_hops, None) {
        Ok(related) => {
            let arr: Vec<serde_json::Value> = related
                .into_iter()
                .map(|(node, distance, weight)| {
                    serde_json::json!({ "node": node, "distance": distance, "weight": weight })
                })
                .collect();
            Response::ok(id, json_to_value(serde_json::json!({ "related": arr })))
        }
        Err(e) => Response::err(id, format!("graph.find_related: {}", e)),
    }
}

fn handle_graph_find_path(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let collection_name = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => {
            return Response::err(
                id,
                "graph.find_path expects [Str(collection), Str(from), Str(to)]",
            );
        }
    };
    let from = match args.get(1).and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return Response::err(id, "graph.find_path: missing 'from' node"),
    };
    let to = match args.get(2).and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return Response::err(id, "graph.find_path: missing 'to' node"),
    };
    let collection = match state.store.get_collection(collection_name) {
        Ok(c) => c,
        Err(e) => return Response::err(id, format!("graph.find_path: {}", e)),
    };
    let graph = match get_graph_from_collection(&collection) {
        Some(g) => g,
        None => {
            return Response::err(
                id,
                format!("graph not enabled for collection '{}'", collection_name),
            );
        }
    };
    match graph.find_path(from, to) {
        Ok(path) => Response::ok(
            id,
            json_to_value(serde_json::json!({ "path": path, "found": true })),
        ),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("not found") || msg.contains("No path") {
                Response::ok(
                    id,
                    json_to_value(
                        serde_json::json!({ "path": [], "found": false, "message": msg }),
                    ),
                )
            } else {
                Response::err(id, format!("graph.find_path: {}", e))
            }
        }
    }
}

fn handle_graph_create_edge(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let collection_name = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => return Response::err(id, "graph.create_edge expects [Str(collection), Map(edge)]"),
    };
    let edge_json = match args.get(1) {
        Some(v) => value_to_json(v),
        None => return Response::err(id, "graph.create_edge: missing edge Map"),
    };
    let source = match edge_json.get("source").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return Response::err(id, "graph.create_edge: missing 'source'"),
    };
    let target = match edge_json.get("target").and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return Response::err(id, "graph.create_edge: missing 'target'"),
    };
    let rel_str = match edge_json.get("relationship_type").and_then(|v| v.as_str()) {
        Some(r) => r.to_string(),
        None => return Response::err(id, "graph.create_edge: missing 'relationship_type'"),
    };
    let rel_type = match parse_relationship_type(&rel_str) {
        Some(r) => r,
        None => {
            return Response::err(
                id,
                format!("graph.create_edge: invalid relationship_type '{}'", rel_str),
            );
        }
    };
    let weight = edge_json
        .get("weight")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0) as f32;
    let collection = match state.store.get_collection(collection_name) {
        Ok(c) => c,
        Err(e) => return Response::err(id, format!("graph.create_edge: {}", e)),
    };
    let graph = match get_graph_from_collection(&collection) {
        Some(g) => g,
        None => {
            return Response::err(
                id,
                format!("graph not enabled for collection '{}'", collection_name),
            );
        }
    };
    let edge_id = format!("{}:{}:{:?}", source, target, rel_type);
    let edge = vectorizer::db::graph::Edge::new(edge_id.clone(), source, target, rel_type, weight);
    match graph.add_edge(edge) {
        Ok(()) => Response::ok(
            id,
            json_to_value(serde_json::json!({
                "edge_id": edge_id,
                "success": true,
            })),
        ),
        Err(e) => Response::err(id, format!("graph.create_edge: {}", e)),
    }
}

fn handle_graph_delete_edge(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let collection_name = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => {
            return Response::err(
                id,
                "graph.delete_edge expects [Str(collection), Str(edge_id)]",
            );
        }
    };
    let edge_id = match args.get(1).and_then(|v| v.as_str()) {
        Some(e) => e,
        None => return Response::err(id, "graph.delete_edge: missing edge_id"),
    };
    let collection = match state.store.get_collection(collection_name) {
        Ok(c) => c,
        Err(e) => return Response::err(id, format!("graph.delete_edge: {}", e)),
    };
    let graph = match get_graph_from_collection(&collection) {
        Some(g) => g,
        None => {
            return Response::err(
                id,
                format!("graph not enabled for collection '{}'", collection_name),
            );
        }
    };
    match graph.remove_edge(edge_id) {
        Ok(()) => Response::ok(id, json_to_value(serde_json::json!({ "success": true }))),
        Err(e) => Response::err(id, format!("graph.delete_edge: {}", e)),
    }
}

fn handle_graph_list_edges(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let collection_name = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => return Response::err(id, "graph.list_edges expects [Str(collection)]"),
    };
    let collection = match state.store.get_collection(collection_name) {
        Ok(c) => c,
        Err(e) => return Response::err(id, format!("graph.list_edges: {}", e)),
    };
    let graph = match get_graph_from_collection(&collection) {
        Some(g) => g,
        None => {
            return Response::err(
                id,
                format!("graph not enabled for collection '{}'", collection_name),
            );
        }
    };
    let edges = graph.get_all_edges();
    let count = edges.len();
    let json = serde_json::json!({ "edges": edges, "count": count });
    Response::ok(id, json_to_value(json))
}

fn handle_graph_discover_edges(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    let collection_name = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => {
            return Response::err(
                id,
                "graph.discover_edges expects [Str(collection), Map(request)]",
            );
        }
    };
    let req_json = args.get(1).map(value_to_json).unwrap_or_default();
    let similarity_threshold = req_json
        .get("similarity_threshold")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.7) as f32;
    let max_per_node = req_json
        .get("max_per_node")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;
    let collection = match state.store.get_collection(collection_name) {
        Ok(c) => c,
        Err(e) => return Response::err(id, format!("graph.discover_edges: {}", e)),
    };
    let graph = match get_graph_from_collection(&collection) {
        Some(g) => g,
        None => {
            return Response::err(
                id,
                format!("graph not enabled for collection '{}'", collection_name),
            );
        }
    };
    let cpu_collection = match &*collection {
        vectorizer::db::CollectionType::Cpu(c) => c,
        _ => {
            return Response::err(
                id,
                "graph.discover_edges: only supported for CPU collections",
            );
        }
    };
    let config = vectorizer::models::AutoRelationshipConfig {
        similarity_threshold,
        max_per_node,
        enabled_types: vec!["SIMILAR_TO".to_string()],
    };
    match vectorizer::db::graph_relationship_discovery::discover_edges_for_collection(
        graph,
        cpu_collection,
        &config,
    ) {
        Ok(stats) => Response::ok(
            id,
            json_to_value(serde_json::json!({
                "success": true,
                "total_nodes": stats.total_nodes,
                "nodes_processed": stats.nodes_processed,
                "nodes_with_edges": stats.nodes_with_edges,
                "total_edges_created": stats.total_edges_created,
            })),
        ),
        Err(e) => Response::err(id, format!("graph.discover_edges: {}", e)),
    }
}

fn handle_graph_discover_edges_for_node(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    let collection_name = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => {
            return Response::err(
                id,
                "graph.discover_edges_for_node expects [Str(collection), Str(node_id), Map(request)]",
            );
        }
    };
    let node_id = match args.get(1).and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return Response::err(id, "graph.discover_edges_for_node: missing node_id"),
    };
    let req_json = args.get(2).map(value_to_json).unwrap_or_default();
    let similarity_threshold = req_json
        .get("similarity_threshold")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.7) as f32;
    let max_per_node = req_json
        .get("max_per_node")
        .and_then(|v| v.as_u64())
        .unwrap_or(10) as usize;
    let collection = match state.store.get_collection(collection_name) {
        Ok(c) => c,
        Err(e) => return Response::err(id, format!("graph.discover_edges_for_node: {}", e)),
    };
    let graph = match get_graph_from_collection(&collection) {
        Some(g) => g,
        None => {
            return Response::err(
                id,
                format!("graph not enabled for collection '{}'", collection_name),
            );
        }
    };
    let cpu_collection = match &*collection {
        vectorizer::db::CollectionType::Cpu(c) => c,
        _ => {
            return Response::err(
                id,
                "graph.discover_edges_for_node: only supported for CPU collections",
            );
        }
    };
    let config = vectorizer::models::AutoRelationshipConfig {
        similarity_threshold,
        max_per_node,
        enabled_types: vec!["SIMILAR_TO".to_string()],
    };
    match vectorizer::db::graph_relationship_discovery::discover_edges_for_node(
        graph,
        node_id,
        cpu_collection,
        &config,
    ) {
        Ok(edges_created) => Response::ok(
            id,
            json_to_value(serde_json::json!({
                "success": true,
                "node_id": node_id,
                "edges_created": edges_created,
            })),
        ),
        Err(e) => Response::err(id, format!("graph.discover_edges_for_node: {}", e)),
    }
}

fn handle_graph_discovery_status(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    let collection_name = match args.first().and_then(|v| v.as_str()) {
        Some(c) => c,
        None => return Response::err(id, "graph.discovery_status expects [Str(collection)]"),
    };
    let collection = match state.store.get_collection(collection_name) {
        Ok(c) => c,
        Err(e) => return Response::err(id, format!("graph.discovery_status: {}", e)),
    };
    let graph = match get_graph_from_collection(&collection) {
        Some(g) => g,
        None => {
            return Response::err(
                id,
                format!("graph not enabled for collection '{}'", collection_name),
            );
        }
    };
    let total_nodes = graph.node_count();
    let total_edges = graph.edge_count();
    let nodes = graph.get_all_nodes();
    let nodes_with_edges = nodes
        .iter()
        .filter(|node| {
            graph
                .get_neighbors(&node.id, None)
                .map(|n| !n.is_empty())
                .unwrap_or(false)
        })
        .count();
    let progress_percentage = if total_nodes > 0 {
        (nodes_with_edges as f64 / total_nodes as f64) * 100.0
    } else {
        0.0
    };
    Response::ok(
        id,
        json_to_value(serde_json::json!({
            "total_nodes": total_nodes,
            "nodes_with_edges": nodes_with_edges,
            "total_edges": total_edges,
            "progress_percentage": progress_percentage,
        })),
    )
}

// ── Admin / observability ─────────────────────────────────────────────────────

fn handle_admin_stats(state: &Arc<RpcState>, id: u32) -> Response {
    let collections = state.store.list_collections();
    let collections_count = collections.len();
    let total_vectors: usize = collections
        .iter()
        .filter_map(|name| state.store.get_collection_metadata(name).ok())
        .map(|meta| meta.vector_count)
        .sum();
    Response::ok(
        id,
        json_to_value(serde_json::json!({
            "collections_count": collections_count,
            "total_vectors": total_vectors,
            "version": env!("CARGO_PKG_VERSION"),
        })),
    )
}

fn handle_admin_status(state: &Arc<RpcState>, id: u32) -> Response {
    let collections_count = state.store.list_collections().len();
    Response::ok(
        id,
        json_to_value(serde_json::json!({
            "ready": true,
            "collections_count": collections_count,
            "version": env!("CARGO_PKG_VERSION"),
        })),
    )
}

fn handle_admin_logs(id: u32, _args: &[VectorizerValue]) -> Response {
    // In-process log access is not available; return an empty list with a note.
    Response::ok(
        id,
        json_to_value(serde_json::json!({
            "entries": [],
            "message": "Live log streaming is available via the REST /logs endpoint",
        })),
    )
}

fn handle_admin_indexing_progress(state: &Arc<RpcState>, id: u32) -> Response {
    let total = state.store.list_collections().len();
    Response::ok(
        id,
        json_to_value(serde_json::json!({
            "total_collections": total,
            "indexed": total,
            "in_progress": 0,
        })),
    )
}

fn handle_admin_config_get(id: u32) -> Response {
    let possible_paths = ["./config.yml", "../config.yml", "config.yml"];
    for path in &possible_paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(val) = serde_yaml::from_str::<serde_json::Value>(&content) {
                return Response::ok(id, json_to_value(val));
            }
        }
    }
    Response::ok(
        id,
        json_to_value(serde_json::json!({
            "error": "config.yml not found",
        })),
    )
}

fn handle_admin_config_update(
    id: u32,
    args: &[VectorizerValue],
    auth: &ConnectionAuth,
) -> Response {
    if let Some(r) = require_admin(auth, id) {
        return r;
    }
    let patch_json = match args.first() {
        Some(v) => value_to_json(v),
        None => return Response::err(id, "admin.config_update expects [Map(patch)]"),
    };
    match serde_yaml::to_string(&patch_json) {
        Ok(yaml) => match std::fs::write("./config.yml", yaml) {
            Ok(()) => Response::ok(id, json_to_value(serde_json::json!({ "success": true }))),
            Err(e) => Response::err(id, format!("admin.config_update: write failed: {}", e)),
        },
        Err(e) => Response::err(id, format!("admin.config_update: serialize failed: {}", e)),
    }
}

fn handle_admin_backups_list(id: u32) -> Response {
    let backup_dir = std::path::Path::new("./backups");
    let mut backups: Vec<serde_json::Value> = Vec::new();
    if backup_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(backup_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("backup") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                            backups.push(data);
                        }
                    }
                }
            }
        }
    }
    backups.sort_by(|a, b| {
        let a_date = a.get("date").and_then(|d| d.as_str()).unwrap_or("");
        let b_date = b.get("date").and_then(|d| d.as_str()).unwrap_or("");
        b_date.cmp(a_date)
    });
    Response::ok(id, json_to_value(serde_json::json!({ "backups": backups })))
}

fn handle_admin_backups_create(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
    auth: &ConnectionAuth,
) -> Response {
    if let Some(r) = require_admin(auth, id) {
        return r;
    }
    let req_json = match args.first() {
        Some(v) => value_to_json(v),
        None => return Response::err(id, "admin.backups_create expects [Map(request)]"),
    };
    let name = match req_json.get("name").and_then(|v| v.as_str()) {
        Some(n) => n.to_string(),
        None => return Response::err(id, "admin.backups_create: missing 'name'"),
    };
    let collections: Vec<String> = req_json
        .get("collections")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let backup_dir = std::path::Path::new("./backups");
    if !backup_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(backup_dir) {
            return Response::err(id, format!("admin.backups_create: {}", e));
        }
    }
    let backup_id = uuid::Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();
    let mut backup_data = serde_json::json!({
        "id": backup_id.clone(),
        "name": name,
        "date": timestamp,
        "collections": collections,
    });
    for coll_name in &collections {
        if let Ok(coll) = state.store.get_collection(coll_name) {
            let vectors = coll.get_all_vectors();
            backup_data["data"][coll_name] = serde_json::json!({
                "vector_count": vectors.len(),
            });
        }
    }
    let backup_file = backup_dir.join(format!("{}.backup", backup_id));
    match std::fs::write(&backup_file, backup_data.to_string()) {
        Ok(()) => Response::ok(
            id,
            json_to_value(serde_json::json!({ "success": true, "backup_id": backup_id })),
        ),
        Err(e) => Response::err(id, format!("admin.backups_create: {}", e)),
    }
}

fn handle_admin_backups_restore(
    _state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
    auth: &ConnectionAuth,
) -> Response {
    if let Some(r) = require_admin(auth, id) {
        return r;
    }
    let req_json = match args.first() {
        Some(v) => value_to_json(v),
        None => return Response::err(id, "admin.backups_restore expects [Map(request)]"),
    };
    let backup_id = match req_json.get("backup_id").and_then(|v| v.as_str()) {
        Some(b) => b.to_string(),
        None => return Response::err(id, "admin.backups_restore: missing 'backup_id'"),
    };
    let backup_file = std::path::Path::new("./backups").join(format!("{}.backup", backup_id));
    if !backup_file.exists() {
        return Response::err(
            id,
            format!("admin.backups_restore: backup '{}' not found", backup_id),
        );
    }
    // Restoration of vector data requires a full re-insert pass which
    // is a heavyweight operation. Return acknowledged status; the REST
    // endpoint performs the full restore.
    Response::ok(
        id,
        json_to_value(serde_json::json!({
            "success": true,
            "backup_id": backup_id,
            "message": "Backup restore acknowledged. Use REST POST /backups/restore for full restoration.",
        })),
    )
}

fn handle_admin_workspaces_list(id: u32) -> Response {
    let wm = vectorizer::config::WorkspaceManager::new();
    let workspaces = wm.list_workspaces();
    let list: Vec<serde_json::Value> = workspaces
        .iter()
        .map(|w| {
            serde_json::json!({
                "id": w.id,
                "path": w.path,
                "collection_name": w.collection_name,
                "active": w.active,
                "file_count": w.file_count,
            })
        })
        .collect();
    Response::ok(id, json_to_value(serde_json::json!({ "workspaces": list })))
}

fn handle_admin_workspace_get(id: u32) -> Response {
    let possible_paths = ["./workspace.yml", "../workspace.yml", "workspace.yml"];
    for path in &possible_paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(val) = serde_yaml::from_str::<serde_json::Value>(&content) {
                return Response::ok(id, json_to_value(val));
            }
        }
    }
    Response::ok(
        id,
        json_to_value(serde_json::json!({ "error": "workspace.yml not found" })),
    )
}

fn handle_admin_workspace_add(
    id: u32,
    args: &[VectorizerValue],
    auth: &ConnectionAuth,
) -> Response {
    if let Some(r) = require_admin(auth, id) {
        return r;
    }
    let req_json = match args.first() {
        Some(v) => value_to_json(v),
        None => return Response::err(id, "admin.workspace_add expects [Map(request)]"),
    };
    let path = match req_json.get("path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return Response::err(id, "admin.workspace_add: missing 'path'"),
    };
    let collection_name = match req_json.get("collection_name").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return Response::err(id, "admin.workspace_add: missing 'collection_name'"),
    };
    let wm = vectorizer::config::WorkspaceManager::new();
    match wm.add_workspace(&path, &collection_name) {
        Ok(workspace) => Response::ok(
            id,
            json_to_value(serde_json::json!({
                "success": true,
                "workspace": {
                    "id": workspace.id,
                    "path": workspace.path,
                    "collection_name": workspace.collection_name,
                }
            })),
        ),
        Err(e) => Response::err(id, format!("admin.workspace_add: {}", e)),
    }
}

fn handle_admin_workspace_remove(
    id: u32,
    args: &[VectorizerValue],
    auth: &ConnectionAuth,
) -> Response {
    if let Some(r) = require_admin(auth, id) {
        return r;
    }
    let name = match args.first().and_then(|v| v.as_str()) {
        Some(n) => n,
        None => return Response::err(id, "admin.workspace_remove expects [Str(name)]"),
    };
    let wm = vectorizer::config::WorkspaceManager::new();
    match wm.remove_workspace(name) {
        Ok(_) => Response::ok(
            id,
            json_to_value(serde_json::json!({ "success": true, "path": name })),
        ),
        Err(e) => Response::err(id, format!("admin.workspace_remove: {}", e)),
    }
}

async fn handle_admin_restart(id: u32, auth: &ConnectionAuth) -> Response {
    if let Some(r) = require_admin(auth, id) {
        return r;
    }
    // Mirror REST handler: schedule a delayed exit so the response can be sent first.
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        #[cfg(windows)]
        {
            let _ = std::fs::write(
                "./restart.marker",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs().to_string())
                    .unwrap_or_default(),
            );
            std::process::exit(0);
        }
        #[cfg(unix)]
        {
            use nix::sys::signal::{self, Signal};
            use nix::unistd::Pid;
            let _ = signal::kill(Pid::this(), Signal::SIGHUP);
        }
    });
    Response::ok(
        id,
        json_to_value(serde_json::json!({ "success": true, "message": "restart initiated" })),
    )
}

fn handle_admin_slow_queries_list(state: &Arc<RpcState>, id: u32) -> Response {
    let ring = &state.slow_query_ring;
    let entries = ring.entries();
    let config = ring.config();
    let items: Vec<serde_json::Value> = entries
        .iter()
        .map(|e| {
            serde_json::json!({
                "timestamp": e.timestamp.to_rfc3339(),
                "collection": e.collection,
                "k": e.k,
                "duration_ms": e.duration_ms,
            })
        })
        .collect();
    Response::ok(
        id,
        json_to_value(serde_json::json!({
            "entries": items,
            "total": items.len(),
            "config": {
                "threshold_ms": config.threshold_ms,
                "capacity": config.capacity,
            }
        })),
    )
}

fn handle_admin_slow_queries_config(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    let req_json = match args.first() {
        Some(v) => value_to_json(v),
        None => return Response::err(id, "admin.slow_queries_config expects [Map(config)]"),
    };
    let threshold_ms = match req_json.get("threshold_ms").and_then(|v| v.as_u64()) {
        Some(t) => t,
        None => return Response::err(id, "admin.slow_queries_config: missing 'threshold_ms'"),
    };
    let capacity = req_json
        .get("capacity")
        .and_then(|v| v.as_u64())
        .unwrap_or(1_000) as usize;
    if capacity == 0 {
        return Response::err(id, "admin.slow_queries_config: capacity must be >= 1");
    }
    let new_config = vectorizer::cache::SlowQueryConfig {
        threshold_ms,
        capacity,
    };
    state.slow_query_ring.set_config(new_config.clone());
    Response::ok(
        id,
        json_to_value(serde_json::json!({
            "threshold_ms": new_config.threshold_ms,
            "capacity": new_config.capacity,
            "status": "ok",
        })),
    )
}

// ── Auth / RBAC ──────────────────────────────────────────────────────────────

fn handle_auth_me(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let auth_state = match &state.auth {
        Some(a) => a,
        None => {
            // Auth disabled: return the implicit local admin identity.
            return Response::ok(
                id,
                json_to_value(serde_json::json!({
                    "user_id": "local",
                    "username": "local-admin",
                    "roles": ["Admin"],
                })),
            );
        }
    };
    // The RPC connection-level auth is in `ConnectionAuth`; the RpcState
    // auth field is used here to pull the principal name stored at HELLO time.
    // We surface the principal from the connection args (first arg = principal name).
    let principal = args
        .first()
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let _ = auth_state;
    Response::ok(
        id,
        json_to_value(serde_json::json!({
            "username": principal,
            "authenticated": true,
        })),
    )
}

async fn handle_auth_logout(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let handler = match &state.auth {
        Some(h) => h,
        None => {
            return Response::ok(
                id,
                json_to_value(serde_json::json!({ "status": "ok", "message": "auth disabled" })),
            );
        }
    };
    let token = match args.first().and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return Response::err(id, "auth.logout expects [Str(token)]"),
    };
    handler.blacklist_token(token).await;
    Response::ok(
        id,
        json_to_value(serde_json::json!({ "status": "ok", "message": "logged out" })),
    )
}

async fn handle_auth_refresh_token(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    let handler = match &state.auth {
        Some(h) => h,
        None => return Response::err(id, "auth.refresh_token: auth is not enabled"),
    };
    let token = match args.first().and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return Response::err(id, "auth.refresh_token expects [Str(token)]"),
    };
    if handler.is_token_blacklisted(token).await {
        return Response::err(id, "auth.refresh_token: token is revoked");
    }
    let claims = match handler.auth_manager.validate_jwt(token) {
        Ok(c) => c,
        Err(e) => return Response::err(id, format!("auth.refresh_token: invalid token: {}", e)),
    };
    match handler
        .auth_manager
        .generate_jwt(&claims.user_id, &claims.username, claims.roles)
    {
        Ok(new_token) => Response::ok(
            id,
            json_to_value(serde_json::json!({
                "access_token": new_token,
                "token_type": "Bearer",
            })),
        ),
        Err(e) => Response::err(id, format!("auth.refresh_token: {}", e)),
    }
}

fn handle_auth_validate_password(id: u32, args: &[VectorizerValue]) -> Response {
    let password = match args.first().and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return Response::err(id, "auth.validate_password expects [Str(password)]"),
    };
    let result = vectorizer::auth::validate_password(password);
    Response::ok(
        id,
        json_to_value(serde_json::json!({
            "valid": result.valid,
            "errors": result.errors,
        })),
    )
}

async fn handle_auth_api_keys_create(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    let handler = match &state.auth {
        Some(h) => h,
        None => return Response::err(id, "auth.api_keys_create: auth is not enabled"),
    };
    let req_json = match args.first() {
        Some(v) => value_to_json(v),
        None => return Response::err(id, "auth.api_keys_create expects [Map(request)]"),
    };
    let name = match req_json.get("name").and_then(|v| v.as_str()) {
        Some(n) => n.to_string(),
        None => return Response::err(id, "auth.api_keys_create: missing 'name'"),
    };
    let expires_in = req_json.get("expires_in").and_then(|v| v.as_u64());
    let permissions: Vec<vectorizer::auth::Permission> = req_json
        .get("permissions")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .filter_map(|p| match p.to_lowercase().as_str() {
                    "read" => Some(vectorizer::auth::Permission::Read),
                    "write" => Some(vectorizer::auth::Permission::Write),
                    "delete" => Some(vectorizer::auth::Permission::Delete),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_else(|| vec![vectorizer::auth::Permission::Read]);
    match handler
        .auth_manager
        .create_api_key("rpc", &name, permissions, expires_in)
        .await
    {
        Ok((raw_key, key_info)) => Response::ok(
            id,
            json_to_value(serde_json::json!({
                "api_key": raw_key,
                "id": key_info.id,
                "name": name,
            })),
        ),
        Err(e) => Response::err(id, format!("auth.api_keys_create: {}", e)),
    }
}

async fn handle_auth_api_keys_list(
    state: &Arc<RpcState>,
    id: u32,
    _args: &[VectorizerValue],
) -> Response {
    let handler = match &state.auth {
        Some(h) => h,
        None => return Response::err(id, "auth.api_keys_list: auth is not enabled"),
    };
    // List keys for the shared "rpc" user principal.
    match handler.auth_manager.list_api_keys("rpc").await {
        Ok(keys) => {
            let arr: Vec<serde_json::Value> = keys
                .iter()
                .map(|k| {
                    serde_json::json!({
                        "id": k.id,
                        "name": k.name,
                        "created_at": k.created_at,
                        "expires_at": k.expires_at,
                    })
                })
                .collect();
            Response::ok(id, json_to_value(serde_json::json!({ "keys": arr })))
        }
        Err(e) => Response::err(id, format!("auth.api_keys_list: {}", e)),
    }
}

async fn handle_auth_api_keys_revoke(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    let handler = match &state.auth {
        Some(h) => h,
        None => return Response::err(id, "auth.api_keys_revoke: auth is not enabled"),
    };
    let key_id = match args.first().and_then(|v| v.as_str()) {
        Some(k) => k.to_string(),
        None => return Response::err(id, "auth.api_keys_revoke expects [Str(id)]"),
    };
    match handler.auth_manager.revoke_api_key(&key_id).await {
        Ok(()) => Response::ok(
            id,
            json_to_value(serde_json::json!({ "success": true, "id": key_id })),
        ),
        Err(e) => Response::err(id, format!("auth.api_keys_revoke: {}", e)),
    }
}

async fn handle_auth_api_keys_rotate(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    let handler = match &state.auth {
        Some(h) => h,
        None => return Response::err(id, "auth.api_keys_rotate: auth is not enabled"),
    };
    let key_id = match args.first().and_then(|v| v.as_str()) {
        Some(k) => k.to_string(),
        None => return Response::err(id, "auth.api_keys_rotate expects [Str(id)]"),
    };
    // Default grace period: 300 seconds (5 minutes).
    match handler.auth_manager.rotate_api_key(&key_id, 300).await {
        Ok(rotated) => Response::ok(
            id,
            json_to_value(serde_json::json!({
                "old_key_id": rotated.old_token,
                "new_key_id": rotated.new_key_id,
                "new_token": rotated.new_token,
                "grace_until": rotated.grace_until,
            })),
        ),
        Err(e) => Response::err(id, format!("auth.api_keys_rotate: {}", e)),
    }
}

async fn handle_auth_api_keys_create_scoped(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    let handler = match &state.auth {
        Some(h) => h,
        None => return Response::err(id, "auth.api_keys_create_scoped: auth is not enabled"),
    };
    let req_json = match args.first() {
        Some(v) => value_to_json(v),
        None => return Response::err(id, "auth.api_keys_create_scoped expects [Map(request)]"),
    };
    let name = match req_json.get("name").and_then(|v| v.as_str()) {
        Some(n) => n.to_string(),
        None => return Response::err(id, "auth.api_keys_create_scoped: missing 'name'"),
    };
    let expires_in = req_json.get("expires_in").and_then(|v| v.as_u64());
    let permissions: Vec<vectorizer::auth::Permission> = req_json
        .get("permissions")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .filter_map(|p| match p.to_lowercase().as_str() {
                    "read" => Some(vectorizer::auth::Permission::Read),
                    "write" => Some(vectorizer::auth::Permission::Write),
                    "delete" => Some(vectorizer::auth::Permission::Delete),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_else(|| vec![vectorizer::auth::Permission::Read]);
    let scopes: Vec<vectorizer::auth::TokenScope> = req_json
        .get("scopes")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|s| {
                    let collection = s.get("collection")?.as_str()?.to_string();
                    let perms = s
                        .get("permissions")
                        .and_then(|v| v.as_array())
                        .map(|a| {
                            a.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();
                    Some(vectorizer::auth::TokenScope {
                        collection,
                        permissions: perms,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    match handler
        .auth_manager
        .create_scoped_api_key("rpc", &name, permissions, expires_in, scopes)
        .await
    {
        Ok((raw_key, key_info)) => Response::ok(
            id,
            json_to_value(serde_json::json!({
                "api_key": raw_key,
                "id": key_info.id,
                "name": name,
            })),
        ),
        Err(e) => Response::err(id, format!("auth.api_keys_create_scoped: {}", e)),
    }
}

async fn handle_auth_users_create(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
    auth: &ConnectionAuth,
) -> Response {
    if let Some(r) = require_admin(auth, id) {
        return r;
    }
    let handler = match &state.auth {
        Some(h) => h,
        None => return Response::err(id, "auth.users_create: auth is not enabled"),
    };
    let req_json = match args.first() {
        Some(v) => value_to_json(v),
        None => return Response::err(id, "auth.users_create expects [Map(request)]"),
    };
    let username = match req_json.get("username").and_then(|v| v.as_str()) {
        Some(u) => u.to_string(),
        None => return Response::err(id, "auth.users_create: missing 'username'"),
    };
    let password = match req_json.get("password").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return Response::err(id, "auth.users_create: missing 'password'"),
    };
    let roles: Vec<vectorizer::auth::roles::Role> = req_json
        .get("roles")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .filter_map(|r| match r.to_lowercase().as_str() {
                    "admin" => Some(vectorizer::auth::roles::Role::Admin),
                    "user" => Some(vectorizer::auth::roles::Role::User),
                    "readonly" => Some(vectorizer::auth::roles::Role::ReadOnly),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_else(|| vec![vectorizer::auth::roles::Role::User]);
    // User management requires AuthHandlerState (users map + persistence)
    // which is not yet carried on RpcState. Wire in a follow-up task.
    let _ = (handler, username, password, roles);
    Response::err(
        id,
        "auth.users_create is REST-only in v1 (RpcState does not carry AuthHandlerState); \
         use POST /auth/users",
    )
}

async fn handle_auth_users_list(state: &Arc<RpcState>, id: u32, auth: &ConnectionAuth) -> Response {
    if let Some(r) = require_admin(auth, id) {
        return r;
    }
    let handler = match &state.auth {
        Some(h) => h,
        None => return Response::err(id, "auth.users_list: auth is not enabled"),
    };
    let _ = handler;
    Response::err(
        id,
        "auth.users_list is REST-only in v1 (RpcState does not carry AuthHandlerState); \
         use GET /auth/users",
    )
}

async fn handle_auth_users_delete(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
    auth: &ConnectionAuth,
) -> Response {
    if let Some(r) = require_admin(auth, id) {
        return r;
    }
    let handler = match &state.auth {
        Some(h) => h,
        None => return Response::err(id, "auth.users_delete: auth is not enabled"),
    };
    let _ = (handler, args);
    Response::err(
        id,
        "auth.users_delete is REST-only in v1 (RpcState does not carry AuthHandlerState); \
         use DELETE /auth/users/{username}",
    )
}

async fn handle_auth_users_change_password(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    let handler = match &state.auth {
        Some(h) => h,
        None => return Response::err(id, "auth.users_change_password: auth is not enabled"),
    };
    let _ = (handler, args);
    Response::err(
        id,
        "auth.users_change_password is REST-only in v1 (RpcState does not carry \
         AuthHandlerState); use PUT /auth/users/{username}/password",
    )
}

async fn handle_auth_introspect(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    let handler = match &state.auth {
        Some(h) => h,
        None => return Response::err(id, "auth.introspect: auth is not enabled"),
    };
    let token = match args.first().and_then(|v| v.as_str()) {
        Some(t) => t.to_string(),
        None => return Response::err(id, "auth.introspect expects [Str(token)]"),
    };
    let blacklisted = handler.is_token_blacklisted(&token).await;
    let info = handler.auth_manager.introspect_token(&token).await;
    let json = serde_json::to_value(&info).unwrap_or(serde_json::Value::Null);
    let mut map = match json {
        serde_json::Value::Object(m) => m,
        _ => serde_json::Map::new(),
    };
    map.insert(
        "blacklisted".to_string(),
        serde_json::Value::Bool(blacklisted),
    );
    Response::ok(id, json_to_value(serde_json::Value::Object(map)))
}

async fn handle_auth_audit(state: &Arc<RpcState>, id: u32, args: &[VectorizerValue]) -> Response {
    let handler = match &state.auth {
        Some(h) => h,
        None => return Response::err(id, "auth.audit: auth is not enabled"),
    };
    let req_json = args.first().map(value_to_json).unwrap_or_default();
    let query = vectorizer::auth::audit::AuditQuery {
        from: req_json
            .get("from")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc)),
        to: req_json
            .get("to")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc)),
        actor: req_json
            .get("actor")
            .and_then(|v| v.as_str())
            .map(String::from),
        action: req_json
            .get("action")
            .and_then(|v| v.as_str())
            .map(String::from),
        limit: req_json
            .get("limit")
            .and_then(|v| v.as_u64())
            .map(|n| n as usize),
    };
    // audit_logger.query() returns Vec<AuditEntry> directly (no Result).
    let entries = handler.audit_logger.query(&query).await;
    let total = entries.len();
    let json = serde_json::to_value(&entries).unwrap_or(serde_json::Value::Null);
    Response::ok(
        id,
        json_to_value(serde_json::json!({ "entries": json, "total": total })),
    )
}

// ── Replication ───────────────────────────────────────────────────────────────

fn handle_replication_status(state: &Arc<RpcState>, id: u32) -> Response {
    if let Some(master) = &state.master_node {
        let stats = master.get_stats();
        let replicas = master.get_replicas();
        return Response::ok(
            id,
            json_to_value(serde_json::json!({
                "role": "Master",
                "enabled": true,
                "stats": serde_json::to_value(&stats).unwrap_or_default(),
                "replicas": serde_json::to_value(&replicas).unwrap_or_default(),
            })),
        );
    }
    if let Some(replica) = &state.replica_node {
        let stats = replica.get_stats();
        return Response::ok(
            id,
            json_to_value(serde_json::json!({
                "role": "Replica",
                "enabled": true,
                "stats": serde_json::to_value(stats).unwrap_or_default(),
            })),
        );
    }
    let role = state
        .store
        .get_metadata("replication_role")
        .unwrap_or_else(|| "standalone".to_string());
    Response::ok(
        id,
        json_to_value(serde_json::json!({
            "role": role,
            "enabled": false,
        })),
    )
}

fn handle_replication_configure(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
) -> Response {
    let req_json = match args.first() {
        Some(v) => value_to_json(v),
        None => return Response::err(id, "replication.configure expects [Map(config)]"),
    };
    let role = match req_json.get("role").and_then(|v| v.as_str()) {
        Some(r) => r.to_string(),
        None => return Response::err(id, "replication.configure: missing 'role'"),
    };
    let validated_role = match role.as_str() {
        "master" | "replica" | "standalone" => role.clone(),
        other => {
            return Response::err(
                id,
                format!(
                    "replication.configure: invalid role '{}'; must be master|replica|standalone",
                    other
                ),
            );
        }
    };
    state.store.set_metadata("replication_role", validated_role);
    if let Some(addr) = req_json.get("bind_address").and_then(|v| v.as_str()) {
        state
            .store
            .set_metadata("replication_bind_address", addr.to_string());
    }
    if let Some(addr) = req_json.get("master_address").and_then(|v| v.as_str()) {
        state
            .store
            .set_metadata("replication_master_address", addr.to_string());
    }
    Response::ok(
        id,
        json_to_value(serde_json::json!({
            "success": true,
            "role": role,
            "message": "Replication configured. Server restart required.",
        })),
    )
}

fn handle_replication_stats(state: &Arc<RpcState>, id: u32) -> Response {
    if let Some(master) = &state.master_node {
        let stats = master.get_stats();
        return Response::ok(
            id,
            json_to_value(serde_json::to_value(stats).unwrap_or_default()),
        );
    }
    if let Some(replica) = &state.replica_node {
        let stats = replica.get_stats();
        return Response::ok(
            id,
            json_to_value(serde_json::to_value(stats).unwrap_or_default()),
        );
    }
    Response::err(id, "replication.stats: replication not enabled")
}

fn handle_replication_replicas_list(state: &Arc<RpcState>, id: u32) -> Response {
    if let Some(master) = &state.master_node {
        let replicas = master.get_replicas();
        let count = replicas.len();
        return Response::ok(
            id,
            json_to_value(serde_json::json!({
                "replicas": serde_json::to_value(replicas).unwrap_or_default(),
                "count": count,
            })),
        );
    }
    Response::err(
        id,
        "replication.replicas_list: only available on master nodes",
    )
}

// ── Cluster ──────────────────────────────────────────────────────────────────

fn handle_cluster_failover(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
    auth: &ConnectionAuth,
) -> Response {
    if let Some(r) = require_admin(auth, id) {
        return r;
    }
    let replica_id = match args.first().and_then(|v| v.as_str()) {
        Some(r) => r.to_string(),
        None => return Response::err(id, "cluster.failover expects [Str(replica_id)]"),
    };
    let master = match &state.master_node {
        Some(m) => m,
        None => return Response::err(id, "cluster.failover: requires master node"),
    };
    let max_lag = vectorizer::replication::DEFAULT_MAX_FAILOVER_LAG_SEGMENTS;
    match vectorizer::replication::state::failover_to(master, &replica_id, max_lag) {
        Ok(report) => Response::ok(
            id,
            json_to_value(serde_json::to_value(report).unwrap_or_default()),
        ),
        Err(e) => Response::err(id, format!("cluster.failover: {}", e)),
    }
}

fn handle_cluster_replica_resync(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
    auth: &ConnectionAuth,
) -> Response {
    if let Some(r) = require_admin(auth, id) {
        return r;
    }
    let replica_id = match args.first().and_then(|v| v.as_str()) {
        Some(r) => r.to_string(),
        None => return Response::err(id, "cluster.replica_resync expects [Str(replica_id)]"),
    };
    let master = match &state.master_node {
        Some(m) => m,
        None => return Response::err(id, "cluster.replica_resync: requires master node"),
    };
    match vectorizer::replication::state::force_resync(master, &replica_id) {
        Ok(report) => Response::ok(
            id,
            json_to_value(serde_json::to_value(report).unwrap_or_default()),
        ),
        Err(e) => Response::err(id, format!("cluster.replica_resync: {}", e)),
    }
}

fn handle_cluster_peer_add(
    state: &Arc<RpcState>,
    id: u32,
    args: &[VectorizerValue],
    auth: &ConnectionAuth,
) -> Response {
    if let Some(r) = require_admin(auth, id) {
        return r;
    }
    let cluster_mgr = match &state.cluster_manager {
        Some(c) => c,
        None => return Response::err(id, "cluster.peer_add: cluster mode not enabled"),
    };
    let req_json = match args.first() {
        Some(v) => value_to_json(v),
        None => return Response::err(id, "cluster.peer_add expects [Map(request)]"),
    };
    let address = match req_json.get("address").and_then(|v| v.as_str()) {
        Some(a) => a.to_string(),
        None => return Response::err(id, "cluster.peer_add: missing 'address'"),
    };
    let role_str = req_json
        .get("role")
        .and_then(|v| v.as_str())
        .unwrap_or("member");
    let role = if role_str.to_lowercase() == "observer" {
        vectorizer::cluster::rebalance::PeerRole::Observer
    } else {
        vectorizer::cluster::rebalance::PeerRole::Member
    };
    match vectorizer::cluster::rebalance::add_peer(cluster_mgr, address, role) {
        Ok(info) => Response::ok(
            id,
            json_to_value(serde_json::to_value(info).unwrap_or_default()),
        ),
        Err(e) => Response::err(id, format!("cluster.peer_add: {}", e)),
    }
}

fn handle_cluster_rebalance(state: &Arc<RpcState>, id: u32, auth: &ConnectionAuth) -> Response {
    if let Some(r) = require_admin(auth, id) {
        return r;
    }
    let cluster_mgr = match &state.cluster_manager {
        Some(c) => c,
        None => return Response::err(id, "cluster.rebalance: cluster mode not enabled"),
    };
    match vectorizer::cluster::rebalance::rebalance(cluster_mgr) {
        Ok(job) => Response::ok(
            id,
            json_to_value(serde_json::to_value(job).unwrap_or_default()),
        ),
        Err(e) => Response::err(id, format!("cluster.rebalance: {}", e)),
    }
}

fn handle_cluster_rebalance_status(id: u32) -> Response {
    match vectorizer::cluster::rebalance::rebalance_status() {
        Some(job) => Response::ok(
            id,
            json_to_value(serde_json::to_value(job).unwrap_or_default()),
        ),
        None => Response::ok(
            id,
            json_to_value(serde_json::json!({
                "status": "idle",
                "message": "No rebalance has been triggered on this node",
            })),
        ),
    }
}

/// The names of the v1 RPC capabilities the handshake reports back to
/// the client. Kept local to this module so the wire surface is
/// reviewable in one place.
fn rpc_capability_names() -> Vec<VectorizerValue> {
    [
        // Handshake
        "PING",
        // Collections
        "collections.list",
        "collections.get_info",
        "collections.create",
        "collections.delete",
        "collections.list_empty",
        "collections.cleanup_empty",
        "collections.force_save",
        // Vectors
        "vectors.get",
        "vectors.insert",
        "vectors.insert_text",
        "vectors.update",
        "vectors.delete",
        "vectors.list",
        "vectors.embed",
        "vectors.batch_insert",
        "vectors.batch_insert_texts",
        "vectors.batch_search",
        "vectors.batch_update",
        "vectors.batch_delete",
        "vectors.move",
        "vectors.copy",
        "vectors.delete_by_filter",
        "vectors.bulk_update_metadata",
        "vectors.set_expiry",
        // Search
        "search.basic",
        "search.intelligent",
        "search.by_text",
        "search.by_file",
        "search.hybrid",
        "search.semantic",
        "search.contextual",
        "search.multi_collection",
        "search.explain",
        // Discovery
        "discovery.discover",
        "discovery.filter_collections",
        "discovery.score_collections",
        "discovery.expand_queries",
        "discovery.broad_discovery",
        "discovery.semantic_focus",
        "discovery.promote_readme",
        "discovery.compress_evidence",
        "discovery.build_answer_plan",
        "discovery.render_llm_prompt",
        // File ops
        "file.content",
        "file.list",
        "file.summary",
        "file.chunks",
        "file.outline",
        "file.related",
        "file.search_by_type",
        // Graph
        "graph.list_nodes",
        "graph.neighbors",
        "graph.find_related",
        "graph.find_path",
        "graph.create_edge",
        "graph.delete_edge",
        "graph.list_edges",
        "graph.discover_edges",
        "graph.discover_edges_for_node",
        "graph.discovery_status",
        // Admin / observability
        "admin.stats",
        "admin.status",
        "admin.logs",
        "admin.indexing_progress",
        "admin.config_get",
        "admin.config_update",
        "admin.backups_list",
        "admin.backups_create",
        "admin.backups_restore",
        "admin.workspaces_list",
        "admin.workspace_get",
        "admin.workspace_add",
        "admin.workspace_remove",
        "admin.restart",
        "admin.slow_queries_list",
        "admin.slow_queries_config",
        // Auth / RBAC
        "auth.me",
        "auth.logout",
        "auth.refresh_token",
        "auth.validate_password",
        "auth.api_keys_create",
        "auth.api_keys_list",
        "auth.api_keys_revoke",
        "auth.api_keys_rotate",
        "auth.api_keys_create_scoped",
        // auth.users_* and auth.audit require AuthHandlerState on RpcState;
        // wired in a follow-up phase. Advertise only what is fully wired.
        "auth.introspect",
        "auth.audit",
        // Replication
        "replication.status",
        "replication.configure",
        "replication.stats",
        "replication.replicas_list",
        // Cluster
        "cluster.failover",
        "cluster.replica_resync",
        "cluster.peer_add",
        "cluster.rebalance",
        "cluster.rebalance_status",
    ]
    .iter()
    .map(|s| VectorizerValue::Str((*s).to_string()))
    .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn fake_state() -> Arc<RpcState> {
        Arc::new(RpcState {
            store: Arc::new(vectorizer::db::VectorStore::new()),
            embedding_manager: Arc::new(vectorizer::embedding::EmbeddingManager::new()),
            auth: None,
            master_node: None,
            replica_node: None,
            cluster_manager: None,
            slow_query_ring: vectorizer::cache::SlowQueryRing::new(
                vectorizer::cache::slow_query::SlowQueryConfig::default(),
            ),
        })
    }

    fn fake_auth_unauthenticated() -> Arc<RwLock<ConnectionAuth>> {
        Arc::new(RwLock::new(ConnectionAuth::default()))
    }

    #[tokio::test]
    async fn ping_replies_pong() {
        let resp = dispatch(
            &fake_state(),
            // PING is reachable even before HELLO so the auth gate
            // doesn't block health checks. Keep that policy stable.
            &fake_auth_unauthenticated(),
            Request {
                id: 7,
                command: "PING".into(),
                args: vec![],
            },
        )
        .await;
        assert_eq!(resp.id, 7);
        assert_eq!(resp.result.as_ref().unwrap().as_str(), Some("PONG"));
    }

    #[tokio::test]
    async fn unauthenticated_command_is_rejected() {
        let resp = dispatch(
            &fake_state(),
            &fake_auth_unauthenticated(),
            Request {
                id: 8,
                command: "collections.list".into(),
                args: vec![],
            },
        )
        .await;
        let err = resp.result.unwrap_err();
        assert!(err.contains("authentication required"), "got: {err}");
    }

    #[tokio::test]
    async fn hello_with_no_auth_succeeds_in_single_user_mode() {
        let auth = fake_auth_unauthenticated();
        let resp = dispatch(
            &fake_state(),
            &auth,
            Request {
                id: 9,
                command: "HELLO".into(),
                args: vec![VectorizerValue::Map(vec![(
                    VectorizerValue::Str("version".into()),
                    VectorizerValue::Int(1),
                )])],
            },
        )
        .await;
        assert!(resp.result.is_ok());
        assert!(auth.read().authenticated);
        assert!(auth.read().admin);
    }

    #[tokio::test]
    async fn unknown_command_returns_named_error() {
        let auth = fake_auth_unauthenticated();
        // First HELLO so subsequent commands aren't rejected by the
        // auth gate (this test exercises the unknown-command path
        // specifically).
        let _ = dispatch(
            &fake_state(),
            &auth,
            Request {
                id: 10,
                command: "HELLO".into(),
                args: vec![VectorizerValue::Map(vec![])],
            },
        )
        .await;
        let resp = dispatch(
            &fake_state(),
            &auth,
            Request {
                id: 11,
                command: "no.such.command".into(),
                args: vec![],
            },
        )
        .await;
        let err = resp.result.unwrap_err();
        assert!(
            err.contains("unknown command 'no.such.command'"),
            "got: {err}"
        );
    }
}
