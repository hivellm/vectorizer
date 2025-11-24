use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::error::{Result, VectorizerError};
use crate::models::{Payload, SparseVector, Vector};
use crate::storage::mmap::MmapVectorStorage;

/// Abstract vector storage backend
#[derive(Clone, Debug)]
pub enum VectorStorageBackend {
    /// In-memory HashMap storage
    Memory(Arc<RwLock<HashMap<String, Vector>>>),
    /// Memory-mapped file storage for dense vectors, memory for others
    Mmap {
        storage: Arc<RwLock<MmapVectorStorage>>,
        id_map: Arc<RwLock<HashMap<String, usize>>>,
        payloads: Arc<RwLock<HashMap<String, Payload>>>,
        sparse: Arc<RwLock<HashMap<String, SparseVector>>>,
    },
}

impl VectorStorageBackend {
    pub fn new_memory() -> Self {
        Self::Memory(Arc::new(RwLock::new(HashMap::new())))
    }

    pub fn new_mmap(storage: MmapVectorStorage) -> Self {
        Self::Mmap {
            storage: Arc::new(RwLock::new(storage)),
            id_map: Arc::new(RwLock::new(HashMap::new())),
            payloads: Arc::new(RwLock::new(HashMap::new())),
            sparse: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn insert(&self, id: String, vector: Vector) -> Result<()> {
        match self {
            Self::Memory(map) => {
                map.write().insert(id, vector);
                Ok(())
            }
            Self::Mmap {
                storage,
                id_map,
                payloads,
                sparse,
            } => {
                let mut storage_guard = storage.write();
                let idx = storage_guard.append(&vector.data)?;
                id_map.write().insert(id.clone(), idx);

                if let Some(payload) = vector.payload {
                    payloads.write().insert(id.clone(), payload);
                }

                if let Some(sp) = vector.sparse {
                    sparse.write().insert(id, sp);
                }

                Ok(())
            }
        }
    }

    pub fn get(&self, id: &str) -> Result<Option<Vector>> {
        match self {
            Self::Memory(map) => Ok(map.read().get(id).cloned()),
            Self::Mmap {
                storage,
                id_map,
                payloads,
                sparse,
            } => {
                let idx = {
                    let map = id_map.read();
                    match map.get(id) {
                        Some(&idx) => idx,
                        None => return Ok(None),
                    }
                };

                let data = storage.read().get(idx)?;
                let payload = payloads.read().get(id).cloned();
                let sparse_vector = sparse.read().get(id).cloned();

                Ok(Some(Vector {
                    id: id.to_string(),
                    data,
                    payload,
                    sparse: sparse_vector,
                }))
            }
        }
    }

    pub fn contains_key(&self, id: &str) -> Result<bool> {
        match self {
            Self::Memory(map) => Ok(map.read().contains_key(id)),
            Self::Mmap { id_map, .. } => Ok(id_map.read().contains_key(id)),
        }
    }

    pub fn update(&self, id: &str, vector: Vector) -> Result<()> {
        match self {
            Self::Memory(map) => {
                if map.read().contains_key(id) {
                    map.write().insert(id.to_string(), vector);
                    Ok(())
                } else {
                    Err(VectorizerError::VectorNotFound(id.to_string()))
                }
            }
            Self::Mmap {
                storage,
                id_map,
                payloads,
                sparse,
            } => {
                let idx = {
                    let map = id_map.read();
                    match map.get(id) {
                        Some(&idx) => idx,
                        None => return Err(VectorizerError::VectorNotFound(id.to_string())),
                    }
                };

                let mut storage_guard = storage.write();
                storage_guard.update(idx, &vector.data)?;

                if let Some(payload) = vector.payload {
                    payloads.write().insert(id.to_string(), payload);
                } else {
                    payloads.write().remove(id);
                }

                if let Some(sp) = vector.sparse {
                    sparse.write().insert(id.to_string(), sp);
                } else {
                    sparse.write().remove(id);
                }

                Ok(())
            }
        }
    }

    pub fn remove(&self, id: &str) -> Result<bool> {
        match self {
            Self::Memory(map) => Ok(map.write().remove(id).is_some()),
            Self::Mmap {
                id_map,
                payloads,
                sparse,
                ..
            } => {
                let removed = id_map.write().remove(id).is_some();
                if removed {
                    payloads.write().remove(id);
                    sparse.write().remove(id);
                }
                Ok(removed)
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Memory(map) => map.read().len(),
            Self::Mmap { id_map, .. } => id_map.read().len(), // Use id_map len for active count
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
