#!/usr/bin/env python3
"""
Teste da Nova Implementação do BitNet
"""

import asyncio
import httpx
import json
import time

async def test_bitnet_v2():
    """Testa a nova implementação do BitNet"""
    base_url = "http://localhost:15005"
    
    print("=== TESTANDO BITNET SERVER V2.0 ===")
    
    async with httpx.AsyncClient(timeout=30.0) as client:
        # Test 1: Health check
        print("\n1. Testando health check...")
        try:
            response = await client.get(f"{base_url}/api/health")
            if response.status_code == 200:
                health_data = response.json()
                print(f"OK Health check OK: {health_data}")
            else:
                print(f"ERROR Health check failed: {response.status_code}")
                return
        except Exception as e:
            print(f"ERROR Health check error: {e}")
            return
        
        # Test 2: Chat API
        print("\n2. Testando chat API...")
        test_queries = [
            "me fale sobre o vectorizer",
            "como funciona a busca no vectorizer",
            "quais são as principais funcionalidades"
        ]
        
        for query in test_queries:
            print(f"\n   Query: '{query}'")
            try:
                start_time = time.time()
                
                response = await client.post(
                    f"{base_url}/api/chat",
                    json={
                        "message": query,
                        "history": []
                    }
                )
                
                if response.status_code == 200:
                    chat_data = response.json()
                    processing_time = time.time() - start_time
                    
                    print(f"   OK Response received in {processing_time:.2f}s")
                    print(f"   Search results: {chat_data.get('search_results', 0)}")
                    print(f"   Context used: {chat_data.get('context_used', False)}")
                    print(f"   Response: {chat_data.get('response', '')[:200]}...")
                else:
                    print(f"   ERROR Chat failed: {response.status_code}")
                    print(f"   Error: {response.text}")
                    
            except Exception as e:
                print(f"   ERROR Chat error: {e}")
        
        # Test 3: WebSocket (simplified)
        print("\n3. Testando WebSocket...")
        try:
            import websockets
            
            async with websockets.connect(f"ws://localhost:15005/ws") as websocket:
                print("   OK WebSocket connected")
                
                # Send test message
                test_message = {
                    "type": "chat",
                    "message": "me fale sobre o vectorizer",
                    "history": []
                }
                
                await websocket.send(json.dumps(test_message))
                print("   Message sent")
                
                # Listen for responses
                response_count = 0
                while response_count < 5:  # Max 5 responses
                    try:
                        response = await asyncio.wait_for(websocket.recv(), timeout=10.0)
                        data = json.loads(response)
                        
                        print(f"   Response {response_count + 1}: {data.get('type', 'unknown')}")
                        
                        if data.get('type') == 'response':
                            print(f"   Final response: {data.get('response', '')[:200]}...")
                            break
                            
                        response_count += 1
                        
                    except asyncio.TimeoutError:
                        print("   TIMEOUT WebSocket timeout")
                        break
                        
        except ImportError:
            print("   WARNING WebSocket test skipped (websockets not installed)")
        except Exception as e:
            print(f"   ERROR WebSocket error: {e}")

if __name__ == "__main__":
    asyncio.run(test_bitnet_v2())
