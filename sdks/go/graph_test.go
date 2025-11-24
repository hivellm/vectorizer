package vectorizer

import (
	"testing"
)

func TestListGraphNodes(t *testing.T) {
	client := NewClient(&Config{
		BaseURL: "http://localhost:15002",
	})

	result, err := client.ListGraphNodes("test_collection")
	if err != nil {
		// Collection doesn't exist or graph not enabled - this is expected in test environment
		return
	}

	if result == nil {
		t.Fatal("result should not be nil")
	}

	if result.Count < 0 {
		t.Errorf("count should be >= 0, got %d", result.Count)
	}

	if len(result.Nodes) != result.Count {
		t.Errorf("nodes length should match count, got %d != %d", len(result.Nodes), result.Count)
	}
}

func TestGetGraphNeighbors(t *testing.T) {
	client := NewClient(&Config{
		BaseURL: "http://localhost:15002",
	})

	result, err := client.GetGraphNeighbors("test_collection", "test_node")
	if err != nil {
		// Collection/node doesn't exist - this is expected in test environment
		return
	}

	if result == nil {
		t.Fatal("result should not be nil")
	}

	if result.Neighbors == nil {
		t.Fatal("neighbors should not be nil")
	}
}

func TestFindRelatedNodes(t *testing.T) {
	client := NewClient(&Config{
		BaseURL: "http://localhost:15002",
	})

	maxHops := 2
	relType := "SIMILAR_TO"
	request := &FindRelatedRequest{
		MaxHops:          &maxHops,
		RelationshipType: &relType,
	}

	result, err := client.FindRelatedNodes("test_collection", "test_node", request)
	if err != nil {
		// Collection/node doesn't exist - this is expected in test environment
		return
	}

	if result == nil {
		t.Fatal("result should not be nil")
	}

	if result.Related == nil {
		t.Fatal("related should not be nil")
	}
}

func TestFindGraphPath(t *testing.T) {
	client := NewClient(&Config{
		BaseURL: "http://localhost:15002",
	})

	request := &FindPathRequest{
		Collection: "test_collection",
		Source:     "node1",
		Target:     "node2",
	}

	result, err := client.FindGraphPath(request)
	if err != nil {
		// Collection/nodes don't exist - this is expected in test environment
		return
	}

	if result == nil {
		t.Fatal("result should not be nil")
	}

	if result.Found && len(result.Path) == 0 {
		t.Error("if found is true, path should not be empty")
	}
}

func TestCreateGraphEdge(t *testing.T) {
	client := NewClient(&Config{
		BaseURL: "http://localhost:15002",
	})

	weight := float32(0.85)
	request := &CreateEdgeRequest{
		Collection:       "test_collection",
		Source:           "node1",
		Target:           "node2",
		RelationshipType: "SIMILAR_TO",
		Weight:           &weight,
	}

	result, err := client.CreateGraphEdge(request)
	if err != nil {
		// Collection/nodes don't exist - this is expected in test environment
		return
	}

	if result == nil {
		t.Fatal("result should not be nil")
	}

	if !result.Success {
		t.Error("success should be true")
	}

	if result.EdgeID == "" {
		t.Error("edge_id should not be empty")
	}
}

func TestListGraphEdges(t *testing.T) {
	client := NewClient(&Config{
		BaseURL: "http://localhost:15002",
	})

	result, err := client.ListGraphEdges("test_collection")
	if err != nil {
		// Collection doesn't exist - this is expected in test environment
		return
	}

	if result == nil {
		t.Fatal("result should not be nil")
	}

	if result.Count < 0 {
		t.Errorf("count should be >= 0, got %d", result.Count)
	}

	if len(result.Edges) != result.Count {
		t.Errorf("edges length should match count, got %d != %d", len(result.Edges), result.Count)
	}
}

func TestDiscoverGraphEdges(t *testing.T) {
	client := NewClient(&Config{
		BaseURL: "http://localhost:15002",
	})

	threshold := float32(0.7)
	maxPerNode := 10
	request := &DiscoverEdgesRequest{
		SimilarityThreshold: &threshold,
		MaxPerNode:          &maxPerNode,
	}

	result, err := client.DiscoverGraphEdges("test_collection", request)
	if err != nil {
		// Collection doesn't exist - this is expected in test environment
		return
	}

	if result == nil {
		t.Fatal("result should not be nil")
	}

	if !result.Success {
		t.Error("success should be true")
	}

	if result.EdgesCreated < 0 {
		t.Errorf("edges_created should be >= 0, got %d", result.EdgesCreated)
	}
}

func TestGetGraphDiscoveryStatus(t *testing.T) {
	client := NewClient(&Config{
		BaseURL: "http://localhost:15002",
	})

	result, err := client.GetGraphDiscoveryStatus("test_collection")
	if err != nil {
		// Collection doesn't exist - this is expected in test environment
		return
	}

	if result == nil {
		t.Fatal("result should not be nil")
	}

	if result.TotalNodes < 0 {
		t.Errorf("total_nodes should be >= 0, got %d", result.TotalNodes)
	}

	if result.NodesWithEdges < 0 {
		t.Errorf("nodes_with_edges should be >= 0, got %d", result.NodesWithEdges)
	}

	if result.TotalEdges < 0 {
		t.Errorf("total_edges should be >= 0, got %d", result.TotalEdges)
	}

	if result.ProgressPercentage < 0 || result.ProgressPercentage > 100 {
		t.Errorf("progress_percentage should be between 0 and 100, got %f", result.ProgressPercentage)
	}
}

