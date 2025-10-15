#!/usr/bin/env python3
"""
Teste Simples do Server.js
Testa apenas o endpoint de chat para identificar o erro
"""

import asyncio
import httpx
import json

async def test_simple_chat():
    """Teste simples do chat"""
    
    print("=== TESTE SIMPLES DO SERVER.JS ===")
    print("=" * 40)
    
    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            # Teste simples
            response = await client.post(
                "http://localhost:15004/api/chat",
                json={
                    "message": "hello",
                    "history": []
                }
            )
            
            print(f"Status: {response.status_code}")
            print(f"Response: {response.text}")
            
            if response.status_code == 200:
                data = response.json()
                print(f"Success: {data.get('response', 'No response')}")
            else:
                print(f"Error: {response.text}")
                
    except Exception as e:
        print(f"Exception: {e}")

if __name__ == "__main__":
    asyncio.run(test_simple_chat())
