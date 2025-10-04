// Compute Shader para Distância Euclidiana
// Otimizado para GPUs modernas (Metal, Vulkan, DirectX)

struct Params {
    query_count: u32,
    vector_count: u32,
    dimension: u32,
    _padding: u32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> queries: array<f32>;
@group(0) @binding(2) var<storage, read> vectors: array<f32>;
@group(0) @binding(3) var<storage, read_write> distances: array<f32>;

@compute @workgroup_size(256, 1, 1)
fn euclidean_distance(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    let total_comparisons = params.query_count * params.vector_count;
    
    if (idx >= total_comparisons) {
        return;
    }
    
    let query_idx = idx / params.vector_count;
    let vector_idx = idx % params.vector_count;
    
    let query_start = query_idx * params.dimension;
    let vector_start = vector_idx * params.dimension;
    
    var distance_squared: f32 = 0.0;
    
    // Calcular distância euclidiana ao quadrado
    for (var i: u32 = 0u; i < params.dimension; i = i + 1u) {
        let diff = queries[query_start + i] - vectors[vector_start + i];
        distance_squared = distance_squared + (diff * diff);
    }
    
    // Armazenar raiz quadrada da distância
    distances[idx] = sqrt(distance_squared);
}

// Versão otimizada com vetorização
@compute @workgroup_size(256, 1, 1)
fn euclidean_distance_vec4(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    let total_comparisons = params.query_count * params.vector_count;
    
    if (idx >= total_comparisons) {
        return;
    }
    
    let query_idx = idx / params.vector_count;
    let vector_idx = idx % params.vector_count;
    
    let query_start = query_idx * params.dimension;
    let vector_start = vector_idx * params.dimension;
    
    var distance_squared: f32 = 0.0;
    
    // Processar 4 elementos por vez
    let vec4_count = params.dimension / 4u;
    let remainder = params.dimension % 4u;
    
    for (var i: u32 = 0u; i < vec4_count; i = i + 1u) {
        let offset = i * 4u;
        let q = vec4<f32>(
            queries[query_start + offset],
            queries[query_start + offset + 1u],
            queries[query_start + offset + 2u],
            queries[query_start + offset + 3u]
        );
        let v = vec4<f32>(
            vectors[vector_start + offset],
            vectors[vector_start + offset + 1u],
            vectors[vector_start + offset + 2u],
            vectors[vector_start + offset + 3u]
        );
        
        let diff = q - v;
        distance_squared = distance_squared + dot(diff, diff);
    }
    
    // Processar elementos restantes
    for (var i: u32 = vec4_count * 4u; i < params.dimension; i = i + 1u) {
        let diff = queries[query_start + i] - vectors[vector_start + i];
        distance_squared = distance_squared + (diff * diff);
    }
    
    distances[idx] = sqrt(distance_squared);
}

// Distância de Manhattan (L1)
@compute @workgroup_size(256, 1, 1)
fn manhattan_distance(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    let total_comparisons = params.query_count * params.vector_count;
    
    if (idx >= total_comparisons) {
        return;
    }
    
    let query_idx = idx / params.vector_count;
    let vector_idx = idx % params.vector_count;
    
    let query_start = query_idx * params.dimension;
    let vector_start = vector_idx * params.dimension;
    
    var distance: f32 = 0.0;
    
    for (var i: u32 = 0u; i < params.dimension; i = i + 1u) {
        let diff = queries[query_start + i] - vectors[vector_start + i];
        distance = distance + abs(diff);
    }
    
    distances[idx] = distance;
}
