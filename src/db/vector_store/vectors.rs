//! Vector-level CRUD dispatched through `VectorStore`.
//!
//! Each method writes to the WAL first (when WAL is enabled), then
//! updates the in-memory collection, then marks the collection for
//! auto-save. Batched inserts use 1000-vector chunks so the per-call
//! DashMap lock scope stays bounded.

use tracing::debug;

use super::VectorStore;
use crate::error::Result;
use crate::models::Vector;

impl VectorStore {
    /// Insert vectors into a collection
    pub fn insert(&self, collection_name: &str, vectors: Vec<Vector>) -> Result<()> {
        debug!(
            "Inserting {} vectors into collection '{}'",
            vectors.len(),
            collection_name
        );

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
    pub fn update(&self, collection_name: &str, vector: Vector) -> Result<()> {
        debug!(
            "Updating vector '{}' in collection '{}'",
            vector.id, collection_name
        );

        // Log to WAL before applying changes
        self.log_wal_update(collection_name, &vector)?;

        let mut collection_ref = self.get_collection_mut(collection_name)?;
        // Use atomic update method (2x faster than delete+add)
        collection_ref.update_vector(vector)?;

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

        let mut collection_ref = self.get_collection_mut(collection_name)?;
        collection_ref.delete_vector(vector_id)?;

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
