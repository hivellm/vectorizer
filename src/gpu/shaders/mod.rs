//! Shaders WGSL para operações GPU

/// Shader de similaridade coseno
pub const COSINE_SIMILARITY_SHADER: &str = include_str!("similarity.wgsl");

/// Shader de distância euclidiana
pub const EUCLIDEAN_DISTANCE_SHADER: &str = include_str!("distance.wgsl");

/// Shader de produto escalar
pub const DOT_PRODUCT_SHADER: &str = include_str!("dot_product.wgsl");

/// Tipo de shader
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderType {
    CosineSimilarity,
    CosineSimilarityVec4,
    EuclideanDistance,
    EuclideanDistanceVec4,
    ManhattanDistance,
    DotProduct,
    DotProductVec4,
}

impl ShaderType {
    /// Obter código fonte do shader
    pub fn source(&self) -> &'static str {
        match self {
            Self::CosineSimilarity | Self::CosineSimilarityVec4 => COSINE_SIMILARITY_SHADER,
            Self::EuclideanDistance | Self::EuclideanDistanceVec4 | Self::ManhattanDistance => {
                EUCLIDEAN_DISTANCE_SHADER
            }
            Self::DotProduct | Self::DotProductVec4 => DOT_PRODUCT_SHADER,
        }
    }

    /// Obter nome da função de entry point
    pub fn entry_point(&self) -> &'static str {
        match self {
            Self::CosineSimilarity => "cosine_similarity",
            Self::CosineSimilarityVec4 => "cosine_similarity_vec4",
            Self::EuclideanDistance => "euclidean_distance",
            Self::EuclideanDistanceVec4 => "euclidean_distance_vec4",
            Self::ManhattanDistance => "manhattan_distance",
            Self::DotProduct => "dot_product",
            Self::DotProductVec4 => "dot_product_vec4",
        }
    }

    /// Verificar se usa vetorização
    pub fn is_vectorized(&self) -> bool {
        matches!(
            self,
            Self::CosineSimilarityVec4 | Self::EuclideanDistanceVec4 | Self::DotProductVec4
        )
    }

    /// Selecionar shader apropriado baseado na dimensão
    pub fn select_for_dimension(base: ShaderType, dimension: usize) -> Self {
        // Usar vetorização se a dimensão for múltiplo de 4
        if dimension % 4 == 0 && dimension >= 128 {
            match base {
                Self::CosineSimilarity => Self::CosineSimilarityVec4,
                Self::EuclideanDistance => Self::EuclideanDistanceVec4,
                Self::DotProduct => Self::DotProductVec4,
                other => other,
            }
        } else {
            base
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_sources() {
        assert!(!COSINE_SIMILARITY_SHADER.is_empty());
        assert!(!EUCLIDEAN_DISTANCE_SHADER.is_empty());
        assert!(!DOT_PRODUCT_SHADER.is_empty());
    }

    #[test]
    fn test_entry_points() {
        assert_eq!(
            ShaderType::CosineSimilarity.entry_point(),
            "cosine_similarity"
        );
        assert_eq!(
            ShaderType::EuclideanDistance.entry_point(),
            "euclidean_distance"
        );
    }

    #[test]
    fn test_shader_selection() {
        // Dimensões pequenas ou não múltiplas de 4 devem usar shader não vetorizado
        let shader = ShaderType::select_for_dimension(ShaderType::CosineSimilarity, 100);
        assert_eq!(shader, ShaderType::CosineSimilarity);

        // Dimensões grandes e múltiplas de 4 devem usar shader vetorizado
        let shader = ShaderType::select_for_dimension(ShaderType::CosineSimilarity, 512);
        assert_eq!(shader, ShaderType::CosineSimilarityVec4);
    }
}
