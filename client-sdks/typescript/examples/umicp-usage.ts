/**
 * Example: Using the Vectorizer client with UMICP protocol
 * 
 * UMICP (Universal Messaging and Inter-process Communication Protocol) provides:
 * - Automatic compression (GZIP, DEFLATE, LZ4)
 * - Built-in encryption
 * - Request/response validation with checksums
 * - Lower latency for large payloads
 */

import { VectorizerClient } from '../src';

async function main() {
  console.log('=== Vectorizer Client with UMICP ===\n');

  // Option 1: Using connection string
  console.log('Option 1: Connection string');
  const client1 = new VectorizerClient({
    connectionString: 'umicp://localhost:15003',
    apiKey: 'your-api-key-here',
  });

  // Option 2: Using explicit configuration
  console.log('Option 2: Explicit configuration');
  const client2 = new VectorizerClient({
    protocol: 'umicp',
    apiKey: 'your-api-key-here',
    umicp: {
      host: 'localhost',
      port: 15003,
      compression: 'gzip', // gzip, deflate, or lz4
      encryption: true,
    },
  });

  // Check which protocol is being used
  console.log(`\nUsing protocol: ${client2.getProtocol()}`);

  try {
    // Health check
    console.log('\n1. Health Check');
    const health = await client2.healthCheck();
    console.log('Server status:', health);

    // List collections
    console.log('\n2. List Collections');
    const collections = await client2.listCollections();
    console.log(`Found ${collections.length} collection(s)`);

    if (collections.length > 0) {
      // Search in first collection
      const collectionName = collections[0].name;
      console.log(`\n3. Searching in collection: ${collectionName}`);
      
      const searchResults = await client2.searchVectors({
        collection: collectionName,
        query: 'example search query',
        limit: 5,
      });

      console.log(`Found ${searchResults.results.length} result(s)`);
      searchResults.results.forEach((result, i) => {
        console.log(`  ${i + 1}. Score: ${result.score.toFixed(4)}`);
      });
    }

    // Create a new collection
    console.log('\n4. Create Collection');
    await client2.createCollection({
      name: 'test-umicp-collection',
      dimension: 384,
      metric: 'cosine',
    });
    console.log('Collection created successfully');

    // Insert vectors
    console.log('\n5. Insert Vectors');
    await client2.insertText({
      collection: 'test-umicp-collection',
      texts: [
        { id: '1', text: 'Hello UMICP world' },
        { id: '2', text: 'UMICP provides efficient communication' },
        { id: '3', text: 'Vector search with compression' },
      ],
    });
    console.log('Vectors inserted successfully');

    // Search with UMICP
    console.log('\n6. Search with UMICP');
    const umicpSearchResults = await client2.searchVectors({
      collection: 'test-umicp-collection',
      query: 'efficient communication',
      limit: 3,
    });

    console.log(`Found ${umicpSearchResults.results.length} result(s):`);
    umicpSearchResults.results.forEach((result, i) => {
      console.log(`  ${i + 1}. ID: ${result.id}, Score: ${result.score.toFixed(4)}`);
      if (result.metadata?.text) {
        console.log(`     Text: ${result.metadata.text}`);
      }
    });

    // Cleanup
    console.log('\n7. Cleanup');
    await client2.deleteCollection('test-umicp-collection');
    console.log('Collection deleted');

  } catch (error) {
    console.error('Error:', error);
  }

  console.log('\n=== UMICP Demo Complete ===');
}

// Run if executed directly
if (require.main === module) {
  main().catch(console.error);
}

