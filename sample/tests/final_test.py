#!/usr/bin/env python3
"""
Teste Final da Nova Implementação do BitNet
Testa se a busca está funcionando corretamente com coleções do vectorizer
"""

import asyncio
import httpx
import json

async def final_test():
    """Teste final da implementação"""
    base_url = "http://localhost:15005"
    
    print("=== TESTE FINAL - BITNET SERVER V2.0 ===")
    
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
        
        # Test 2: Vectorizer-specific queries
        print("\n2. Testando queries específicas do vectorizer...")
        
        vectorizer_queries = [
            "me fale sobre o vectorizer",
            "como funciona o vectorizer",
            "quais são as funcionalidades do vectorizer",
            "documentação do vectorizer"
        ]
        
        for query in vectorizer_queries:
            print(f"\n   Query: '{query}'")
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
                    
                    # Check if response mentions vectorizer collections
                    vectorizer_collections_found = []
                    for collection in ['vectorizer-source', 'vectorizer-docs', 'vectorizer-sdk']:
                        if collection in response_text:
                            vectorizer_collections_found.append(collection)
                    
                    if vectorizer_collections_found:
                        print(f"   SUCCESS: Found vectorizer collections: {vectorizer_collections_found}")
                    else:
                        print(f"   ISSUE: No vectorizer collections found in response")
                        print(f"   Response preview: {response_text[:200]}...")
                        
                else:
                    print(f"   ERROR: {response.status_code}")
                    
            except Exception as e:
                print(f"   ERROR: {e}")
        
        # Test 3: Non-vectorizer query
        print("\n3. Testando query não relacionada ao vectorizer...")
        
        non_vectorizer_query = "me fale sobre governança"
        print(f"   Query: '{non_vectorizer_query}'")
        
        try:
            response = await client.post(
                f"{base_url}/api/chat",
                json={
                    "message": non_vectorizer_query,
                    "history": []
                }
            )
            
            if response.status_code == 200:
                data = response.json()
                search_results = data.get('search_results', 0)
                response_text = data.get('response', '')
                
                print(f"   Search results: {search_results}")
                
                # Check if response mentions governance collections
                gov_collections_found = []
                for collection in ['gov-', 'governance']:
                    if collection in response_text:
                        gov_collections_found.append(collection)
                
                if gov_collections_found:
                    print(f"   SUCCESS: Found governance collections")
                else:
                    print(f"   Response preview: {response_text[:200]}...")
                    
            else:
                print(f"   ERROR: {response.status_code}")
                
        except Exception as e:
            print(f"   ERROR: {e}")
        
        print("\n=== TESTE FINAL CONCLUÍDO ===")

if __name__ == "__main__":
    asyncio.run(final_test())
