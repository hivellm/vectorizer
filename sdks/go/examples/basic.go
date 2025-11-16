package main

import (
	"fmt"
	"log"

	"github.com/hivellm/vectorizer-go"
)

func main() {
	// Create client
	client := vectorizer.NewClient(&vectorizer.Config{
		BaseURL: "http://localhost:15002",
		APIKey:  "your-api-key",
	})

	// Check health
	if err := client.Health(); err != nil {
		log.Fatalf("Health check failed: %v", err)
	}
	fmt.Println("✓ Server is healthy")

	// Create collection
	collection, err := client.CreateCollection(&vectorizer.CreateCollectionRequest{
		Name: "documents",
		Config: &vectorizer.CollectionConfig{
			Dimension: 384,
			Metric:    vectorizer.MetricCosine,
		},
	})
	if err != nil {
		log.Fatalf("Failed to create collection: %v", err)
	}
	fmt.Printf("✓ Created collection: %s\n", collection.Name)

	// Insert text
	result, err := client.InsertText("documents", "Hello, world!", nil)
	if err != nil {
		log.Fatalf("Failed to insert text: %v", err)
	}
	fmt.Printf("✓ Inserted vector ID: %s\n", result.ID)

	// Search
	results, err := client.SearchText("documents", "hello", &vectorizer.SearchOptions{
		Limit: 10,
	})
	if err != nil {
		log.Fatalf("Failed to search: %v", err)
	}
	fmt.Printf("✓ Found %d results\n", len(results))

	// List collections
	collections, err := client.ListCollections()
	if err != nil {
		log.Fatalf("Failed to list collections: %v", err)
	}
	fmt.Printf("✓ Collections: %v\n", collections)
}

