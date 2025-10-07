#!/usr/bin/env python3
"""
Teste da Nova Implementação - Verificar se está passando pelo BitNet
"""

import asyncio
import httpx
import json
import time

async def test_bitnet_integration():
    """Testa se a nova implementação está usando o BitNet corretamente"""
    
    print("=== TESTE DA NOVA IMPLEMENTAÇÃO - BITNET INTEGRATION ===")
    print("=" * 60)
    
    # Teste 1: Nova versão direta (bitnet_server.py)
    print("\n1. NOVA VERSÃO DIRETA (bitnet_server.py - porta 15006)")
    print("-" * 50)
    
    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            start_time = time.time()
            
            response = await client.post(
                "http://localhost:15006/api/chat",
                json={
                    "message": "o que e o vectorizer ?",
                    "history": []
                }
            )
            
            processing_time = time.time() - start_time
            
            if response.status_code == 200:
                data = response.json()
                response_text = data.get('response', '')
                search_results = data.get('search_results', 0)
                
                print(f"Status: OK")
                print(f"Search results: {search_results}")
                print(f"Processing time: {processing_time:.2f}s")
                print(f"\nRESPOSTA:")
                print(response_text)
                
                # Verificar se a resposta parece ter sido gerada pelo BitNet
                if "Encontrei informa" in response_text or "Baseado no contexto" in response_text:
                    print("\n✅ RESPOSTA PARECE TER SIDO GERADA PELO BITNET")
                else:
                    print("\n❌ RESPOSTA NÃO PARECE TER SIDO GERADA PELO BITNET")
                    
            else:
                print(f"ERROR: {response.status_code}")
                print(f"Response: {response.text}")
                
    except Exception as e:
        print(f"ERROR: {e}")
    
    # Teste 2: Server.js (porta 15004)
    print("\n\n2. SERVER.JS (porta 15004)")
    print("-" * 50)
    
    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            start_time = time.time()
            
            response = await client.post(
                "http://localhost:15004/api/chat",
                json={
                    "message": "o que e o vectorizer ?",
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
                print(response_text)
                
                # Verificar se a resposta parece ter sido gerada pelo BitNet
                if "Encontrei informa" in response_text or "Baseado no contexto" in response_text:
                    print("\n✅ SERVER.JS ESTÁ USANDO O BITNET CORRETAMENTE")
                else:
                    print("\n❌ SERVER.JS NÃO ESTÁ USANDO O BITNET CORRETAMENTE")
                    
            else:
                print(f"ERROR: {response.status_code}")
                print(f"Response: {response.text}")
                
    except Exception as e:
        print(f"ERROR: {e}")
    
    # Teste 3: Comparação de logs
    print("\n\n3. ANÁLISE DOS LOGS")
    print("-" * 50)
    
    print("Verificando logs do bitnet_server.py:")
    print("- Se aparecer 'Response generated in X.XXs' = BitNet está funcionando")
    print("- Se aparecer 'Chat request: X' = Requisição chegou")
    print("- Se aparecer 'Found X relevant results' = Busca funcionou")
    
    print("\nVerificando logs do server.js:")
    print("- Se aparecer 'BitNet generation error' = Problema na integração")
    print("- Se aparecer 'BitNet streaming error' = Problema no WebSocket")
    
    print("\n" + "=" * 60)
    print("CONCLUSÃO:")
    print("=" * 60)
    print("Se ambas as respostas forem similares e mencionarem")
    print("'Encontrei informações' ou 'Baseado no contexto',")
    print("então o BitNet está funcionando corretamente!")

if __name__ == "__main__":
    asyncio.run(test_bitnet_integration())
