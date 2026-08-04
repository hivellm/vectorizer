//! Vector-level CRUD dispatched through `VectorStore`.
//!
//! Each method writes to the WAL first (when WAL is enabled), then
//! updates the in-memory collection, then marks the collection for
//! auto-save. Batched inserts use 1000-vector chunks so the per-call
//! DashMap lock scope stays bounded.

use tracing::debug;

use super::{CollectionType, VectorStore};
use crate::error::{Result, VectorizerError};
use crate::models::{Payload, Vector};

/// Apply a collection-level TTL to `vectors` by stamping `__expires_at`.
///
/// Vectors that already carry an explicit `__expires_at` are left alone —
/// a per-vector expiry (`PATCH …/expiry`, or an insert that set the field
/// directly) is more specific than the collection rule and wins.
///
/// Stamping happens before the WAL record is written, so a WAL replay
/// after a restart restores the original expiry instead of computing a
/// fresh one from the replay time.
///
/// A payload whose JSON root is not an object has nowhere to hold the
/// field, so it is rejected rather than silently inserted without an
/// expiry: a TTL that quietly does not apply is the bug this exists to
/// prevent.
fn apply_collection_ttl(
    vectors: &mut [Vector],
    ttl_secs: u64,
    collection_name: &str,
) -> Result<()> {
    let expires_at = Vector::now_ms() + (ttl_secs as i64) * 1000;

    for vector in vectors.iter_mut() {
        let payload = vector
            .payload
            .get_or_insert_with(|| Payload::new(serde_json::Value::Object(Default::default())));

        if payload.expires_at().is_some() {
            continue;
        }
        if !payload.data.is_object() {
            return Err(VectorizerError::ConfigurationError(format!(
                "collection '{}' has a TTL of {}s but vector '{}' carries a \
                 non-object payload, which cannot hold the '__expires_at' \
                 field; wrap the payload in a JSON object or clear the \
                 collection TTL",
                collection_name, ttl_secs, vector.id
            )));
        }
        payload.set_expires_at(expires_at);
    }

    Ok(())
}

impl VectorStore {
    /// Insert vectors into a collection
    pub fn insert(&self, collection_name: &str, mut vectors: Vec<Vector>) -> Result<()> {
        debug!(
            "Inserting {} vectors into collection '{}'",
            vectors.len(),
            collection_name
        );

        // Reads the free-form metadata DashMap, not the collections map, so
        // it cannot re-enter the shard lock the insert loop takes below.
        if let Some(ttl_secs) = self.collection_ttl(collection_name) {
            apply_collection_ttl(&mut vectors, ttl_secs, collection_name)?;
        }

        // Log to WAL before applying changes
        self.log_wal_insert(collection_name, &vectors)?;

        // Optimized: Use insert_batch for much better performance
        // insert_batch processes vectors in batch which is 10-100x faster than individual inserts
        // Use larger chunks to reduce lock acquisition overhead
        let chunk_size = 1000; // Large chunks for maximum throughput

        for chunk in vectors.chunks(chunk_size) {
            // Get mutable reference for this chunk only
            let mut collection_ref = self.get_collection_mut(collection_name)?;

            // Use insert_batch which is optimized for batch operations
            // This is much faster than calling add_vector individually
            collection_ref.insert_batch(chunk.to_vec())?;

            // Lock is released here when collection_ref goes out of scope
        }

        // Mark collection for auto-save
        self.mark_collection_for_save(collection_name);

        Ok(())
    }

    /// Update a vector in a collection
    pub fn update(&self, collection_name: &str, mut vector: Vector) -> Result<()> {
        debug!(
            "Updating vector '{}' in collection '{}'",
            vector.id, collection_name
        );

        // An update replaces the stored payload wholesale, so without this
        // the caller's new payload would drop the `__expires_at` stamped at
        // insert time and the vector would outlive the collection TTL.
        if let Some(ttl_secs) = self.collection_ttl(collection_name) {
            apply_collection_ttl(std::slice::from_mut(&mut vector), ttl_secs, collection_name)?;
        }

        // Log to WAL before applying changes
        self.log_wal_update(collection_name, &vector)?;

        // Prefer a shared DashMap shard reference for variants whose inner
        // update uses interior mutability (CPU, Sharded), mirroring the
        // pattern `delete` uses below. Holding only a shared shard lock
        // means concurrent readers do not deadlock against this call, and
        // it removes the `get_collection`/`get_collection_mut` re-entrancy
        // trap documented on those two methods (bulk_update_metadata
        // production deadlock, fixed in phase39) for the common case.
        let pending_gpu_vector = {
            let collection_ref = self.get_collection(collection_name)?;
            match &*collection_ref {
                CollectionType::Cpu(c) => {
                    c.update(vector)?;
                    None
                }
                CollectionType::Sharded(c) => {
                    c.update(vector)?;
                    None
                }
                CollectionType::DistributedSharded(_) => {
                    return Err(VectorizerError::Storage(
                        "update is not supported synchronously on distributed \
                         collections; use the async cluster router"
                            .to_string(),
                    ));
                }
                #[cfg(feature = "hive-gpu")]
                CollectionType::HiveGpu(_) => Some(vector),
            }
        };

        // HiveGpu still needs &mut self because it tracks vector_count in a
        // non-atomic field. Re-acquire the shard with an exclusive lock only
        // for that case.
        if let Some(vector) = pending_gpu_vector {
            let mut collection_ref = self.get_collection_mut(collection_name)?;
            collection_ref.update_vector(vector)?;
        }

        // Mark collection for auto-save
        self.mark_collection_for_save(collection_name);

        Ok(())
    }

    /// Delete a vector from a collection
    pub fn delete(&self, collection_name: &str, vector_id: &str) -> Result<()> {
        debug!(
            "Deleting vector '{}' from collection '{}'",
            vector_id, collection_name
        );

        // Log to WAL before applying changes
        self.log_wal_delete(collection_name, vector_id)?;

        // Prefer a shared DashMap shard reference for variants whose inner
        // delete uses interior mutability (CPU, Sharded). Holding only a
        // shared shard lock means concurrent readers (e.g. an HTTP handler
        // or the replication apply loop) do not deadlock against each other.
        let needs_mut = {
            let collection_ref = self.get_collection(collection_name)?;
            match &*collection_ref {
                CollectionType::Cpu(c) => {
                    c.delete(vector_id)?;
                    false
                }
                CollectionType::Sharded(c) => {
                    c.delete(vector_id)?;
                    false
                }
                CollectionType::DistributedSharded(_) => {
                    return Err(VectorizerError::Storage(
                        "delete is not supported synchronously on distributed \
                         collections; use the async cluster router"
                            .to_string(),
                    ));
                }
                #[cfg(feature = "hive-gpu")]
                CollectionType::HiveGpu(_) => true,
            }
        };

        // HiveGpu still needs &mut self because it tracks vector_count in a
        // non-atomic field. Re-acquire the shard with an exclusive lock only
        // for that case.
        if needs_mut {
            let mut collection_ref = self.get_collection_mut(collection_name)?;
            collection_ref.delete_vector(vector_id)?;
        }

        // Mark collection for auto-save
        self.mark_collection_for_save(collection_name);

        Ok(())
    }

    /// Get a vector by ID
    pub fn get_vector(&self, collection_name: &str, vector_id: &str) -> Result<Vector> {
        let collection_ref = self.get_collection(collection_name)?;
        collection_ref.get_vector(vector_id)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::models::{CollectionConfig, DistanceMetric};

    const COLLECTION: &str = "ttl_stamp";

    fn store_with_collection() -> VectorStore {
        let store = VectorStore::new();
        store
            .create_collection(
                COLLECTION,
                CollectionConfig {
                    dimension: 4,
                    metric: DistanceMetric::Cosine,
                    ..Default::default()
                },
            )
            .unwrap();
        store
    }

    fn vector_with(id: &str, payload: Option<Payload>) -> Vector {
        Vector {
            id: id.to_string(),
            data: vec![0.1, 0.2, 0.3, 0.4],
            sparse: None,
            payload,
            document_id: None,
        }
    }

    /// Read the stored expiry through the raw accessor — `get_vector` hides
    /// expired vectors, so it cannot answer "what was stamped?".
    fn stored_expiry(store: &VectorStore, id: &str) -> Option<i64> {
        store
            .get_collection(COLLECTION)
            .unwrap()
            .get_all_vectors()
            .into_iter()
            .find(|v| v.id == id)
            .expect("vector stored")
            .payload
            .and_then(|p| p.expires_at())
    }

    #[test]
    fn insert_stamps_expiry_from_the_collection_ttl() {
        let store = store_with_collection();
        store.set_collection_ttl(COLLECTION, Some(60));

        let before = Vector::now_ms();
        store
            .insert(
                COLLECTION,
                vec![vector_with(
                    "v1",
                    Some(Payload::new(serde_json::json!({ "tag": "a" }))),
                )],
            )
            .unwrap();
        let after = Vector::now_ms();

        let expires_at = stored_expiry(&store, "v1").expect("collection TTL must stamp an expiry");
        assert!(
            expires_at >= before + 60_000 && expires_at <= after + 60_000,
            "expiry {expires_at} must be ~60s ahead of insertion (window {}..={})",
            before + 60_000,
            after + 60_000
        );

        // The rest of the payload survives the stamp.
        let vector = store
            .get_collection(COLLECTION)
            .unwrap()
            .get_all_vectors()
            .into_iter()
            .find(|v| v.id == "v1")
            .unwrap();
        assert_eq!(
            vector.payload.unwrap().data["tag"].as_str(),
            Some("a"),
            "stamping must not clobber the caller's payload"
        );
    }

    #[test]
    fn insert_without_a_collection_ttl_stamps_nothing() {
        let store = store_with_collection();

        store
            .insert(
                COLLECTION,
                vec![vector_with(
                    "v1",
                    Some(Payload::new(serde_json::json!({ "tag": "a" }))),
                )],
            )
            .unwrap();

        assert_eq!(stored_expiry(&store, "v1"), None);
    }

    #[test]
    fn explicit_per_vector_expiry_wins_over_the_collection_ttl() {
        let store = store_with_collection();
        store.set_collection_ttl(COLLECTION, Some(60));

        let explicit = Vector::now_ms() + 5_000_000;
        let mut payload = Payload::new(serde_json::json!({ "tag": "a" }));
        payload.set_expires_at(explicit);

        store
            .insert(COLLECTION, vec![vector_with("v1", Some(payload))])
            .unwrap();

        assert_eq!(
            stored_expiry(&store, "v1"),
            Some(explicit),
            "a per-vector expiry is more specific than the collection rule"
        );
    }

    #[test]
    fn payloadless_vector_gets_a_payload_carrying_the_expiry() {
        let store = store_with_collection();
        store.set_collection_ttl(COLLECTION, Some(60));

        store
            .insert(COLLECTION, vec![vector_with("v1", None)])
            .unwrap();

        assert!(
            stored_expiry(&store, "v1").is_some(),
            "a vector with no payload must still expire, or the TTL is a \
             partial lie"
        );
    }

    #[test]
    fn non_object_payload_is_rejected_while_a_ttl_is_configured() {
        let store = store_with_collection();
        store.set_collection_ttl(COLLECTION, Some(60));

        let err = store
            .insert(
                COLLECTION,
                vec![vector_with(
                    "v1",
                    Some(Payload::new(serde_json::json!("just a string"))),
                )],
            )
            .expect_err("a payload that cannot hold __expires_at must fail loudly");
        assert!(
            err.to_string().contains("non-object payload"),
            "error must name the cause, got: {err}"
        );

        // And nothing was written.
        assert!(
            store
                .get_collection(COLLECTION)
                .unwrap()
                .get_all_vectors()
                .is_empty()
        );
    }

    #[test]
    fn non_object_payload_is_accepted_without_a_collection_ttl() {
        let store = store_with_collection();

        store
            .insert(
                COLLECTION,
                vec![vector_with(
                    "v1",
                    Some(Payload::new(serde_json::json!("just a string"))),
                )],
            )
            .expect("the TTL check must not reject payloads it has no reason to touch");
    }

    #[test]
    fn update_restamps_the_expiry_the_new_payload_dropped() {
        let store = store_with_collection();
        store.set_collection_ttl(COLLECTION, Some(60));

        store
            .insert(
                COLLECTION,
                vec![vector_with(
                    "v1",
                    Some(Payload::new(serde_json::json!({ "tag": "a" }))),
                )],
            )
            .unwrap();
        assert!(stored_expiry(&store, "v1").is_some());

        // A caller-supplied payload with no expiry field would otherwise
        // make the vector immortal.
        store
            .update(
                COLLECTION,
                vector_with("v1", Some(Payload::new(serde_json::json!({ "tag": "b" })))),
            )
            .unwrap();

        assert!(
            stored_expiry(&store, "v1").is_some(),
            "an update must not strip the collection TTL"
        );
    }

    #[test]
    fn ttl_applies_only_to_the_collection_it_was_set_on() {
        let store = store_with_collection();
        store
            .create_collection(
                "ttl_stamp_other",
                CollectionConfig {
                    dimension: 4,
                    metric: DistanceMetric::Cosine,
                    ..Default::default()
                },
            )
            .unwrap();
        store.set_collection_ttl(COLLECTION, Some(60));

        store
            .insert(
                "ttl_stamp_other",
                vec![vector_with(
                    "v1",
                    Some(Payload::new(serde_json::json!({ "tag": "a" }))),
                )],
            )
            .unwrap();

        let expiry = store
            .get_collection("ttl_stamp_other")
            .unwrap()
            .get_all_vectors()
            .into_iter()
            .find(|v| v.id == "v1")
            .unwrap()
            .payload
            .and_then(|p| p.expires_at());
        assert_eq!(expiry, None);
    }
}
