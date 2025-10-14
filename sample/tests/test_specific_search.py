#!/usr/bin/env python3
"""
Teste Específico da Busca Inteligente
"""

import asyncio
import httpx
import json

async def test_intelligent_search():
    """Testa a busca inteligente diretamente"""
    base_url = "http://localhost:15005"
    
    print("=== TESTE ESPECÍFICO DA BUSCA INTELIGENTE ===")
    
    async with httpx.AsyncClient(timeout=30.0) as client:
        # Test chat API
        print("\n1. Testando chat API...")
        
        test_query = "me fale sobre o vectorizer"
        print(f"Query: '{test_query}'")
        
        try:
            response = await client.post(
                f"{base_url}/api/chat",
                json={
                    "message": test_query,
                    "history": []
                }
            )
            
            if response.status_code == 200:
                data = response.json()
                print(f"Status: OK")
                print(f"Search results: {data.get('search_results', 0)}")
                print(f"Context used: {data.get('context_used', False)}")
                print(f"Processing time: {data.get('processing_time', 0):.2f}s")
                
                response_text = data.get('response', '')
                print(f"\nResponse:")
                print(response_text)
                
                # Check if response contains vectorizer-related content
                if 'vectorizer' in response_text.lower():
                    print("\n*** SUCCESS: Response contains vectorizer content! ***")
                else:
                    print("\n*** ISSUE: Response does not contain vectorizer content ***")
                    
            else:
                print(f"Error: {response.status_code}")
                print(f"Response: {response.text}")
                
        except Exception as e:
            print(f"Error: {e}")

if __name__ == "__main__":
    asyncio.run(test_intelligent_search())
