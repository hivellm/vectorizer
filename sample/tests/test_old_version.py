#!/usr/bin/env python3
"""
Teste da Versão Antiga do BitNet
Testa o endpoint correto da versão antiga
"""

import asyncio
import httpx
import json
import time

async def test_old_version():
    """Testa a versão antiga do BitNet"""
    
    query = "o que e o vectorizer ?"
    
    print("=== TESTE DA VERSÃO ANTIGA (porta 15004) ===")
    print(f"Query: '{query}'")
    print("=" * 50)
    
    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            # Teste 1: Health check
            print("\n1. Health Check...")
            try:
                response = await client.get("http://localhost:15004/api/health")
                if response.status_code == 200:
                    health_data = response.json()
                    print(f"Status: {health_data.get('status', 'unknown')}")
                    print("OK")
                else:
                    print(f"ERROR: {response.status_code}")
            except Exception as e:
                print(f"ERROR: {e}")
            
            # Teste 2: Chat via WebSocket (simulado)
            print("\n2. Testando chat...")
            
            try:
                import websockets
                
                async with websockets.connect("ws://localhost:15004/ws") as websocket:
                    print("WebSocket connected")
                    
                    # Send test message
                    test_message = {
                        "type": "chat",
                        "message": query,
                        "history": []
                    }
                    
                    await websocket.send(json.dumps(test_message))
                    print("Message sent")
                    
                    # Listen for responses
                    response_count = 0
                    final_response = ""
                    
                    while response_count < 10:  # Max 10 responses
                        try:
                            response = await asyncio.wait_for(websocket.recv(), timeout=15.0)
                            data = json.loads(response)
                            
                            print(f"Response {response_count + 1}: {data.get('type', 'unknown')}")
                            
                            if data.get('type') == 'response':
                                final_response = data.get('response', '')
                                break
                                
                            response_count += 1
                            
                        except asyncio.TimeoutError:
                            print("Timeout")
                            break
                    
                    if final_response:
                        print(f"\nRESPOSTA DA VERSÃO ANTIGA:")
                        print(final_response)
                        
                        # Verificar se encontrou coleções do vectorizer
                        vectorizer_collections = []
                        for collection in ['vectorizer-source', 'vectorizer-docs', 'vectorizer-sdk']:
                            if collection in final_response:
                                vectorizer_collections.append(collection)
                        
                        if vectorizer_collections:
                            print(f"\nColeções do vectorizer encontradas: {vectorizer_collections}")
                        else:
                            print(f"\nNenhuma coleção do vectorizer encontrada")
                    else:
                        print("Nenhuma resposta final recebida")
                        
            except ImportError:
                print("WebSocket test skipped (websockets not installed)")
            except Exception as e:
                print(f"WebSocket error: {e}")
                
    except Exception as e:
        print(f"ERROR: {e}")

if __name__ == "__main__":
    asyncio.run(test_old_version())
