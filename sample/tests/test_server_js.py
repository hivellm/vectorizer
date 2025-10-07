#!/usr/bin/env python3
"""
Teste do Server.js com Nova Versão do BitNet
Verifica se o server.js está usando corretamente a nova versão do BitNet
"""

import asyncio
import httpx
import json
import time

async def test_server_js():
    """Testa se o server.js está usando a nova versão do BitNet"""
    
    query = "o que e o vectorizer ?"
    
    print("=== TESTE DO SERVER.JS COM NOVA VERSÃO ===")
    print(f"Query: '{query}'")
    print("=" * 50)
    
    # Teste 1: Health check do server.js
    print("\n1. Health Check do Server.js...")
    print("-" * 40)
    
    try:
        async with httpx.AsyncClient(timeout=10.0) as client:
            response = await client.get("http://localhost:15004/api/health")
            if response.status_code == 200:
                health_data = response.json()
                print(f"Status: {health_data.get('status', 'unknown')}")
                print(f"Vectorizer: {health_data.get('vectorizer', False)}")
                print(f"BitNet: {health_data.get('bitnet', False)}")
                print(f"Model: {health_data.get('model', False)}")
                print("RESULT: OK")
            else:
                print(f"ERROR: Health check failed with status {response.status_code}")
                print("RESULT: FAILED")
                return
    except Exception as e:
        print(f"ERROR: Health check error: {e}")
        print("RESULT: FAILED")
        return
    
    # Teste 2: Chat via API do server.js
    print("\n2. Chat via API do Server.js...")
    print("-" * 40)
    
    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            start_time = time.time()
            
            response = await client.post(
                "http://localhost:15004/api/chat",
                json={
                    "message": query,
                    "history": []
                }
            )
            
            processing_time = time.time() - start_time
            
            if response.status_code == 200:
                data = response.json()
                response_text = data.get('response', '')
                search_results = data.get('searchResults', [])
                
                print(f"Status: OK")
                print(f"Search results: {len(search_results)}")
                print(f"Processing time: {processing_time:.2f}s")
                print(f"\nRESPOSTA:")
                print(response_text[:500] + "..." if len(response_text) > 500 else response_text)
                
                # Verificar se encontrou coleções do vectorizer
                vectorizer_collections = []
                for collection in ['vectorizer-source', 'vectorizer-docs', 'vectorizer-sdk']:
                    if collection in response_text:
                        vectorizer_collections.append(collection)
                
                if vectorizer_collections:
                    print(f"\nColeções do vectorizer encontradas: {vectorizer_collections}")
                    print("SUCCESS: Server.js está usando a nova versão corretamente!")
                    print("RESULT: OK")
                else:
                    print(f"\nNenhuma coleção do vectorizer encontrada")
                    print("WARNING: Pode estar usando versão antiga ou sem busca inteligente")
                    print("RESULT: PARTIAL")
                    
            else:
                print(f"ERROR: Chat API failed with status {response.status_code}")
                print(f"Response: {response.text}")
                print("RESULT: FAILED")
                
    except Exception as e:
        print(f"ERROR: Chat API error: {e}")
        print("RESULT: FAILED")
    
    # Teste 3: Comparação com nova versão direta
    print("\n\n3. COMPARAÇÃO COM NOVA VERSÃO DIRETA")
    print("-" * 40)
    
    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            start_time = time.time()
            
            response = await client.post(
                "http://localhost:15006/api/chat",
                json={
                    "message": query,
                    "history": []
                }
            )
            
            processing_time = time.time() - start_time
            
            if response.status_code == 200:
                data = response.json()
                response_text = data.get('response', '')
                search_results = data.get('search_results', 0)
                
                print(f"Nova versão direta:")
                print(f"- Search results: {search_results}")
                print(f"- Processing time: {processing_time:.2f}s")
                print(f"- Response preview: {response_text[:200]}...")
                
                print(f"\nServer.js:")
                print(f"- Usa nova versão na porta 15006")
                print(f"- Deve ter resultados similares")
                
                print("\nCONCLUSÃO:")
                print("Se ambos retornarem resultados similares, o server.js está")
                print("funcionando corretamente com a nova versão do BitNet!")
                
            else:
                print(f"ERROR: Nova versão direta falhou: {response.status_code}")
                
    except Exception as e:
        print(f"ERROR: Teste da nova versão direta: {e}")
    
    print("\n" + "=" * 50)
    print("TESTE CONCLUÍDO")
    print("=" * 50)

if __name__ == "__main__":
    asyncio.run(test_server_js())
