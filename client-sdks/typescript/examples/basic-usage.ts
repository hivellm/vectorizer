/**
 * Basic usage example for the Hive Vectorizer TypeScript SDK.
 */

import { VectorizerClient } from '../src/index';

async function main() {
  // Create client
  const client = new VectorizerClient({
    baseURL: 'http://localhost:15001',
    apiKey: 'your-api-key-here',
    logger: { level: 'info', enabled: true }
  });

  try {
    // Health check
    console.log('🔍 Checking server health...');
    const health = await client.healthCheck();
    console.log('✅ Server status:', health.status);

    // Get database stats
    console.log('\n📊 Getting database statistics...');
    const stats = await client.getDatabaseStats();
    console.log('📈 Database stats:', {
      collections: stats.total_collections,
      vectors: stats.total_vectors,
      size: `${(stats.total_size_bytes / 1024 / 1024).toFixed(2)} MB`
    });

    // List existing collections
    console.log('\n📋 Listing collections...');
    const collections = await client.listCollections();
    console.log('📁 Collections:', collections.map(c => c.name));

    // Create a new collection
    console.log('\n🆕 Creating collection...');
    const collection = await client.createCollection({
      name: 'example-documents',
      dimension: 384,
      similarity_metric: 'cosine',
      description: 'Example collection for testing'
    });
    console.log('✅ Collection created:', collection.name);

    // Insert some example vectors
    console.log('\n📥 Inserting vectors...');
    const vectors = [
      {
        data: Array.from({ length: 384 }, () => Math.random()),
        metadata: { 
          source: 'document1.pdf',
          title: 'Introduction to Machine Learning',
          category: 'AI'
        }
      },
      {
        data: Array.from({ length: 384 }, () => Math.random()),
        metadata: { 
          source: 'document2.pdf',
          title: 'Deep Learning Fundamentals',
          category: 'AI'
        }
      },
      {
        data: Array.from({ length: 384 }, () => Math.random()),
        metadata: { 
          source: 'document3.pdf',
          title: 'Data Science Best Practices',
          category: 'Data'
        }
      }
    ];

    const insertResult = await client.insertVectors('example-documents', vectors);
    console.log('✅ Vectors inserted:', insertResult.inserted);

    // Search for similar vectors
    console.log('\n🔍 Searching for similar vectors...');
    const queryVector = Array.from({ length: 384 }, () => Math.random());
    const searchResults = await client.searchVectors('example-documents', {
      query_vector: queryVector,
      limit: 3,
      include_metadata: true
    });

    console.log('🎯 Search results:');
    searchResults.results.forEach((result, index) => {
      console.log(`  ${index + 1}. Score: ${result.score.toFixed(4)}`);
      console.log(`     Title: ${result.metadata?.title}`);
      console.log(`     Category: ${result.metadata?.category}`);
    });

    // Text search
    console.log('\n📝 Performing text search...');
    const textResults = await client.searchText('example-documents', {
      query: 'machine learning algorithms',
      limit: 2,
      include_metadata: true
    });

    console.log('🔤 Text search results:');
    textResults.results.forEach((result, index) => {
      console.log(`  ${index + 1}. Score: ${result.score.toFixed(4)}`);
      console.log(`     Title: ${result.metadata?.title}`);
    });

    // Generate embeddings
    console.log('\n🧠 Generating embeddings...');
    const embedding = await client.embedText({
      text: 'artificial intelligence and machine learning',
      model: 'bert-base'
    });
    console.log('✅ Embedding generated:', {
      dimension: embedding.embedding.length,
      model: embedding.model
    });

    // Get collection info
    console.log('\n📊 Getting collection information...');
    const collectionInfo = await client.getCollection('example-documents');
    console.log('📈 Collection info:', {
      name: collectionInfo.name,
      dimension: collectionInfo.dimension,
      vectorCount: collectionInfo.vector_count,
      size: `${(collectionInfo.size_bytes || 0) / 1024} KB`
    });

    // WebSocket example (if configured)
    if (client.getConfig().wsURL) {
      console.log('\n🔌 Testing WebSocket connection...');
      try {
        await client.connectWebSocket();
        console.log('✅ WebSocket connected');

        // Listen for messages
        client.onWebSocketEvent('message', (data) => {
          console.log('📨 WebSocket message received:', data);
        });

        // Send a test message
        client.sendWebSocketMessage({
          type: 'ping',
          timestamp: Date.now()
        });

        // Wait a bit then disconnect
        await new Promise(resolve => setTimeout(resolve, 1000));
        client.disconnectWebSocket();
        console.log('🔌 WebSocket disconnected');
      } catch (error) {
        console.log('⚠️ WebSocket not available:', error.message);
      }
    }

    // Batch operations example
    console.log('\n🔄 Batch operations example...');
    
    // Batch insert texts
    console.log('📥 Batch inserting texts...');
    const batchInsertResult = await client.batchInsertTexts('example-documents', {
      texts: [
        {
          id: 'batch-text-1',
          text: 'This is the first batch text for testing',
          metadata: { source: 'batch_test', type: 'example' }
        },
        {
          id: 'batch-text-2', 
          text: 'This is the second batch text for testing',
          metadata: { source: 'batch_test', type: 'example' }
        },
        {
          id: 'batch-text-3',
          text: 'This is the third batch text for testing',
          metadata: { source: 'batch_test', type: 'example' }
        }
      ],
      config: {
        max_batch_size: 100,
        parallel_workers: 4,
        atomic: true
      }
    });
    console.log('✅ Batch insert result:', {
      successful: batchInsertResult.successful_operations,
      failed: batchInsertResult.failed_operations,
      duration: `${batchInsertResult.duration_ms}ms`
    });

    // Batch search
    console.log('🔍 Batch searching...');
    const batchSearchResult = await client.batchSearchVectors('example-documents', {
      queries: [
        { query: 'batch text', limit: 5 },
        { query: 'testing', limit: 3 },
        { query: 'example', limit: 2 }
      ],
      config: {
        parallel_workers: 2
      }
    });
    console.log('✅ Batch search result:', {
      successful: batchSearchResult.successful_queries,
      failed: batchSearchResult.failed_queries,
      duration: `${batchSearchResult.duration_ms}ms`,
      totalResults: batchSearchResult.results.reduce((sum, r) => sum + r.length, 0)
    });

    // Batch delete
    console.log('🗑️ Batch deleting...');
    const batchDeleteResult = await client.batchDeleteVectors('example-documents', {
      vector_ids: ['batch-text-1', 'batch-text-2', 'batch-text-3'],
      config: {
        atomic: true
      }
    });
    console.log('✅ Batch delete result:', {
      successful: batchDeleteResult.successful_operations,
      failed: batchDeleteResult.failed_operations,
      duration: `${batchDeleteResult.duration_ms}ms`
    });

    // Summarization example
    console.log('\n📝 Testing summarization...');
    const longText = `
    Artificial Intelligence (AI) has revolutionized numerous industries and continues to shape the future of technology. 
    From healthcare to finance, AI applications are transforming how we work, live, and interact with the world around us.
    
    In healthcare, AI is being used for medical diagnosis, drug discovery, and personalized treatment plans. 
    Machine learning algorithms can analyze vast amounts of medical data to identify patterns and predict outcomes.
    
    In finance, AI powers algorithmic trading, fraud detection, and risk assessment. 
    These systems can process millions of transactions in real-time to identify suspicious activities.
    
    The automotive industry is leveraging AI for autonomous vehicles, traffic optimization, and predictive maintenance.
    Self-driving cars use computer vision and machine learning to navigate roads safely.
    
    As AI technology continues to advance, we can expect even more innovative applications across various sectors.
    However, it's important to consider the ethical implications and ensure responsible AI development.
    `;

    // Summarize text using extractive method
    console.log('📄 Summarizing text using extractive method...');
    const textSummary = await client.summarizeText({
      text: longText,
      method: 'extractive',
      compression_ratio: 0.3,
      language: 'en'
    });
    console.log('✅ Text summary:', {
      originalLength: textSummary.original_length,
      summaryLength: textSummary.summary_length,
      compressionRatio: textSummary.compression_ratio,
      summary: textSummary.summary.substring(0, 100) + '...'
    });

    // Summarize context using keyword method
    console.log('🔑 Summarizing context using keyword method...');
    const contextSummary = await client.summarizeContext({
      context: longText,
      method: 'keyword',
      max_length: 100,
      language: 'en'
    });
    console.log('✅ Context summary:', {
      originalLength: contextSummary.original_length,
      summaryLength: contextSummary.summary_length,
      summary: contextSummary.summary
    });

    // Get summary by ID
    console.log('🔍 Retrieving summary by ID...');
    const retrievedSummary = await client.getSummary(textSummary.summary_id);
    console.log('✅ Retrieved summary:', {
      method: retrievedSummary.method,
      summaryLength: retrievedSummary.summary_length,
      createdAt: retrievedSummary.created_at
    });

    // List summaries
    console.log('📋 Listing summaries...');
    const summaries = await client.listSummaries({ limit: 5 });
    console.log('✅ Found summaries:', {
      count: summaries.summaries.length,
      totalCount: summaries.total_count,
      methods: summaries.summaries.map(s => s.method)
    });

    // Clean up
    console.log('\n🧹 Cleaning up...');
    await client.deleteCollection('example-documents');
    console.log('✅ Collection deleted');

  } catch (error) {
    console.error('❌ Error:', error.message);
    if (error.details) {
      console.error('📋 Details:', error.details);
    }
  } finally {
    // Close client
    await client.close();
    console.log('\n👋 Client closed');
  }
}

// Run the example
if (require.main === module) {
  main().catch(console.error);
}

export { main };
