//! Turns committed store changes into replication operations on the leader.
//!
//! Registered on the store only while this node is the master, so every write
//! path replicates — not just the handlers that used to call
//! [`MasterNode::replicate`] themselves. Replicas apply operations through
//! the same store methods, but carry no publisher, so nothing is re-sent.

use std::sync::Arc;

use super::MasterNode;
use super::types::{CollectionConfigData, VectorOperation};
use crate::db::{MutationListener, StoreMutation};
use crate::models::DistanceMetric;

/// Forwards every committed store change to a [`MasterNode`].
pub struct ReplicationPublisher {
    master: Arc<MasterNode>,
}

impl ReplicationPublisher {
    /// Publish to `master`.
    pub fn new(master: Arc<MasterNode>) -> Self {
        Self { master }
    }
}

impl MutationListener for ReplicationPublisher {
    fn on_mutation(&self, mutation: &StoreMutation<'_>) {
        for operation in operations_for(mutation) {
            self.master.replicate(operation);
        }
    }
}

/// The replication operations that reproduce `mutation` on a replica.
///
/// Vector payloads travel as the JSON of the stored payload and `owner_id`
/// is only carried on collection creation, matching what replicas apply.
pub fn operations_for(mutation: &StoreMutation<'_>) -> Vec<VectorOperation> {
    match mutation {
        StoreMutation::CollectionCreated {
            name,
            config,
            owner_id,
        } => vec![VectorOperation::CreateCollection {
            name: (*name).to_string(),
            config: CollectionConfigData {
                dimension: config.dimension,
                metric: metric_name(&config.metric).to_string(),
            },
            owner_id: owner_id.map(|id| id.to_string()),
        }],
        StoreMutation::CollectionDeleted { name } => vec![VectorOperation::DeleteCollection {
            name: (*name).to_string(),
            owner_id: None,
        }],
        StoreMutation::CollectionRenamed { from, to } => {
            vec![VectorOperation::RenameCollection {
                old_name: (*from).to_string(),
                new_name: (*to).to_string(),
            }]
        }
        StoreMutation::VectorsUpserted {
            collection,
            vectors,
        } => vectors
            .iter()
            .map(|vector| VectorOperation::InsertVector {
                collection: (*collection).to_string(),
                id: vector.id.clone(),
                vector: vector.data.clone(),
                payload: vector
                    .payload
                    .as_ref()
                    .and_then(|payload| serde_json::to_vec(payload).ok()),
                owner_id: None,
            })
            .collect(),
        StoreMutation::VectorDeleted { collection, id } => vec![VectorOperation::DeleteVector {
            collection: (*collection).to_string(),
            id: (*id).to_string(),
            owner_id: None,
        }],
    }
}

/// The metric spelling replicas parse back.
fn metric_name(metric: &DistanceMetric) -> &'static str {
    match metric {
        DistanceMetric::Cosine => "cosine",
        DistanceMetric::Euclidean => "euclidean",
        DistanceMetric::DotProduct => "dot_product",
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::models::{CollectionConfig, Payload, Vector};

    #[test]
    fn upsert_carries_every_vector_and_its_payload() {
        let vectors = vec![
            Vector {
                id: "a".into(),
                data: vec![1.0, 0.0],
                sparse: None,
                payload: Some(Payload {
                    data: serde_json::json!({"k": "v"}),
                }),
                document_id: None,
            },
            Vector {
                id: "b".into(),
                data: vec![0.0, 1.0],
                sparse: None,
                payload: None,
                document_id: None,
            },
        ];
        let ops = operations_for(&StoreMutation::VectorsUpserted {
            collection: "c",
            vectors: &vectors,
        });
        assert_eq!(ops.len(), 2);
        match &ops[0] {
            VectorOperation::InsertVector {
                collection,
                id,
                vector,
                payload,
                ..
            } => {
                assert_eq!(collection, "c");
                assert_eq!(id, "a");
                assert_eq!(vector, &vec![1.0, 0.0]);
                let decoded: serde_json::Value =
                    serde_json::from_slice(payload.as_ref().unwrap()).unwrap();
                assert_eq!(decoded, serde_json::json!({"k": "v"}));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn deletes_and_collection_changes_map_one_to_one() {
        let config = CollectionConfig {
            dimension: 4,
            metric: DistanceMetric::DotProduct,
            ..Default::default()
        };
        let owner = uuid::Uuid::new_v4();
        let created = operations_for(&StoreMutation::CollectionCreated {
            name: "c",
            config: &config,
            owner_id: Some(owner),
        });
        assert!(matches!(
            &created[..],
            [VectorOperation::CreateCollection { name, config, owner_id }]
                if name == "c" && config.dimension == 4 && config.metric == "dot_product"
                    && owner_id.as_deref() == Some(owner.to_string().as_str())
        ));

        assert!(matches!(
            &operations_for(&StoreMutation::VectorDeleted { collection: "c", id: "a" })[..],
            [VectorOperation::DeleteVector { collection, id, .. }] if collection == "c" && id == "a"
        ));
        assert!(matches!(
            &operations_for(&StoreMutation::CollectionDeleted { name: "c" })[..],
            [VectorOperation::DeleteCollection { name, .. }] if name == "c"
        ));
        assert!(matches!(
            &operations_for(&StoreMutation::CollectionRenamed { from: "c", to: "d" })[..],
            [VectorOperation::RenameCollection { old_name, new_name }]
                if old_name == "c" && new_name == "d"
        ));
    }
}
