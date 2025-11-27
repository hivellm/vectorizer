package main

import (
	"fmt"

	"github.com/hivellm/vectorizer-sdk-go"
)

func main() {
	fmt.Println("🐹 Vectorizer Go SDK Basic Example")
	fmt.Println("==================================")

	// Create client
	client := vectorizer.NewClient(&vectorizer.Config{
		BaseURL: "http://localhost:15002",
		APIKey:  "your-api-key",
	})
	fmt.Println("✅ Client created successfully")

	collectionName := "example-documents"

	// Health check
	fmt.Println("\n🔍 Checking server health...")
	if err := client.Health(); err != nil {
		fmt.Printf("⚠️ Health check failed: %v\n", err)
	} else {
		fmt.Println("✅ Server is healthy")
	}

	// Get database stats
	fmt.Println("\n📊 Getting database statistics...")
	stats, err := client.GetStats()
	if err != nil {
		fmt.Printf("⚠️ Get stats failed: %v\n", err)
	} else {
		fmt.Printf("📈 Database stats:\n")
		fmt.Printf("   Collections: %d\n", stats.Collections)
		fmt.Printf("   Vectors: %d\n", stats.Vectors)
	}

	// List existing collections
	fmt.Println("\n📋 Listing collections...")
	collections, err := client.ListCollections()
	if err != nil {
		fmt.Printf("⚠️ Error listing collections: %v\n", err)
	} else {
		fmt.Printf("📁 Found %d collections:\n", len(collections))
		for i, name := range collections {
			if i >= 5 {
				break
			}
			fmt.Printf("   - %s\n", name)
		}
	}

	// Create a new collection
	fmt.Println("\n🆕 Creating collection...")
	collection, err := client.CreateCollection(&vectorizer.CreateCollectionRequest{
		Name: collectionName,
		Config: &vectorizer.CollectionConfig{
			Dimension: 384,
			Metric:    vectorizer.MetricCosine,
		},
	})
	if err != nil {
		fmt.Printf("⚠️ Collection creation failed (may already exist): %v\n", err)
	} else {
		fmt.Printf("✅ Collection created: %s\n", collection.Name)
		if collection.Config != nil {
			fmt.Printf("   Dimension: %d\n", collection.Config.Dimension)
			fmt.Printf("   Metric: %s\n", collection.Config.Metric)
		}
	}

	// Insert texts
	fmt.Println("\n📥 Inserting texts...")
	texts := []struct {
		id       string
		text     string
		metadata map[string]interface{}
	}{
		{
			id:   "doc_1",
			text: "Introduction to Machine Learning",
			metadata: map[string]interface{}{
				"source":   "document1.pdf",
				"title":    "Introduction to Machine Learning",
				"category": "AI",
			},
		},
		{
			id:   "doc_2",
			text: "Deep Learning Fundamentals",
			metadata: map[string]interface{}{
				"source":   "document2.pdf",
				"title":    "Deep Learning Fundamentals",
				"category": "AI",
			},
		},
		{
			id:   "doc_3",
			text: "Data Science Best Practices",
			metadata: map[string]interface{}{
				"source":   "document3.pdf",
				"title":    "Data Science Best Practices",
				"category": "Data",
			},
		},
	}

	inserted := 0
	for _, text := range texts {
		result, err := client.InsertText(collectionName, text.text, text.metadata)
		if err != nil {
			fmt.Printf("⚠️ Insert text failed for %s: %v\n", text.id, err)
		} else {
			fmt.Printf("✅ Inserted text: %s (ID: %s)\n", text.id, result.VectorID)
			inserted++
		}
	}
	fmt.Printf("✅ Total texts inserted: %d\n", inserted)

	// Search for similar vectors
	fmt.Println("\n🔍 Searching for similar vectors...")
	results, err := client.SearchText(collectionName, "machine learning algorithms", &vectorizer.SearchOptions{
		Limit: 3,
	})
	if err != nil {
		fmt.Printf("⚠️ Search failed: %v\n", err)
	} else {
		fmt.Println("🎯 Search results:")
		for i, result := range results {
			fmt.Printf("   %d. Score: %.4f\n", i+1, result.Score)
			if result.Payload != nil {
				if title, ok := result.Payload["title"].(string); ok {
					fmt.Printf("      Title: %s\n", title)
				}
				if category, ok := result.Payload["category"].(string); ok {
					fmt.Printf("      Category: %s\n", category)
				}
			}
		}
	}

	// Get collection info
	fmt.Println("\n📊 Getting collection information...")
	info, err := client.GetCollectionInfo(collectionName)
	if err != nil {
		fmt.Printf("⚠️ Get collection info failed: %v\n", err)
	} else {
		fmt.Println("📈 Collection info:")
		fmt.Printf("   Name: %s\n", info.Name)
		fmt.Printf("   Dimension: %d\n", info.Dimension)
		fmt.Printf("   Vector count: %d\n", info.VectorCount)
		fmt.Printf("   Metric: %s\n", info.Metric)
	}

	fmt.Println("\n🌐 All operations completed successfully!")

	// Clean up
	fmt.Println("\n🧹 Cleaning up...")
	if err := client.DeleteCollection(collectionName); err != nil {
		fmt.Printf("⚠️ Delete collection failed: %v\n", err)
	} else {
		fmt.Println("✅ Collection deleted")
	}

	fmt.Println("\n👋 Example completed!")
}
