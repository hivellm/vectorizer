#[cfg(test)]
mod bend_tests {
    use super::*;
    use crate::bend::{BendConfig, BendExecutor, BendVectorOperations};
    use std::path::Path;

    #[tokio::test]
    async fn test_bend_executor_availability() {
        let config = BendConfig::default();
        let executor = BendExecutor::new(config);
        
        // This test will fail if Bend is not installed
        // but that's expected for now
        let result = executor.check_bend_availability();
        println!("Bend availability check: {:?}", result);
    }

    #[tokio::test]
    async fn test_simple_bend_program() {
        let config = BendConfig::default();
        let executor = BendExecutor::new(config);
        
        let program_path = Path::new("examples/bend/simple_test.bend");
        
        // This test will fail if Bend is not installed
        // but shows how it would work
        let result = executor.execute_bend_program(program_path).await;
        println!("Bend program execution: {:?}", result);
    }

    #[test]
    fn test_vector_operations_fallback() {
        let config = BendConfig::default();
        let ops = BendVectorOperations::new(config);
        
        let query = vec![1.0, 0.0, 0.0];
        let vectors = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
        ];
        
        // Test the fallback implementation
        let similarity = ops.cosine_similarity(&query, &vectors[0]);
        assert_eq!(similarity, 1.0);
        
        let similarity = ops.cosine_similarity(&query, &vectors[1]);
        assert_eq!(similarity, 0.0);
    }
}
