/**
 * Test Master/Replica routing functionality
 */
import { VectorizerClient, SearchRequest, CreateVectorRequest, ReadOptions } from './src/index';

const MASTER_URL = 'http://localhost:15002';
const REPLICA_URL = 'http://localhost:17780';
const API_KEY = 'eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjoiYWRtaW4iLCJ1c2VybmFtZSI6ImFkbWluIiwicm9sZXMiOlsiQWRtaW4iXSwiaWF0IjoxNzY0Mzk0MzI3LCJleHAiOjE3NjQzOTc5Mjd9.AnLPTgdHRfCFMdMp6VFemhcIZPUfpzjwB5r6xOkCxNQ';

async function testMasterReplicaRouting() {
  console.log('=== TypeScript SDK Master/Replica Test ===\n');

  // 1. Test with hosts configuration
  console.log('1. Creating client with hosts configuration...');
  const client = new VectorizerClient({
    hosts: {
      master: MASTER_URL,
      replicas: [REPLICA_URL]
    },
    readPreference: 'replica',
    apiKey: API_KEY
  });

  // 2. Test health check (read operation - should go to replica)
  console.log('2. Testing health check (read)...');
  try {
    const health = await client.healthCheck();
    console.log(`   Health status: ${health.status}`);
  } catch (e: any) {
    console.log(`   Health failed: ${e.message}`);
  }

  // 3. Create a test collection (write operation - should go to master)
  const testCollection = `test_ts_routing_${Date.now()}`;
  console.log(`3. Creating collection "${testCollection}" (write)...`);
  try {
    await client.createCollection({
      name: testCollection,
      dimension: 128,
      similarity_metric: 'cosine'
    });
    console.log('   Collection created on master');
  } catch (e: any) {
    console.log(`   Create failed: ${e.message}`);
  }

  // 4. List collections (read operation)
  console.log('4. Listing collections (read)...');
  try {
    const collections = await client.listCollections();
    console.log(`   Found ${collections.length} collections`);
  } catch (e: any) {
    console.log(`   List failed: ${e.message}`);
  }

  // 5. Insert vectors (write operation)
  console.log('5. Inserting vectors (write)...');
  const queryVector: number[] = Array(128).fill(0).map(() => Math.random());
  const vectors: CreateVectorRequest[] = [
    { data: queryVector },
    { data: Array(128).fill(0).map(() => Math.random()) },
    { data: Array(128).fill(0).map(() => Math.random()) }
  ];
  try {
    await client.insertVectors(testCollection, vectors);
    console.log('   Vectors inserted on master');
  } catch (e: any) {
    console.log(`   Insert failed: ${e.message}`);
  }

  // 6. Search vectors (read operation)
  console.log('6. Searching vectors (read - should go to replica)...');
  try {
    const searchRequest: SearchRequest = {
      query_vector: queryVector,
      limit: 2
    };
    const results = await client.searchVectors(testCollection, searchRequest);
    console.log(`   Found ${results.results?.length || 0} results`);
  } catch (e: any) {
    console.log(`   Search failed: ${e.message}`);
  }

  // 7. Test withMaster for read-your-writes
  console.log('7. Testing withMaster() for read-your-writes pattern...');
  try {
    await client.withMaster(async (masterClient) => {
      // Insert new vector
      await masterClient.insertVectors(testCollection, [
        { data: Array(128).fill(0).map(() => Math.random()) }
      ]);
      // Immediately read from master
      const searchRequest: SearchRequest = {
        query_vector: queryVector,
        limit: 5
      };
      const results = await masterClient.searchVectors(testCollection, searchRequest);
      console.log(`   withMaster: Inserted and found ${results.results?.length || 0} results on master`);
    });
  } catch (e: any) {
    console.log(`   withMaster failed: ${e.message}`);
  }

  // 8. Test read preference override
  console.log('8. Testing read preference override to master...');
  try {
    const searchRequest: SearchRequest = {
      query_vector: queryVector,
      limit: 2
    };
    const options: ReadOptions = { readPreference: 'master' };
    const results = await client.searchVectors(testCollection, searchRequest, options);
    console.log(`   Override to master: Found ${results.results?.length || 0} results`);
  } catch (e: any) {
    console.log(`   Override failed: ${e.message}`);
  }

  // 9. Cleanup
  console.log('9. Cleaning up...');
  try {
    await client.deleteCollection(testCollection);
    console.log('   Collection deleted');
  } catch (e: any) {
    console.log(`   Delete failed: ${e.message}`);
  }

  // 10. Test backward compatibility with single baseURL
  console.log('\n10. Testing backward compatibility (single baseURL)...');
  const singleClient = new VectorizerClient({
    baseURL: MASTER_URL,
    apiKey: API_KEY
  });
  try {
    const health = await singleClient.healthCheck();
    console.log(`    Single URL mode works: ${health.status}`);
  } catch (e: any) {
    console.log(`    Single URL failed: ${e.message}`);
  }

  console.log('\n=== TypeScript SDK Test Complete ===');
}

testMasterReplicaRouting().catch(console.error);
