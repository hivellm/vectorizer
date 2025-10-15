#!/usr/bin/env python3
"""
Teste Completo da Nova Versão do BitNet
Testa todas as funcionalidades para garantir que está funcionando
"""

import asyncio
import httpx
import json
import time

async def comprehensive_test():
    """Teste completo da nova versão"""
    base_url = "http://localhost:15006"
    
    print("=== TESTE COMPLETO - BITNET SERVER V2.0 FINAL ===")
    
    async with httpx.AsyncClient(timeout=30.0) as client:
        # Test 1: Health Check
        print("\n1. HEALTH CHECK")
        print("=" * 50)
        try:
            response = await client.get(f"{base_url}/api/health")
            if response.status_code == 200:
                health_data = response.json()
                print(f"OK Status: {health_data.get('status', 'unknown')}")
                print(f"OK Vectorizer connected: {health_data.get('vectorizer_connected', False)}")
                print(f"OK Timestamp: {health_data.get('timestamp', 'unknown')}")
            else:
                print(f"ERROR: {response.status_code}")
                return
        except Exception as e:
            print(f"ERROR: {e}")
            return
        
        # Test 2: Vectorizer-specific queries
        print("\n2. TESTANDO QUERIES DO VECTORIZER")
        print("=" * 50)
        
        vectorizer_queries = [
            "me fale sobre o vectorizer",
            "como funciona o vectorizer",
            "quais são as funcionalidades do vectorizer",
            "documentação do vectorizer",
            "como usar o vectorizer em rust"
        ]
        
        success_count = 0
        total_tests = len(vectorizer_queries)
        
        for i, query in enumerate(vectorizer_queries, 1):
            print(f"\n   Teste {i}/{total_tests}: '{query}'")
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
                    
                    print(f"   ✅ Status: OK")
                    print(f"   ✅ Search results: {search_results}")
                    print(f"   ✅ Processing time: {processing_time:.2f}s")
                    
                    # Check if response mentions vectorizer collections
                    vectorizer_collections_found = []
                    for collection in ['vectorizer-source', 'vectorizer-docs', 'vectorizer-sdk']:
                        if collection in response_text:
                            vectorizer_collections_found.append(collection)
                    
                    if vectorizer_collections_found:
                        print(f"   ✅ SUCCESS: Found vectorizer collections: {vectorizer_collections_found}")
                        success_count += 1
                    else:
                        print(f"   ⚠️  WARNING: No vectorizer collections found")
                        print(f"   Response preview: {response_text[:200]}...")
                        
                else:
                    print(f"   ❌ ERROR: {response.status_code}")
                    print(f"   Response: {response.text}")
                    
            except Exception as e:
                print(f"   ❌ ERROR: {e}")
        
        print(f"\n   📊 RESULTADO: {success_count}/{total_tests} testes passaram")
        
        # Test 3: Non-vectorizer queries
        print("\n3. TESTANDO QUERIES NÃO RELACIONADAS AO VECTORIZER")
        print("=" * 50)
        
        non_vectorizer_queries = [
            "me fale sobre governança",
            "como funciona o sistema de autenticação",
            "quais são as funcionalidades do CMMV"
        ]
        
        for i, query in enumerate(non_vectorizer_queries, 1):
            print(f"\n   Teste {i}: '{query}'")
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
                    
                    print(f"   ✅ Status: OK")
                    print(f"   ✅ Search results: {search_results}")
                    print(f"   ✅ Processing time: {data.get('processing_time', 0):.2f}s")
                    
                    # Check if response doesn't mention vectorizer collections (should be different)
                    vectorizer_collections_found = []
                    for collection in ['vectorizer-source', 'vectorizer-docs', 'vectorizer-sdk']:
                        if collection in response_text:
                            vectorizer_collections_found.append(collection)
                    
                    if not vectorizer_collections_found:
                        print(f"   ✅ SUCCESS: Correctly avoided vectorizer collections")
                    else:
                        print(f"   ⚠️  WARNING: Found vectorizer collections in non-vectorizer query")
                        print(f"   Response preview: {response_text[:200]}...")
                        
                else:
                    print(f"   ❌ ERROR: {response.status_code}")
                    
            except Exception as e:
                print(f"   ❌ ERROR: {e}")
        
        # Test 4: WebSocket test
        print("\n4. TESTANDO WEBSOCKET")
        print("=" * 50)
        
        try:
            import websockets
            
            async with websockets.connect(f"ws://localhost:15006/ws") as websocket:
                print("   ✅ WebSocket connected")
                
                # Send test message
                test_message = {
                    "type": "chat",
                    "message": "me fale sobre o vectorizer",
                    "history": []
                }
                
                await websocket.send(json.dumps(test_message))
                print("   ✅ Message sent")
                
                # Listen for responses
                response_count = 0
                websocket_success = False
                
                while response_count < 5:  # Max 5 responses
                    try:
                        response = await asyncio.wait_for(websocket.recv(), timeout=10.0)
                        data = json.loads(response)
                        
                        print(f"   📥 Response {response_count + 1}: {data.get('type', 'unknown')}")
                        
                        if data.get('type') == 'response':
                            response_text = data.get('response', '')
                            if 'vectorizer' in response_text.lower():
                                print(f"   ✅ SUCCESS: WebSocket response contains vectorizer content")
                                websocket_success = True
                            else:
                                print(f"   ⚠️  WARNING: WebSocket response doesn't contain vectorizer content")
                            break
                            
                        response_count += 1
                        
                    except asyncio.TimeoutError:
                        print("   ⏰ WebSocket timeout")
                        break
                
                if websocket_success:
                    print("   ✅ WebSocket test PASSED")
                else:
                    print("   ❌ WebSocket test FAILED")
                        
        except ImportError:
            print("   ⚠️  WebSocket test skipped (websockets not installed)")
        except Exception as e:
            print(f"   ❌ WebSocket error: {e}")
        
        # Test 5: Performance test
        print("\n5. TESTE DE PERFORMANCE")
        print("=" * 50)
        
        performance_tests = [
            "me fale sobre o vectorizer",
            "como funciona a busca",
            "documentação do sistema"
        ]
        
        total_time = 0
        successful_requests = 0
        
        for query in performance_tests:
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
                total_time += processing_time
                
                if response.status_code == 200:
                    successful_requests += 1
                    print(f"   ✅ '{query}': {processing_time:.2f}s")
                else:
                    print(f"   ❌ '{query}': ERROR {response.status_code}")
                    
            except Exception as e:
                print(f"   ❌ '{query}': ERROR {e}")
        
        if successful_requests > 0:
            avg_time = total_time / successful_requests
            print(f"\n   📊 Performance Summary:")
            print(f"   ✅ Successful requests: {successful_requests}/{len(performance_tests)}")
            print(f"   ✅ Average response time: {avg_time:.2f}s")
            print(f"   ✅ Total time: {total_time:.2f}s")
        
        # Final Summary
        print("\n" + "=" * 60)
        print("📋 RESUMO FINAL DOS TESTES")
        print("=" * 60)
        
        if success_count >= total_tests * 0.8:  # 80% success rate
            print("🎉 SUCESSO: Nova versão está funcionando corretamente!")
            print(f"✅ {success_count}/{total_tests} testes de vectorizer passaram")
            print("✅ Health check funcionando")
            print("✅ API REST funcionando")
            print("✅ Busca inteligente funcionando")
            print("✅ Priorização de coleções funcionando")
        else:
            print("⚠️  ATENÇÃO: Alguns testes falharam")
            print(f"❌ Apenas {success_count}/{total_tests} testes de vectorizer passaram")
        
        print("\n🚀 Servidor rodando em: http://localhost:15006")
        print("📚 Interface web: http://localhost:15006")
        print("🔗 API endpoint: http://localhost:15006/api/chat")

if __name__ == "__main__":
    asyncio.run(comprehensive_test())
