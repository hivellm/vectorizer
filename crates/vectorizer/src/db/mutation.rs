//! Committed changes to a [`VectorStore`](crate::db::VectorStore), published
//! to listeners.
//!
//! Every write path — REST, RPC, MCP, GraphQL, gRPC — ends in a handful of
//! store methods. Publishing from there, after the change is committed, lets
//! replication and auto-save observe all of them without each handler having
//! to remember to. Before this, only four handler paths replicated: deletes,
//! updates and collection drops never reached followers, and followers never
//! persisted what they received.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use parking_lot::RwLock;

use crate::models::{CollectionConfig, Vector};

/// A change the store has committed.
#[derive(Debug)]
pub enum StoreMutation<'a> {
    /// A collection was created.
    CollectionCreated {
        /// Name of the new collection.
        name: &'a str,
        /// Configuration it was created with.
        config: &'a CollectionConfig,
        /// Owning tenant, in multi-tenant mode.
        owner_id: Option<uuid::Uuid>,
    },
    /// A collection was deleted.
    CollectionDeleted {
        /// Canonical name of the deleted collection.
        name: &'a str,
    },
    /// A collection was renamed.
    CollectionRenamed {
        /// Canonical name before the rename.
        from: &'a str,
        /// New name.
        to: &'a str,
    },
    /// Vectors were inserted or replaced, as stored (collection TTL applied).
    VectorsUpserted {
        /// Collection the vectors belong to.
        collection: &'a str,
        /// The vectors as committed.
        vectors: &'a [Vector],
    },
    /// A vector was deleted.
    VectorDeleted {
        /// Collection the vector belonged to.
        collection: &'a str,
        /// Id of the deleted vector.
        id: &'a str,
    },
}

/// Receives [`StoreMutation`]s.
///
/// Called synchronously on the write path, after the change is committed and
/// with no store lock held. Implementations must be cheap — enqueue, flag,
/// or hand off — and must not call back into the store's write methods.
pub trait MutationListener: Send + Sync {
    /// Observe one committed change.
    fn on_mutation(&self, mutation: &StoreMutation<'_>);
}

/// Identifies a registered listener so it can be removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MutationListenerId(u64);

/// The listeners registered on one store. Shared by every clone of it.
#[derive(Default)]
pub struct MutationListeners {
    next_id: AtomicU64,
    listeners: RwLock<Vec<(MutationListenerId, Arc<dyn MutationListener>)>>,
}

impl std::fmt::Debug for MutationListeners {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MutationListeners")
            .field("count", &self.listeners.read().len())
            .finish()
    }
}

impl MutationListeners {
    /// Register `listener`; it sees every mutation committed from now on.
    pub fn add(&self, listener: Arc<dyn MutationListener>) -> MutationListenerId {
        let id = MutationListenerId(self.next_id.fetch_add(1, Ordering::Relaxed));
        self.listeners.write().push((id, listener));
        id
    }

    /// Unregister a listener. Returns whether it was registered.
    pub fn remove(&self, id: MutationListenerId) -> bool {
        let mut listeners = self.listeners.write();
        let before = listeners.len();
        listeners.retain(|(registered, _)| *registered != id);
        listeners.len() != before
    }

    /// Deliver `mutation` to every listener. The registry lock is released
    /// before any listener runs, so a listener may add or remove listeners.
    pub fn publish(&self, mutation: &StoreMutation<'_>) {
        let listeners: Vec<Arc<dyn MutationListener>> = self
            .listeners
            .read()
            .iter()
            .map(|(_, listener)| Arc::clone(listener))
            .collect();
        for listener in listeners {
            listener.on_mutation(mutation);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use parking_lot::Mutex;

    use super::*;

    #[derive(Default)]
    struct Recorder(Mutex<Vec<String>>);

    impl MutationListener for Recorder {
        fn on_mutation(&self, mutation: &StoreMutation<'_>) {
            let entry = match mutation {
                StoreMutation::CollectionCreated { name, .. } => format!("created:{name}"),
                StoreMutation::CollectionDeleted { name } => format!("deleted:{name}"),
                StoreMutation::CollectionRenamed { from, to } => format!("renamed:{from}->{to}"),
                StoreMutation::VectorsUpserted {
                    collection,
                    vectors,
                } => format!("upserted:{collection}:{}", vectors.len()),
                StoreMutation::VectorDeleted { collection, id } => {
                    format!("vector-deleted:{collection}:{id}")
                }
            };
            self.0.lock().push(entry);
        }
    }

    #[test]
    fn removed_listener_stops_receiving() {
        let registry = MutationListeners::default();
        let recorder = Arc::new(Recorder::default());
        let id = registry.add(recorder.clone());

        registry.publish(&StoreMutation::CollectionDeleted { name: "a" });
        assert!(registry.remove(id));
        registry.publish(&StoreMutation::CollectionDeleted { name: "b" });

        assert_eq!(*recorder.0.lock(), vec!["deleted:a".to_string()]);
        assert!(!registry.remove(id), "second remove must report absence");
    }
}
