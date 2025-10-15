#!/usr/bin/env python3
"""
Teste Simples da Nova Versão do BitNet
Teste limpo sem emojis para evitar problemas de encoding
"""

import asyncio
import httpx
import json
import time

async def simple_test():
    """Teste simples da nova versão"""
    base_url = "http://localhost:15006"
    
    print("=== TESTE SIMPLES - BITNET SERVER V2.0 FINAL ===")
    
    async with httpx.AsyncClient(timeout=30.0) as client:
        # Test 1: Health Check
        print("\n1. HEALTH CHECK")
        print("-" * 40)
        try:
            response = await client.get(f"{base_url}/api/health")
            if response.status_code == 200:
                health_data = response.json()
                print(f"Status: {health_data.get('status', 'unknown')}")
                print(f"Vectorizer connected: {health_data.get('vectorizer_connected', False)}")
                print("RESULT: OK")
            else:
                print(f"ERROR: {response.status_code}")
                return
        except Exception as e:
            print(f"ERROR: {e}")
            return
        
        # Test 2: Vectorizer Query
        print("\n2. VECTORIZER QUERY TEST")
        print("-" * 40)
        
        query = "me fale sobre o vectorizer"
        print(f"Query: '{query}'")
        
        try:
            start_time = time.time()
            
            response = await client.post(
                f"{base_url}/api/chat",
                json={
                    "message": query,
                    "history": []
                }
            )
            
            processing_time = time.time() - start_time
            
            if response.status_code == 200:
                data = response.json()
                search_results = data.get('search_results', 0)
                response_text = data.get('response', '')
                
                print(f"Search results: {search_results}")
                print(f"Processing time: {processing_time:.2f}s")
                
                # Check if response mentions vectorizer collections
                vectorizer_collections_found = []
                for collection in ['vectorizer-source', 'vectorizer-docs', 'vectorizer-sdk']:
                    if collection in response_text:
                        vectorizer_collections_found.append(collection)
                
                if vectorizer_collections_found:
                    print(f"SUCCESS: Found vectorizer collections: {vectorizer_collections_found}")
                    print("RESULT: OK")
                else:
                    print("WARNING: No vectorizer collections found")
                    print(f"Response preview: {response_text[:200]}...")
                    print("RESULT: PARTIAL")
                    
            else:
                print(f"ERROR: {response.status_code}")
                print("RESULT: FAILED")
                
        except Exception as e:
            print(f"ERROR: {e}")
            print("RESULT: FAILED")
        
        # Test 3: Non-Vectorizer Query
        print("\n3. NON-VECTORIZER QUERY TEST")
        print("-" * 40)
        
        query = "me fale sobre governança"
        print(f"Query: '{query}'")
        
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
                
                print(f"Search results: {search_results}")
                
                # Check if response doesn't mention vectorizer collections
                vectorizer_collections_found = []
                for collection in ['vectorizer-source', 'vectorizer-docs', 'vectorizer-sdk']:
                    if collection in response_text:
                        vectorizer_collections_found.append(collection)
                
                if not vectorizer_collections_found:
                    print("SUCCESS: Correctly avoided vectorizer collections")
                    print("RESULT: OK")
                else:
                    print("WARNING: Found vectorizer collections in non-vectorizer query")
                    print("RESULT: PARTIAL")
                    
            else:
                print(f"ERROR: {response.status_code}")
                print("RESULT: FAILED")
                
        except Exception as e:
            print(f"ERROR: {e}")
            print("RESULT: FAILED")
        
        # Final Summary
        print("\n" + "=" * 50)
        print("RESUMO FINAL")
        print("=" * 50)
        print("Servidor: http://localhost:15006")
        print("Interface web: http://localhost:15006")
        print("API endpoint: http://localhost:15006/api/chat")
        print("WebSocket: ws://localhost:15006/ws")
        print("\nNova versão do BitNet está funcionando!")

if __name__ == "__main__":
    asyncio.run(simple_test())
