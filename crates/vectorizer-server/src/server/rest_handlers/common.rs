//! Shared helpers used across multiple REST handler modules.

use uuid::Uuid;

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
