use crate::{
	db::{HnswIndex, VectorStore},
	models::{vector_utils, CollectionConfig, DistanceMetric, HnswConfig, Payload, Vector},
};
use proptest::prelude::*;

#[test]
fn test_embedding_manager_error_cases() {
	use crate::embedding::{BagOfWordsEmbedding, EmbeddingManager, EmbeddingProvider, TfIdfEmbedding};

	let mut manager = EmbeddingManager::new();

	// No default provider yet
	let err = manager.get_default_provider().err().unwrap();
	let msg = format!("{}", err);
	assert!(msg.contains("No default provider"));

	// Register providers
	manager.register_provider("tfidf".to_string(), Box::new(TfIdfEmbedding::new(8)));
	manager.register_provider("bow".to_string(), Box::new(BagOfWordsEmbedding::new(8)));

	// Setting non-existent default should fail
	let err = manager.set_default_provider("missing").err().unwrap();
	let msg = format!("{}", err);
	assert!(msg.contains("not found"));

	// Set valid default and use it
	manager.set_default_provider("tfidf").unwrap();
	let emb = manager.embed("hello world").unwrap();
	assert_eq!(emb.len(), 8);
}

#[test]
fn test_vector_store_stats_multiple_collections() {
	let store = VectorStore::new();

	let cfg_small = CollectionConfig {
		dimension: 3,
		metric: DistanceMetric::Euclidean,
		hnsw_config: HnswConfig::default(),
		quantization: None,
		compression: Default::default(),
	};
	let cfg_large = CollectionConfig {
		dimension: 64,
		metric: DistanceMetric::Cosine,
		hnsw_config: HnswConfig::default(),
		quantization: None,
		compression: Default::default(),
	};

	store.create_collection("small", cfg_small).unwrap();
	store.create_collection("large", cfg_large).unwrap();

	// Insert a few vectors in each
	store.insert(
		"small",
		vec![
			Vector::new("s1".to_string(), vec![1.0, 0.0, 0.0]),
			Vector::new("s2".to_string(), vec![0.0, 1.0, 0.0]),
		],
	)
	.unwrap();

	store
		.insert(
			"large",
			vec![
				Vector::new("l1".to_string(), vec![0.1; 64]),
				Vector::new("l2".to_string(), vec![0.2; 64]),
				Vector::new("l3".to_string(), vec![0.3; 64]),
			],
		)
		.unwrap();

	let stats = store.stats();
	assert_eq!(stats.collection_count, 2);
	assert_eq!(stats.total_vectors, 5);
	assert!(stats.total_memory_bytes > 0);
}

#[test]
fn test_payload_serialization_nested() {
	let store = VectorStore::new();
	let cfg = CollectionConfig {
		dimension: 4,
		metric: DistanceMetric::Euclidean,
		hnsw_config: HnswConfig::default(),
		quantization: None,
		compression: Default::default(),
	};
	store.create_collection("nested", cfg).unwrap();

	let payload = Payload::from_value(serde_json::json!({
		"meta": {
			"source": "unit_test",
			"tags": ["nested", "json", {"k": "v"}],
			"score": 0.87
		}
	}))
	.unwrap();

	store
		.insert(
			"nested",
			vec![Vector::with_payload(
				"n1".to_string(),
				vec![0.0, 1.0, 2.0, 3.0],
				payload,
			)],
		)
		.unwrap();

	let got = store.get_vector("nested", "n1").unwrap();
	let meta = &got.payload.unwrap().data["meta"];
	assert_eq!(meta["source"], "unit_test");
	assert_eq!(meta["tags"][0], "nested");
	assert_eq!(meta["tags"][2]["k"], "v");
}

proptest! {
	#[test]
	fn prop_normalize_vector_has_unit_norm_nonzero(v in proptest::collection::vec(-10.0f32..10.0f32, 1..64)) {
		let norm_sq: f32 = v.iter().map(|x| x * x).sum();
		prop_assume!(norm_sq > 1e-12);
		let n = vector_utils::normalize_vector(&v);
		let n_sq: f32 = n.iter().map(|x| x * x).sum();
		prop_assert!((n_sq - 1.0).abs() < 5e-4);
	}
}

#[test]
fn test_hnsw_stats_and_rebuild() {
	let mut index = HnswIndex::new(HnswConfig::default(), DistanceMetric::Euclidean, 3);
	index.add("a", &[1.0, 0.0, 0.0]).unwrap();
	index.add("b", &[0.0, 1.0, 0.0]).unwrap();

	let s1 = index.stats();
	assert_eq!(s1.vector_count, 2);
	assert!(!s1.needs_rebuild);
	assert_eq!(s1.dimension, 3);

	// Update marks as needing rebuild
	index.update("a", &[2.0, 0.0, 0.0]).unwrap();
	let s2 = index.stats();
	assert!(s2.needs_rebuild);

	// Rebuild clears flag
	index.rebuild().unwrap();
	let s3 = index.stats();
	assert!(!s3.needs_rebuild);
}
