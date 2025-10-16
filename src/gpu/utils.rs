//! Utilitários para operações GPU

use crate::error::Result;

/// Achatar vetores 2D em vetor 1D
pub fn flatten_vectors(vectors: &[Vec<f32>]) -> Vec<f32> {
    let dimension = if vectors.is_empty() {
        0
    } else {
        vectors[0].len()
    };

    let mut flat = Vec::with_capacity(vectors.len() * dimension);
    for v in vectors {
        flat.extend_from_slice(v);
    }
    flat
}

/// Converter vetor 1D achatado de volta para 2D
pub fn unflatten_vectors(flat: &[f32], count: usize, dimension: usize) -> Vec<Vec<f32>> {
    let mut vectors = Vec::with_capacity(count);
    for i in 0..count {
        let start = i * dimension;
        let end = start + dimension;
        vectors.push(flat[start..end].to_vec());
    }
    vectors
}

/// Normalizar vetor (para similaridade coseno)
pub fn normalize_vector(v: &[f32]) -> Vec<f32> {
    let magnitude: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude > 0.0 {
        v.iter().map(|x| x / magnitude).collect()
    } else {
        v.to_vec()
    }
}

/// Verificar se dimensões são compatíveis
pub fn check_dimensions(vectors: &[Vec<f32>]) -> Result<usize> {
    if vectors.is_empty() {
        return Ok(0);
    }

    let dimension = vectors[0].len();

    for (i, v) in vectors.iter().enumerate() {
        if v.len() != dimension {
            return Err(crate::error::VectorizerError::InvalidDimension {
                expected: dimension,
                got: v.len(),
            });
        }
    }

    Ok(dimension)
}

/// Calcular número ótimo de elementos por workgroup
pub fn optimal_workgroup_elements(total_elements: usize, max_workgroup_size: u32) -> u32 {
    let mut size = max_workgroup_size;

    // Se temos poucos elementos, reduzir workgroup size
    while size > 32 && total_elements < (size as usize / 2) {
        size /= 2;
    }

    size
}

/// Formatar tamanho de memória em string legível
pub fn format_memory_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Estimar uso de memória para operação
pub fn estimate_memory_usage(query_count: usize, vector_count: usize, dimension: usize) -> u64 {
    // Queries
    let query_mem = (query_count * dimension * std::mem::size_of::<f32>()) as u64;

    // Vectors
    let vector_mem = (vector_count * dimension * std::mem::size_of::<f32>()) as u64;

    // Resultados
    let result_mem = (query_count * vector_count * std::mem::size_of::<f32>()) as u64;

    // Params + overhead (~5% extra)
    let overhead = ((query_mem + vector_mem + result_mem) as f64 * 0.05) as u64;

    query_mem + vector_mem + result_mem + overhead + 1024 // +1KB para params
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatten_unflatten() {
        let vectors = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
            vec![7.0, 8.0, 9.0],
        ];

        let flat = flatten_vectors(&vectors);
        assert_eq!(flat.len(), 9);
        assert_eq!(flat, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]);

        let unflat = unflatten_vectors(&flat, 3, 3);
        assert_eq!(unflat, vectors);
    }

    #[test]
    fn test_normalize_vector() {
        let v = vec![3.0, 4.0]; // magnitude = 5.0
        let normalized = normalize_vector(&v);

        // Verificar magnitude = 1.0
        let magnitude: f32 = normalized.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_check_dimensions() {
        let vectors = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];

        assert_eq!(check_dimensions(&vectors).unwrap(), 3);

        let invalid = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0], // dimensão diferente
        ];

        assert!(check_dimensions(&invalid).is_err());
    }

    #[test]
    fn test_format_memory_size() {
        assert_eq!(format_memory_size(512), "512 bytes");
        assert_eq!(format_memory_size(2048), "2.00 KB");
        assert_eq!(format_memory_size(5 * 1024 * 1024), "5.00 MB");
        assert_eq!(format_memory_size(3 * 1024 * 1024 * 1024), "3.00 GB");
    }

    #[test]
    fn test_memory_estimation() {
        let mem = estimate_memory_usage(10, 100, 512);

        // Queries: 10 * 512 * 4 = 20,480 bytes
        // Vectors: 100 * 512 * 4 = 204,800 bytes
        // Results: 10 * 100 * 4 = 4,000 bytes
        // Total com overhead deve ser > 229,280 bytes

        assert!(mem > 229_280);
        assert!(mem < 300_000); // com overhead não deve passar muito
    }
}
