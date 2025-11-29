/**
 * Test Master/Replica routing functionality
 */
import { VectorizerClient } from './src/client.js';

const MASTER_URL = 'http://localhost:15002';
const REPLICA_URL = 'http://localhost:17780';
const API_KEY = 'eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjoiYWRtaW4iLCJ1c2VybmFtZSI6ImFkbWluIiwicm9sZXMiOlsiQWRtaW4iXSwiaWF0IjoxNzY0Mzk0MzI3LCJleHAiOjE3NjQzOTc5Mjd9.AnLPTgdHRfCFMdMp6VFemhcIZPUfpzjwB5r6xOkCxNQ';

async function testMasterReplicaRouting() {
  console.log('=== JavaScript SDK Master/Replica Test ===\n');

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
  console.log('   Client created with master/replica topology');

  // 2. Test health check (read operation - should go to replica)
  console.log('2. Testing health check (read - should go to replica)...');
  try {
    const health = await client.healthCheck();
    console.log(`   Health status: ${health.status}`);
  } catch (e) {
    console.log(`   Health failed: ${e.message}`);
  }

  // 3. Test listing collections (read operation)
  console.log('3. Listing collections (read)...');
  try {
    const collections = await client.listCollections();
    console.log(`   Found ${collections?.length || 0} collections`);
  } catch (e) {
    console.log(`   List failed: ${e.message}`);
  }

  // 4. Test withMaster callback
  console.log('4. Testing withMaster() callback...');
  try {
    await client.withMaster(async (masterClient) => {
      console.log('   Inside withMaster callback');
      const health = await masterClient.healthCheck();
      console.log(`   Master health: ${health.status}`);
    });
  } catch (e) {
    console.log(`   withMaster failed: ${e.message}`);
  }

  // 5. Test backward compatibility with single baseURL
  console.log('\n5. Testing backward compatibility (single baseURL)...');
  const singleClient = new VectorizerClient({
    baseURL: MASTER_URL,
    apiKey: API_KEY
  });
  try {
    const health = await singleClient.healthCheck();
    console.log(`   Single URL mode works: ${health.status}`);
  } catch (e) {
    console.log(`   Single URL failed: ${e.message}`);
  }

  console.log('\n=== JavaScript SDK Test Complete ===');
}

testMasterReplicaRouting().catch(console.error);
