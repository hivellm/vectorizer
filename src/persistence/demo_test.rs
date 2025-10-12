use crate::persistence::{
    types::{EnhancedCollectionMetadata, CollectionType, CollectionSource, WALEntry, Operation, Transaction, TransactionStatus},
    wal::{WriteAheadLog, WALConfig},
};
use crate::models::{CollectionConfig, DistanceMetric, QuantizationConfig, HnswConfig, CompressionConfig};
use std::collections::HashMap;
use tempfile::tempdir;
use tracing::info;

/// Demonstração completa do sistema de persistência
#[tokio::test]
async fn test_persistence_demo() {
    // Configurar logging para ver o que está acontecendo
    let _ = tracing_subscriber::fmt::try_init();
    
    info!("🚀 Iniciando demonstração do sistema de persistência");
    
    // 1. Criar configuração de coleção
    let config = CollectionConfig {
        dimension: 384,
        metric: DistanceMetric::Cosine,
        quantization: QuantizationConfig::default(),
        hnsw_config: HnswConfig::default(),
        compression: CompressionConfig::default(),
            normalization: None,
    };
    
    info!("✅ Configuração de coleção criada: dimensão={}, métrica={:?}", 
          config.dimension, config.metric);
    
    // 2. Criar metadados de coleção workspace
    let workspace_metadata = EnhancedCollectionMetadata::new_workspace(
        "demo-workspace-collection".to_string(),
        "demo-project".to_string(),
        "/workspace/config.yml".to_string(),
        config.clone(),
    );
    
    info!("✅ Metadados de coleção workspace criados:");
    info!("   - Nome: {}", workspace_metadata.name);
    info!("   - Tipo: {:?}", workspace_metadata.collection_type);
    info!("   - Read-only: {}", workspace_metadata.is_read_only);
    info!("   - Dimensão: {}", workspace_metadata.dimension);
    
    // 3. Criar metadados de coleção dinâmica
    let dynamic_metadata = EnhancedCollectionMetadata::new_dynamic(
        "demo-dynamic-collection".to_string(),
        Some("user123".to_string()),
        "/collections".to_string(),
        config,
    );
    
    info!("✅ Metadados de coleção dinâmica criados:");
    info!("   - Nome: {}", dynamic_metadata.name);
    info!("   - Tipo: {:?}", dynamic_metadata.collection_type);
    info!("   - Read-only: {}", dynamic_metadata.is_read_only);
    info!("   - Criado por: {:?}", dynamic_metadata.source);
    
    // 4. Testar Write-Ahead Log (WAL)
    let temp_dir = tempdir().unwrap();
    let wal_path = temp_dir.path().join("demo.wal");
    
    let wal_config = WALConfig::default();
    let wal = WriteAheadLog::new(&wal_path, wal_config).await.unwrap();
    
    info!("✅ Write-Ahead Log inicializado em: {}", wal_path.display());
    
    // 5. Criar e executar transação
    let mut transaction = Transaction::new(1, "demo-collection".to_string());
    
    // Adicionar operações à transação
    let operation1 = Operation::InsertVector {
        vector_id: "vec1".to_string(),
        data: vec![0.1, 0.2, 0.3, 0.4],
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("source".to_string(), "demo".to_string());
            meta.insert("type".to_string(), "test".to_string());
            meta
        },
    };
    
    let operation2 = Operation::InsertVector {
        vector_id: "vec2".to_string(),
        data: vec![0.5, 0.6, 0.7, 0.8],
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("source".to_string(), "demo".to_string());
            meta.insert("type".to_string(), "test".to_string());
            meta
        },
    };
    
    transaction.add_operation(operation1);
    transaction.add_operation(operation2);
    
    info!("✅ Transação criada com {} operações", transaction.operations.len());
    
    // 6. Aplicar transação ao WAL
    let sequence = wal.append_transaction(&transaction).await.unwrap();
    
    info!("✅ Transação aplicada ao WAL com sequência: {}", sequence);
    
    // 7. Verificar integridade do WAL
    wal.validate_integrity().await.unwrap();
    
    info!("✅ Integridade do WAL validada com sucesso");
    
    // 8. Ler entradas do WAL
    let entries = wal.read_from(0).await.unwrap();
    
    info!("✅ Lidas {} entradas do WAL:", entries.len());
    for (i, entry) in entries.iter().enumerate() {
        info!("   {}. Sequência: {}, Operação: {}, Coleção: {}", 
              i + 1, entry.sequence, entry.operation.operation_type(), entry.collection_id);
    }
    
    // 9. Testar checkpoint
    let checkpoint_sequence = wal.checkpoint().await.unwrap();
    
    info!("✅ Checkpoint criado com sequência: {}", checkpoint_sequence);
    
    // 10. Verificar estatísticas do WAL
    let stats = wal.get_stats().await.unwrap();
    
    info!("✅ Estatísticas do WAL:");
    info!("   - Tamanho do arquivo: {} bytes", stats.file_size_bytes);
    info!("   - Número de entradas: {}", stats.entry_count);
    info!("   - Sequência atual: {}", stats.current_sequence);
    
    // 11. Testar checksums de integridade
    let data_checksum = workspace_metadata.calculate_data_checksum();
    let index_checksum = workspace_metadata.calculate_index_checksum();
    
    info!("✅ Checksums calculados:");
    info!("   - Data checksum: {}", data_checksum);
    info!("   - Index checksum: {}", index_checksum);
    
    // 12. Testar atualização de metadados
    let mut updated_metadata = workspace_metadata.clone();
    updated_metadata.update_after_operation(100, 50);
    updated_metadata.update_checksums();
    
    info!("✅ Metadados atualizados:");
    info!("   - Vectors: {}", updated_metadata.vector_count);
    info!("   - Documents: {}", updated_metadata.document_count);
    info!("   - Updated at: {}", updated_metadata.updated_at);
    
    // 13. Verificar tipos de coleção
    assert!(workspace_metadata.is_workspace());
    assert!(!workspace_metadata.is_dynamic());
    assert!(!dynamic_metadata.is_workspace());
    assert!(dynamic_metadata.is_dynamic());
    
    info!("✅ Verificação de tipos de coleção passou");
    
    // 14. Testar status de transação
    assert_eq!(transaction.status, TransactionStatus::InProgress);
    assert!(!transaction.is_completed());
    
    transaction.commit();
    assert_eq!(transaction.status, TransactionStatus::Committed);
    assert!(transaction.is_completed());
    
    info!("✅ Teste de status de transação passou");
    
    info!("🎉 Demonstração do sistema de persistência concluída com sucesso!");
    info!("📊 Resumo:");
    info!("   - ✅ Metadados de coleção workspace e dinâmica");
    info!("   - ✅ Write-Ahead Log com transações atômicas");
    info!("   - ✅ Validação de integridade");
    info!("   - ✅ Checkpoint e recuperação");
    info!("   - ✅ Checksums para verificação de dados");
    info!("   - ✅ Sistema de tipos de coleção");
    info!("   - ✅ Gerenciamento de transações");
}

/// Teste de performance básico do WAL
#[tokio::test]
async fn test_wal_performance_demo() {
    let temp_dir = tempdir().unwrap();
    let wal_path = temp_dir.path().join("performance.wal");
    
    let wal_config = WALConfig::default();
    let wal = WriteAheadLog::new(&wal_path, wal_config).await.unwrap();
    
    let start_time = std::time::Instant::now();
    
    // Inserir 1000 operações
    for i in 0..1000 {
        let operation = Operation::InsertVector {
            vector_id: format!("vec_{}", i),
            data: vec![i as f32; 384],
            metadata: HashMap::new(),
        };
        
        wal.append("test-collection", operation).await.unwrap();
    }
    
    let duration = start_time.elapsed();
    
    println!("✅ Performance test: {} operações em {:?} ({:.2} ops/sec)", 
             1000, duration, 1000.0 / duration.as_secs_f64());
    
    // Verificar que todas as operações foram registradas
    let entries = wal.read_from(0).await.unwrap();
    assert_eq!(entries.len(), 1000);
    
    println!("✅ Todas as 1000 operações foram registradas corretamente");
}
