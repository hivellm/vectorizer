// RPC quickstart example for the Vectorizer Go SDK (v3.x default).
//
// Connects to a server on 127.0.0.1:15503 (the default RPC port),
// does the HELLO handshake, lists collections, and runs a search
// against the first one.
//
// Run a Vectorizer server with RPC enabled (the v3.x default config
// does this automatically), then:
//
//	cd sdks/go
//	go run ./examples/rpc_quickstart
//
// Or with a custom URL:
//
//	VECTORIZER_URL=vectorizer://my-host:15503 go run ./examples/rpc_quickstart
package main

import (
	"context"
	"fmt"
	"log"
	"os"
	"time"

	"github.com/hivellm/vectorizer-sdk-go/rpc"
)

func main() {
	url := os.Getenv("VECTORIZER_URL")
	if url == "" {
		url = "vectorizer://127.0.0.1:15503"
	}
	fmt.Printf("→ Dialing %s\n", url)

	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	client, err := rpc.ConnectURL(ctx, url, rpc.ConnectOptions{})
	if err != nil {
		log.Fatalf("connect: %v", err)
	}
	defer client.Close()

	// HELLO handshake. In single-user mode (auth.enabled: false on
	// the server side), credentials are accepted-but-ignored. When
	// auth is enabled, attach a JWT or API key:
	//   rpc.HelloPayload{ClientName: "...", Token: "<jwt>"}
	hello, err := client.Hello(ctx, rpc.HelloPayload{ClientName: "rpc-quickstart"})
	if err != nil {
		log.Fatalf("hello: %v", err)
	}
	fmt.Printf("✓ HELLO ok — server=%s  protocol_version=%d  authenticated=%v  admin=%v\n",
		hello.ServerVersion, hello.ProtocolVersion, hello.Authenticated, hello.Admin)
	fmt.Printf("  capabilities: %v\n", hello.Capabilities)

	// PING (auth-exempt — works pre-HELLO too).
	pong, err := client.Ping(ctx)
	if err != nil {
		log.Fatalf("ping: %v", err)
	}
	fmt.Printf("✓ PING → %s\n", pong)

	collections, err := client.ListCollections(ctx)
	if err != nil {
		log.Fatalf("list collections: %v", err)
	}
	fmt.Printf("✓ %d collection(s): %v\n", len(collections), collections)

	if len(collections) == 0 {
		fmt.Println("  (no collections to search — create one via REST/MCP or the dashboard)")
		return
	}

	first := collections[0]
	fmt.Printf("→ Searching '%s' for 'vector database'\n", first)

	info, err := client.GetCollectionInfo(ctx, first)
	if err != nil {
		log.Fatalf("get_collection_info: %v", err)
	}
	fmt.Printf("  collection has %d vectors across %d documents (dim=%d)\n",
		info.VectorCount, info.DocumentCount, info.Dimension)

	hits, err := client.SearchBasic(ctx, first, "vector database", 5)
	if err != nil {
		log.Fatalf("search.basic: %v", err)
	}
	fmt.Printf("  top %d hit(s):\n", len(hits))
	for _, hit := range hits {
		fmt.Printf("    %s (score=%.4f)\n", hit.ID, hit.Score)
	}
}
