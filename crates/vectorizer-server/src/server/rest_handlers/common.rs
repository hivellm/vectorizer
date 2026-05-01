//! Shared helpers used across multiple REST handler modules.

use uuid::Uuid;
use vectorizer::db::{AdmissionError, AdmissionStatus, UpsertQueue, UpsertTicket};

use crate::server::error_middleware::{ErrorResponse, create_queue_full_error};

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
