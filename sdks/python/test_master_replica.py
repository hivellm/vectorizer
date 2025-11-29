"""
Test Master/Replica routing functionality
"""
import asyncio
import sys
sys.path.insert(0, '.')
from client import VectorizerClient
from models import HostConfig, ReadPreference

MASTER_URL = 'http://localhost:15002'
REPLICA_URL = 'http://localhost:17780'
API_KEY = 'eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjoiYWRtaW4iLCJ1c2VybmFtZSI6ImFkbWluIiwicm9sZXMiOlsiQWRtaW4iXSwiaWF0IjoxNzY0Mzk0MzI3LCJleHAiOjE3NjQzOTc5Mjd9.AnLPTgdHRfCFMdMp6VFemhcIZPUfpzjwB5r6xOkCxNQ'


async def test_master_replica_routing():
    print('=== Python SDK Master/Replica Test ===\n')

    # 1. Test with hosts configuration
    print('1. Creating client with hosts configuration...')
    client = VectorizerClient(
        hosts=HostConfig(master=MASTER_URL, replicas=[REPLICA_URL]),
        read_preference=ReadPreference.REPLICA,
        api_key=API_KEY
    )
    print('   Client created with master/replica topology')

    # 2. Test health check (read operation - should go to replica)
    print('2. Testing health check (read - should go to replica)...')
    try:
        health = await client.health_check()
        print(f'   Health status: {health.get("status", "unknown")}')
    except Exception as e:
        print(f'   Health failed: {e}')

    # 3. Test listing collections (read operation)
    print('3. Listing collections (read)...')
    try:
        collections = await client.list_collections()
        print(f'   Found {len(collections) if collections else 0} collections')
    except Exception as e:
        print(f'   List failed: {e}')

    # 4. Test with_master callback
    print('4. Testing with_master() callback...')
    try:
        async def master_callback(master_client):
            print('   Inside with_master callback')
            health = await master_client.health_check()
            return health.get("status", "unknown")

        result = await client.with_master(master_callback)
        print(f'   Master health: {result}')
    except Exception as e:
        print(f'   with_master failed: {e}')

    # 5. Test backward compatibility with single base_url
    print('\n5. Testing backward compatibility (single base_url)...')
    single_client = VectorizerClient(
        base_url=MASTER_URL,
        api_key=API_KEY
    )
    try:
        health = await single_client.health_check()
        print(f'   Single URL mode works: {health.get("status", "unknown")}')
    except Exception as e:
        print(f'   Single URL failed: {e}')

    # Close clients properly
    await client.close()
    await single_client.close()

    print('\n=== Python SDK Test Complete ===')


if __name__ == '__main__':
    asyncio.run(test_master_replica_routing())
