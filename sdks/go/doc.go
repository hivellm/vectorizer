// Package vectorizer provides a Go client library for the Hive Vectorizer vector database.
//
// Quick Start:
//
//	client := vectorizer.NewClient(&vectorizer.Config{
//		BaseURL: "http://localhost:15002",
//		APIKey:  "your-api-key",
//	})
//
//	// Create collection
//	collection, err := client.CreateCollection(&vectorizer.CreateCollectionRequest{
//		Name: "documents",
//		Config: &vectorizer.CollectionConfig{
//			Dimension: 384,
//			Metric:    vectorizer.MetricCosine,
//		},
//	})
//
//	// Insert text
//	result, err := client.InsertText("documents", "Hello, world!", nil)
//
//	// Search
//	results, err := client.SearchText("documents", "hello", &vectorizer.SearchOptions{
//		Limit: 10,
//	})
package vectorizer

