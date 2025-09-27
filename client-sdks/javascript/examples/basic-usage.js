/**
 * Basic usage example for the Hive Vectorizer JavaScript SDK.
 */

import { VectorizerClient } from '../src/index.js';

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
if (import.meta.url === `file://${process.argv[1]}`) {
  main().catch(console.error);
}

export { main };
