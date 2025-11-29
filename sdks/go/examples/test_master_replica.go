package main

import (
	"fmt"
	"time"

	vectorizer "github.com/hivellm/vectorizer-sdk-go"
)

const (
	MASTER_URL  = "http://localhost:15002"
	REPLICA_URL = "http://localhost:17780"
	API_KEY     = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjoiYWRtaW4iLCJ1c2VybmFtZSI6ImFkbWluIiwicm9sZXMiOlsiQWRtaW4iXSwiaWF0IjoxNzY0Mzk0MzI3LCJleHAiOjE3NjQzOTc5Mjd9.AnLPTgdHRfCFMdMp6VFemhcIZPUfpzjwB5r6xOkCxNQ"
)

func main() {
	fmt.Println("=== Go SDK Master/Replica Test ===\n")

	// 1. Test with hosts configuration
	fmt.Println("1. Creating client with hosts configuration...")
	client := vectorizer.NewClient(&vectorizer.Config{
		Hosts: &vectorizer.HostConfig{
			Master:   MASTER_URL,
			Replicas: []string{REPLICA_URL},
		},
		ReadPreference: vectorizer.ReadPreferenceReplica,
		APIKey:         API_KEY,
		Timeout:        30 * time.Second,
	})
	fmt.Println("   Client created with master/replica topology")

	// 2. Test health (read operation - should go to replica)
	fmt.Println("2. Testing Health() (read - should go to replica)...")
	err := client.Health()
	if err != nil {
		fmt.Printf("   Health failed: %v\n", err)
	} else {
		fmt.Println("   Health: OK")
	}

	// 3. Test listing collections (read operation)
	fmt.Println("3. Listing collections (read)...")
	collections, err := client.ListCollections()
	if err != nil {
		fmt.Printf("   List failed: %v\n", err)
	} else {
		fmt.Printf("   Found %d collections\n", len(collections))
	}

	// 4. Test WithMaster
	fmt.Println("4. Testing WithMaster()...")
	masterClient := client.WithMaster()
	err = masterClient.Health()
	if err != nil {
		fmt.Printf("   WithMaster health failed: %v\n", err)
	} else {
		fmt.Println("   Master Health: OK")
	}

	// 5. Test backward compatibility with single BaseURL
	fmt.Println("\n5. Testing backward compatibility (single BaseURL)...")
	singleClient := vectorizer.NewClient(&vectorizer.Config{
		BaseURL: MASTER_URL,
		APIKey:  API_KEY,
		Timeout: 30 * time.Second,
	})
	err = singleClient.Health()
	if err != nil {
		fmt.Printf("   Single URL failed: %v\n", err)
	} else {
		fmt.Println("   Single URL mode works: OK")
	}

	fmt.Println("\n=== Go SDK Test Complete ===")
}
