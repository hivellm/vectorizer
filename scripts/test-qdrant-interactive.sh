#!/bin/bash
# Interactive Qdrant Compatibility Test Script
# Interactive menu-driven testing tool

BASE_URL="${BASE_URL:-http://localhost:15002/qdrant}"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Generate a vector array JSON string
generate_vector() {
    local value=$1
    local dimension=$2
    local vector="["
    for i in $(seq 1 $dimension); do
        if [ $i -gt 1 ]; then
            vector+=","
        fi
        vector+="$value"
    done
    vector+="]"
    echo "$vector"
}

show_menu() {
    echo ""
    echo -e "${BLUE}Qdrant Compatibility Test Menu${NC}"
    echo "1. Test Collection Management"
    echo "2. Test Vector Operations"
    echo "3. Test Search Operations"
    echo "4. Test Filter Operations"
    echo "5. Test Error Handling"
    echo "6. Run All Tests"
    echo "7. Performance Benchmark"
    echo "0. Exit"
    echo ""
    read -p "Select option: " choice
}

test_collections() {
    echo -e "\n${GREEN}Testing Collection Management${NC}"
    
    read -p "Collection name (default: test_collection): " collection_name
    collection_name=${collection_name:-test_collection}
    
    echo "1. Creating collection..."
    curl -X PUT "$BASE_URL/collections/$collection_name" \
        -H "Content-Type: application/json" \
        -d '{"vectors": {"size": 384, "distance": "Cosine"}}' | jq '.'
    
    echo -e "\n2. Getting collection info..."
    curl -s "$BASE_URL/collections/$collection_name" | jq '.'
    
    echo -e "\n3. Listing all collections..."
    curl -s "$BASE_URL/collections" | jq '.result.collections'
}

test_vectors() {
    echo -e "\n${GREEN}Testing Vector Operations${NC}"
    
    read -p "Collection name: " collection_name
    if [ -z "$collection_name" ]; then
        echo "Collection name required"
        return
    fi
    
    echo "1. Upserting points..."
    VECTOR1=$(generate_vector "0.1" 384)
    VECTOR2=$(generate_vector "0.2" 384)
    curl -X PUT "$BASE_URL/collections/$collection_name/points" \
        -H "Content-Type: application/json" \
        -d "{
            \"points\": [
                {\"id\": \"1\", \"vector\": $VECTOR1, \"payload\": {\"text\": \"doc1\", \"status\": \"active\"}},
                {\"id\": \"2\", \"vector\": $VECTOR2, \"payload\": {\"text\": \"doc2\", \"status\": \"inactive\"}}
            ]
        }" | jq '.'
    
    echo -e "\n2. Retrieving points..."
    curl -s "$BASE_URL/collections/$collection_name/points?ids=1,2" | jq '.'
    
    echo -e "\n3. Counting points..."
    curl -X POST "$BASE_URL/collections/$collection_name/points/count" \
        -H "Content-Type: application/json" \
        -d '{}' | jq '.'
}

test_search() {
    echo -e "\n${GREEN}Testing Search Operations${NC}"
    
    read -p "Collection name: " collection_name
    if [ -z "$collection_name" ]; then
        echo "Collection name required"
        return
    fi
    
    VECTOR=$(generate_vector "0.1" 384)
    echo "1. Basic search..."
    curl -X POST "$BASE_URL/collections/$collection_name/points/search" \
        -H "Content-Type: application/json" \
        -d "{\"vector\": $VECTOR, \"limit\": 10}" | jq '.result | length'
    
    echo -e "\n2. Batch search..."
    curl -X POST "$BASE_URL/collections/$collection_name/points/search/batch" \
        -H "Content-Type: application/json" \
        -d "{\"searches\": [{\"vector\": $VECTOR, \"limit\": 5}]}" | jq '.'
}

test_filters() {
    echo -e "\n${GREEN}Testing Filter Operations${NC}"
    
    read -p "Collection name: " collection_name
    if [ -z "$collection_name" ]; then
        echo "Collection name required"
        return
    fi
    
    VECTOR=$(generate_vector "0.1" 384)
    echo "1. Match filter..."
    curl -X POST "$BASE_URL/collections/$collection_name/points/search" \
        -H "Content-Type: application/json" \
        -d "{
            \"vector\": $VECTOR,
            \"filter\": {\"must\": [{\"type\": \"match\", \"key\": \"status\", \"match_value\": \"active\"}]},
            \"limit\": 10
        }" | jq '.result | length'
    
    echo -e "\n2. Range filter..."
    curl -X POST "$BASE_URL/collections/$collection_name/points/search" \
        -H "Content-Type: application/json" \
        -d "{
            \"vector\": $VECTOR,
            \"filter\": {\"must\": [{\"type\": \"range\", \"key\": \"price\", \"range\": {\"gte\": 50}}]},
            \"limit\": 10
        }" | jq '.'
}

test_errors() {
    echo -e "\n${GREEN}Testing Error Handling${NC}"
    
    echo "1. Collection not found (404)..."
    curl -s -w "\nHTTP Status: %{http_code}\n" \
        "$BASE_URL/collections/nonexistent" | jq '.'
    
    echo -e "\n2. Invalid vector dimension (400)..."
    read -p "Collection name: " collection_name
    if [ -n "$collection_name" ]; then
        VECTOR_100=$(generate_vector "0.1" 100)
        curl -X PUT "$BASE_URL/collections/$collection_name/points" \
            -H "Content-Type: application/json" \
            -d "{\"points\": [{\"id\": \"1\", \"vector\": $VECTOR_100}]}" \
            -w "\nHTTP Status: %{http_code}\n" | jq '.'
    fi
}

run_all_tests() {
    echo -e "\n${GREEN}Running All Tests${NC}"
    bash "$(dirname "$0")/test-qdrant-compatibility.sh"
}

benchmark() {
    echo -e "\n${GREEN}Performance Benchmark${NC}"
    
    read -p "Collection name: " collection_name
    read -p "Number of iterations (default: 10): " iterations
    iterations=${iterations:-10}
    
    echo "Running $iterations search iterations..."
    
    VECTOR=$(generate_vector "0.1" 384)
    total_time=0
    for i in $(seq 1 $iterations); do
        start=$(date +%s.%N)
        curl -s -X POST "$BASE_URL/collections/$collection_name/points/search" \
            -H "Content-Type: application/json" \
            -d "{\"vector\": $VECTOR, \"limit\": 10}" > /dev/null
        end=$(date +%s.%N)
        elapsed=$(echo "$end - $start" | bc)
        total_time=$(echo "$total_time + $elapsed" | bc)
        echo -n "."
    done
    
    avg_time=$(echo "scale=3; $total_time / $iterations" | bc)
    echo -e "\nAverage time: ${avg_time}s"
}

# Main loop
while true; do
    show_menu
    case $choice in
        1) test_collections ;;
        2) test_vectors ;;
        3) test_search ;;
        4) test_filters ;;
        5) test_errors ;;
        6) run_all_tests ;;
        7) benchmark ;;
        0) echo "Exiting..."; exit 0 ;;
        *) echo "Invalid option" ;;
    esac
done

