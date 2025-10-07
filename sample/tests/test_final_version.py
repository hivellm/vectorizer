#!/usr/bin/env python3
"""
Teste Final da Versão Corrigida
"""

import asyncio
import httpx
import json

async def test_final_version():
    """Testa a versão final corrigida"""
    base_url = "http://localhost:15006"
    
    print("=== TESTE FINAL - VERSÃO CORRIGIDA ===")
    
    async with httpx.AsyncClient(timeout=30.0) as client:
        # Test 1: Health check
        print("\n1. Health Check...")
        try:
            response = await client.get(f"{base_url}/api/health")
            if response.status_code == 200:
                health_data = response.json()
                print(f"   Status: {health_data.get('status', 'unknown')}")
                print(f"   Vectorizer connected: {health_data.get('vectorizer_connected', False)}")
            else:
                print(f"   ERROR: {response.status_code}")
                return
        except Exception as e:
            print(f"   ERROR: {e}")
            return
        
        # Test 2: Vectorizer query
        print("\n2. Testando query do vectorizer...")
        
        query = "me fale sobre o vectorizer"
        print(f"   Query: '{query}'")
        
        try:
            response = await client.post(
                f"{base_url}/api/chat",
                json={
                    "message": query,
                    "history": []
                }
            )
            
            if response.status_code == 200:
                data = response.json()
                search_results = data.get('search_results', 0)
                response_text = data.get('response', '')
                
                print(f"   Search results: {search_results}")
                print(f"   Processing time: {data.get('processing_time', 0):.2f}s")
                
                # Check if response mentions vectorizer collections
                vectorizer_collections_found = []
                for collection in ['vectorizer-source', 'vectorizer-docs', 'vectorizer-sdk']:
                    if collection in response_text:
                        vectorizer_collections_found.append(collection)
                
                if vectorizer_collections_found:
                    print(f"   SUCCESS: Found vectorizer collections: {vectorizer_collections_found}")
                    print(f"   Response preview: {response_text[:300]}...")
                else:
                    print(f"   ISSUE: No vectorizer collections found in response")
                    print(f"   Response preview: {response_text[:300]}...")
                    
            else:
                print(f"   ERROR: {response.status_code}")
                
        except Exception as e:
            print(f"   ERROR: {e}")
        
        print("\n=== TESTE FINAL CONCLUÍDO ===")

if __name__ == "__main__":
    asyncio.run(test_final_version())
