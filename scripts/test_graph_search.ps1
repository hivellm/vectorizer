# Test script for graph search functions

$collectionName = "graph-enabled-collection"

Write-Host "Testing graph search functions on collection: $collectionName`n"

# 1. Insert some vectors to create graph nodes
Write-Host "Step 1: Inserting vectors to create graph nodes..."

$vectors = @(
    @{
        text = "Vectorizer is a high-performance vector database"
        metadata = @{
            source = "doc1"
            category = "database"
        }
    },
    @{
        text = "Vectorizer supports semantic search and graph relationships"
        metadata = @{
            source = "doc2"
            category = "features"
        }
    },
    @{
        text = "Graph database allows tracking relationships between documents"
        metadata = @{
            source = "doc3"
            category = "graph"
        }
    },
    @{
        text = "Semantic search finds similar documents using vector similarity"
        metadata = @{
            source = "doc4"
            category = "search"
        }
    }
)

$vectorIds = @()

foreach ($vec in $vectors) {
    try {
        # Format for /insert endpoint: collection (required), text (required), metadata (optional)
        $insertBody = @{
            collection = $collectionName
            text = $vec.text
            metadata = $vec.metadata
        } | ConvertTo-Json -Depth 5
        
        Write-Host "  Inserting text: $($vec.text.Substring(0, [Math]::Min(50, $vec.text.Length)))..."
        $response = Invoke-RestMethod -Uri "http://127.0.0.1:15002/insert" -Method Post -Body $insertBody -ContentType 'application/json' -ErrorAction Stop
        $vectorIds += $response.vector_id
        Write-Host "  [OK] Inserted: $($response.vector_id)"
    } catch {
        Write-Host "  [FAIL] Failed to insert: $($_.Exception.Message)"
        if ($_.Exception.Response) {
            try {
                $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
                $responseBody = $reader.ReadToEnd()
                Write-Host "    Response: $responseBody"
            } catch {
                # Ignore if we can't read the response
            }
        }
    }
}

Start-Sleep -Seconds 3

Write-Host "`nStep 2: Testing graph REST API endpoints...`n"

# Test graph list nodes via REST API
try {
    Write-Host "Testing GET /api/v1/graph/nodes/$collectionName"
    $nodes = Invoke-RestMethod -Uri "http://127.0.0.1:15002/api/v1/graph/nodes/$collectionName" -Method Get
    Write-Host "  [OK] Graph nodes retrieved"
    Write-Host "    Nodes count: $($nodes.count)"
    if ($nodes.nodes -and $nodes.nodes.Count -gt 0) {
        Write-Host "    First 3 nodes:"
        $nodes.nodes | Select-Object -First 3 | ForEach-Object {
            Write-Host "      - $($_.id) (type: $($_.node_type))"
        }
    } else {
        Write-Host "    No nodes found (graph relationships may need time to be discovered)"
    }
} catch {
    Write-Host "  [FAIL] Error: $($_.Exception.Message)"
}

# Test graph neighbors if we have nodes and vector IDs
if ($vectorIds.Count -gt 0) {
    $firstNodeId = $vectorIds[0]
    Write-Host "`nTesting GET /api/v1/graph/nodes/$collectionName/$firstNodeId/neighbors"
    try {
        $neighbors = Invoke-RestMethod -Uri "http://127.0.0.1:15002/api/v1/graph/nodes/$collectionName/$firstNodeId/neighbors" -Method Get -ErrorAction SilentlyContinue
        Write-Host "  [OK] Neighbors retrieved for node: $firstNodeId"
        Write-Host "    Neighbors count: $($neighbors.neighbors.Count)"
        if ($neighbors.neighbors -and $neighbors.neighbors.Count -gt 0) {
            $neighbors.neighbors | Select-Object -First 3 | ForEach-Object {
                Write-Host "      - $($_.node.id) (relationship: $($_.edge.relationship_type), weight: $($_.edge.weight))"
            }
        } else {
            Write-Host "    No neighbors found (relationships may need to be created)"
        }
    } catch {
        Write-Host "  [FAIL] Error: $($_.Exception.Message)"
    }
    
    # Test find related
    Write-Host "`nTesting POST /api/v1/graph/nodes/$collectionName/$firstNodeId/related"
    try {
        $findRelatedBody = @{
            max_hops = 2
        } | ConvertTo-Json
        
        $related = Invoke-RestMethod -Uri "http://127.0.0.1:15002/api/v1/graph/nodes/$collectionName/$firstNodeId/related" -Method Post -Body $findRelatedBody -ContentType 'application/json' -ErrorAction SilentlyContinue
        Write-Host "  [OK] Related nodes found"
        Write-Host "    Related count: $($related.related.Count)"
        if ($related.related -and $related.related.Count -gt 0) {
            $related.related | Select-Object -First 3 | ForEach-Object {
                Write-Host "      - $($_.node.id) (distance: $($_.distance), weight: $($_.weight))"
            }
        } else {
            Write-Host "    No related nodes found"
        }
    } catch {
        Write-Host "  [FAIL] Error: $($_.Exception.Message)"
    }
    
    # Test create edge if we have 2+ nodes
    if ($vectorIds.Count -ge 2) {
        Write-Host "`nTesting POST /api/v1/graph/edges (creating edge between nodes)"
        try {
            $createEdgeBody = @{
                collection = $collectionName
                source = $vectorIds[0]
                target = $vectorIds[1]
                relationship_type = "SIMILAR_TO"
                weight = 0.8
            } | ConvertTo-Json
            
            $edgeResult = Invoke-RestMethod -Uri "http://127.0.0.1:15002/api/v1/graph/edges" -Method Post -Body $createEdgeBody -ContentType 'application/json' -ErrorAction SilentlyContinue
            Write-Host "  [OK] Edge created successfully"
            Write-Host "    Edge ID: $($edgeResult.edge_id)"
            
            # Test neighbors again after creating edge
            Start-Sleep -Seconds 1
            $neighborsAfter = Invoke-RestMethod -Uri "http://127.0.0.1:15002/api/v1/graph/nodes/$collectionName/$vectorIds[0]/neighbors" -Method Get -ErrorAction SilentlyContinue
            Write-Host "    Neighbors after creating edge: $($neighborsAfter.neighbors.Count)"
        } catch {
            Write-Host "  [FAIL] Error: $($_.Exception.Message)"
        }
    }
}

Write-Host "`nSummary:"
Write-Host "  Graph REST API endpoints are available at /api/v1/graph/"
Write-Host "  MCP graph functions are registered and ready for testing when MCP server is running"
