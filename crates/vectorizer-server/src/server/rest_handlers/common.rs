//! Shared helpers used across multiple REST handler modules.

use uuid::Uuid;
use vectorizer::db::{AdmissionError, AdmissionStatus, UpsertQueue, UpsertTicket};
use vectorizer::models::CollectionConfig;
use vectorizer_core::error::VectorizerError;

use crate::server::VectorizerServer;
use crate::server::error_middleware::{ErrorResponse, create_queue_full_error};
use crate::server::runtime_metrics::{DashboardEvent, build_collections_snapshot};

/// Phase30 §1.4 — publish an immediate `Collections` snapshot on the
/// dashboard broadcast bus so the dashboard table reflects a
/// create / delete / rename without waiting for the 30 s tick. The
/// `send` call returns an error when no subscribers are alive, which
/// is the normal idle state — drop it on the floor.
pub(super) fn publish_collections_snapshot(state: &VectorizerServer) {
    let snapshot = build_collections_snapshot(&state.store);
    let _ = state
        .dashboard_tx
        .send(DashboardEvent::Collections(snapshot));
}

/// Admit one in-flight upsert against the per-collection queue
/// (issue #263). On hard-limit exceedance returns a 429 with
/// `Retry-After` already set; on high-water exceedance emits a warn
/// log and admits the request anyway. The returned [`UpsertTicket`]
/// MUST be held for the duration of the upsert so the depth counter
/// decrements when work completes (or panics — Drop is called on
/// unwind).
pub(super) fn admit_upsert(
    queue: &UpsertQueue,
    collection: &str,
) -> Result<UpsertTicket, ErrorResponse> {
    use vectorizer::monitoring::metrics::METRICS;

    match queue.try_admit(collection) {
        Ok((ticket, status)) => {
            // Update gauges with the post-admit depth so the /metrics
            // scrape reflects the live in-flight number.
            let depth = queue.depth(collection) as f64;
            METRICS
                .upsert_queue_depth
                .with_label_values(&[collection])
                .set(depth);
            METRICS
                .upsert_in_flight
                .with_label_values(&[collection])
                .set(depth);

            if status == AdmissionStatus::AdmittedHighWater {
                METRICS
                    .upsert_rejected_total
                    .with_label_values(&["queue_high_water_warn"])
                    .inc();
                tracing::warn!(
                    collection = collection,
                    depth = queue.depth(collection),
                    hard_limit = queue.hard_limit(),
                    "upsert queue depth at or above high-water mark",
                );
            }
            Ok(ticket)
        }
        Err(AdmissionError::QueueFull {
            depth,
            hard_limit,
            retry_after_seconds,
        }) => {
            METRICS
                .upsert_rejected_total
                .with_label_values(&["queue_full"])
                .inc();
            tracing::warn!(
                collection = collection,
                depth = depth,
                hard_limit = hard_limit,
                retry_after_seconds = retry_after_seconds,
                "upsert queue full — replying 429",
            );
            Err(create_queue_full_error(
                collection,
                depth,
                hard_limit,
                retry_after_seconds,
            ))
        }
    }
}

/// Refuse a text operation on a collection that stores pre-computed vectors
/// (phase6_raw-vector-collections).
///
/// Such a collection has no embedding provider, so there is nothing to
/// vectorize the text with. Falling back to the server default would embed it
/// with BM25 — writing vectors from a different space than the ones the
/// caller uploaded, giving a collection that searches badly with nothing
/// reporting why. That silent coercion is what phase33 (issue #306) removed;
/// this keeps it from returning through the raw-vector door.
///
/// `operation` names the refused call in the error, so a caller who reached
/// the wrong endpoint learns which one to use instead.
///
/// This config-based form is preferred wherever the collection is already in
/// hand: taking a second DashMap `Ref` on the same shard while one is alive is
/// the shape behind this repo's phase39 production deadlock, and reading the
/// config we already hold costs nothing.
pub(super) fn reject_text_on_raw_vector_config(
    config: &CollectionConfig,
    collection_name: &str,
    operation: &str,
) -> Result<(), ErrorResponse> {
    if config.is_raw_vector() {
        return Err(ErrorResponse::from(
            VectorizerError::CollectionHasNoEmbeddingProvider {
                collection: collection_name.to_string(),
                operation: operation.to_string(),
            },
        ));
    }
    Ok(())
}

/// Name-based form of [`reject_text_on_raw_vector_config`], for callers that
/// do not hold the collection yet.
///
/// A missing collection is *not* rejected here — the caller's own existence
/// check owns that error, and answering "no embedding provider" for a
/// collection that does not exist would send the caller down the wrong path
/// entirely.
pub(super) fn reject_text_on_raw_vector_collection(
    state: &VectorizerServer,
    collection_name: &str,
    operation: &str,
) -> Result<(), ErrorResponse> {
    match state.store.get_collection(collection_name) {
        Ok(collection) => {
            reject_text_on_raw_vector_config(collection.config(), collection_name, operation)
        }
        Err(_) => Ok(()),
    }
}

/// Extract tenant ID as UUID from request extensions (if present)
///
/// Returns None if:
/// - Hub mode is disabled
/// - No tenant context in request
/// - Tenant ID is not a valid UUID
pub(super) fn extract_tenant_id(
    tenant_ctx: &Option<axum::Extension<vectorizer::hub::middleware::RequestTenantContext>>,
) -> Option<Uuid> {
    tenant_ctx
        .as_ref()
        .and_then(|ctx| Uuid::parse_str(&ctx.0.0.tenant_id).ok())
}

/// Deterministic UUID derived from a collection name.
///
/// Prior to this helper, three call sites in `rest_handlers.rs` passed a
/// freshly-generated `Uuid::new_v4()` to `HubManager::record_usage`, so every
/// request against the same collection was recorded under a different UUID,
/// making per-collection usage aggregation impossible.
///
/// `Uuid::new_v5` with a fixed namespace produces a stable UUID: the same
/// collection name always yields the same UUID, no on-disk migration needed.
/// The namespace is a v4 UUID minted for Vectorizer and hardcoded here — it
/// never needs to change, and its only role is to isolate our v5 outputs from
/// any other system that reuses `NAMESPACE_OID` / `NAMESPACE_URL`.
pub(super) const COLLECTION_NAMESPACE_UUID: Uuid =
    Uuid::from_u128(0x7f_5a_c6_40_3d_fe_4e_1a_9d_82_d8_2d_4e_a7_55_01);

/// Compute the stable metrics UUID for a named collection. Same name in →
/// same UUID out, no storage required.
pub(crate) fn collection_metrics_uuid(name: &str) -> Uuid {
    Uuid::new_v5(&COLLECTION_NAMESPACE_UUID, name.as_bytes())
}
