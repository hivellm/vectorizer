// Compute Shader para Similaridade Coseno
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
@group(0) @binding(3) var<storage, read_write> similarities: array<f32>;

// Workgroup size otimizado para a maioria das GPUs
@compute @workgroup_size(256, 1, 1)
fn cosine_similarity(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    let total_comparisons = params.query_count * params.vector_count;
    
    if (idx >= total_comparisons) {
        return;
    }
    
    let query_idx = idx / params.vector_count;
    let vector_idx = idx % params.vector_count;
    
    let query_start = query_idx * params.dimension;
    let vector_start = vector_idx * params.dimension;
    
    var dot_product: f32 = 0.0;
    var query_magnitude: f32 = 0.0;
    var vector_magnitude: f32 = 0.0;
    
    // Calcular produto escalar e magnitudes em um único loop
    for (var i: u32 = 0u; i < params.dimension; i = i + 1u) {
        let q = queries[query_start + i];
        let v = vectors[vector_start + i];
        
        dot_product = dot_product + (q * v);
        query_magnitude = query_magnitude + (q * q);
        vector_magnitude = vector_magnitude + (v * v);
    }
    
    // Calcular similaridade coseno
    query_magnitude = sqrt(query_magnitude);
    vector_magnitude = sqrt(vector_magnitude);
    
    if (query_magnitude > 0.0 && vector_magnitude > 0.0) {
        similarities[idx] = dot_product / (query_magnitude * vector_magnitude);
    } else {
        similarities[idx] = 0.0;
    }
}

// Versão otimizada com vetorização para dimensões múltiplas de 4
@compute @workgroup_size(256, 1, 1)
fn cosine_similarity_vec4(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    let total_comparisons = params.query_count * params.vector_count;
    
    if (idx >= total_comparisons) {
        return;
    }
    
    let query_idx = idx / params.vector_count;
    let vector_idx = idx % params.vector_count;
    
    let query_start = query_idx * params.dimension;
    let vector_start = vector_idx * params.dimension;
    
    var dot_product: f32 = 0.0;
    var query_magnitude: f32 = 0.0;
    var vector_magnitude: f32 = 0.0;
    
    // Processar 4 elementos por vez (vetorização)
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
        
        dot_product = dot_product + dot(q, v);
        query_magnitude = query_magnitude + dot(q, q);
        vector_magnitude = vector_magnitude + dot(v, v);
    }
    
    // Processar elementos restantes
    for (var i: u32 = vec4_count * 4u; i < params.dimension; i = i + 1u) {
        let q = queries[query_start + i];
        let v = vectors[vector_start + i];
        
        dot_product = dot_product + (q * v);
        query_magnitude = query_magnitude + (q * q);
        vector_magnitude = vector_magnitude + (v * v);
    }
    
    // Calcular similaridade coseno
    query_magnitude = sqrt(query_magnitude);
    vector_magnitude = sqrt(vector_magnitude);
    
    if (query_magnitude > 0.0 && vector_magnitude > 0.0) {
        similarities[idx] = dot_product / (query_magnitude * vector_magnitude);
    } else {
        similarities[idx] = 0.0;
    }
}
