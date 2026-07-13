//! Native per-collection snapshots + collection reindex.
//!
//! Split out of the persistence monolith in phase41 §4.3.

use std::path::PathBuf;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use super::super::{CollectionType, VectorStore};
use crate::error::{Result, VectorizerError};

// ─── Native snapshot types ────────────────────────────────────────────────────

/// Metadata for a native per-collection snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeSnapshotInfo {
    /// Unique snapshot identifier (timestamp-based).
    pub id: String,
    /// Collection name this snapshot was taken from.
    pub collection: String,
    /// Wall-clock creation time.
    pub created_at: chrono::DateTime<Utc>,
    /// Snapshot file size in bytes.
    pub size_bytes: u64,
}

impl VectorStore {
    /// Return the directory used to store native per-collection snapshots.
    ///
    /// Layout: `<data_dir>/collection_snapshots/<collection_name>/`
    fn native_snapshot_dir(data_dir: &std::path::Path, collection_name: &str) -> PathBuf {
        data_dir.join("collection_snapshots").join(collection_name)
    }

    /// Create a native snapshot of `collection_name`.
    ///
    /// The snapshot is a gzip-compressed JSON file containing all vectors and
    /// metadata — the same format produced by `VectorStore::save`. The file
    /// name encodes the UTC timestamp so it is both unique and sortable.
    ///
    /// Returns [`NativeSnapshotInfo`] on success.
    pub fn snapshot_collection_native(&self, collection_name: &str) -> Result<NativeSnapshotInfo> {
        use std::fs::File;
        use std::io::Write;

        use flate2::Compression;
        use flate2::write::GzEncoder;

        let canonical = self.resolve_alias_target(collection_name)?;
        let coll_ref = self.get_collection(canonical.as_str())?;

        let metadata = coll_ref.metadata();
        let vectors: Vec<crate::persistence::PersistedVector> = coll_ref
            .get_all_vectors()
            .into_iter()
            .map(crate::persistence::PersistedVector::from)
            .collect();

        let persisted = crate::persistence::PersistedVectorStore {
            version: 1,
            collections: vec![crate::persistence::PersistedCollection {
                name: canonical.clone(),
                config: Some(metadata.config),
                vectors,
                hnsw_dump_basename: None,
            }],
        };

        let json = serde_json::to_string(&persisted)
            .map_err(|e| VectorizerError::Serialization(e.to_string()))?;

        let data_dir = Self::get_data_dir();
        let snap_dir = Self::native_snapshot_dir(&data_dir, &canonical);
        std::fs::create_dir_all(&snap_dir).map_err(VectorizerError::Io)?;

        let now = Utc::now();
        let snap_id = now.format("%Y%m%dT%H%M%SZ").to_string();
        let file_path = snap_dir.join(format!("{}.vecdb.gz", snap_id));

        let file = File::create(&file_path).map_err(VectorizerError::Io)?;
        let mut encoder = GzEncoder::new(file, Compression::best());
        encoder
            .write_all(json.as_bytes())
            .map_err(VectorizerError::Io)?;
        encoder.finish().map_err(VectorizerError::Io)?;

        let size_bytes = std::fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);

        info!(
            "native snapshot '{}' for collection '{}': {} bytes",
            snap_id, canonical, size_bytes
        );

        Ok(NativeSnapshotInfo {
            id: snap_id,
            collection: canonical,
            created_at: now,
            size_bytes,
        })
    }

    /// List all native snapshots for `collection_name` (newest first).
    pub fn list_native_snapshots(&self, collection_name: &str) -> Result<Vec<NativeSnapshotInfo>> {
        let canonical = self.resolve_alias_target(collection_name)?;
        let data_dir = Self::get_data_dir();
        let snap_dir = Self::native_snapshot_dir(&data_dir, &canonical);

        if !snap_dir.exists() {
            return Ok(Vec::new());
        }

        let mut infos: Vec<NativeSnapshotInfo> = std::fs::read_dir(&snap_dir)
            .map_err(VectorizerError::Io)?
            .filter_map(|e| e.ok())
            .filter_map(|entry| {
                let path = entry.path();
                let name = path.file_name()?.to_str()?.to_string();
                let id = name.strip_suffix(".vecdb.gz")?.to_string();
                let size_bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                // Parse timestamp from id (YYYYmmddTHHMMSSZ)
                let created_at = chrono::NaiveDateTime::parse_from_str(&id, "%Y%m%dT%H%M%SZ")
                    .ok()
                    .map(|ndt| ndt.and_utc())
                    .unwrap_or_else(|| Utc::now());
                Some(NativeSnapshotInfo {
                    id,
                    collection: canonical.clone(),
                    created_at,
                    size_bytes,
                })
            })
            .collect();

        infos.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(infos)
    }

    /// Restore a collection from a native snapshot.
    ///
    /// Deletes the current in-memory collection (if it exists) and replaces it
    /// with the snapshot data. Safe to call on a non-existent collection (e.g.
    /// after a `delete_collection`).
    pub fn restore_native_snapshot(&self, collection_name: &str, snapshot_id: &str) -> Result<()> {
        use std::io::Read;

        use flate2::read::GzDecoder;

        let canonical = self.resolve_alias_target(collection_name)?;
        let data_dir = Self::get_data_dir();
        let snap_dir = Self::native_snapshot_dir(&data_dir, &canonical);
        let file_path = snap_dir.join(format!("{}.vecdb.gz", snapshot_id));

        if !file_path.exists() {
            return Err(VectorizerError::NotFound(format!(
                "native snapshot '{}' not found for collection '{}'",
                snapshot_id, collection_name
            )));
        }

        let file = std::fs::File::open(&file_path).map_err(VectorizerError::Io)?;
        let mut decoder = GzDecoder::new(file);
        let mut json = String::new();
        decoder
            .read_to_string(&mut json)
            .map_err(VectorizerError::Io)?;

        let persisted: crate::persistence::PersistedVectorStore = serde_json::from_str(&json)?;

        let pc = persisted.collections.into_iter().next().ok_or_else(|| {
            VectorizerError::Storage("snapshot contains no collections".to_string())
        })?;

        // Drop existing collection if present (ignore not-found errors).
        let _ = self.delete_collection(&canonical);

        let config = pc.config.unwrap_or_default();
        self.create_collection_with_quantization(&canonical, config)?;
        if !pc.vectors.is_empty() {
            self.load_collection_from_cache(&canonical, pc.vectors)?;
        }

        info!(
            "restored collection '{}' from native snapshot '{}'",
            canonical, snapshot_id
        );
        Ok(())
    }

    /// Rebuild the HNSW index for `collection_name` with new HNSW parameters.
    ///
    /// Delegates to [`Collection::reindex_with_params`]; non-Cpu variants
    /// return an appropriate error.
    pub fn reindex_collection(
        &self,
        collection_name: &str,
        new_params: crate::models::HnswConfig,
    ) -> Result<()> {
        let coll_ref = self.get_collection(collection_name)?;
        match &*coll_ref {
            CollectionType::Cpu(c) => c.reindex_with_params(new_params),
            CollectionType::Sharded(_) => Err(VectorizerError::Storage(
                "reindex is not supported on sharded collections".to_string(),
            )),
            CollectionType::DistributedSharded(_) => Err(VectorizerError::Storage(
                "reindex is not supported on distributed collections".to_string(),
            )),
            #[allow(unreachable_patterns)]
            _ => Err(VectorizerError::Storage(
                "reindex is not supported on this collection type".to_string(),
            )),
        }
    }
}
