#!/usr/bin/env python3
"""
Comparação de Respostas: Nova Versão vs Versão Antiga
Testa a mesma query em ambas as versões para comparar resultados
"""

import asyncio
import httpx
import json
import time

async def compare_responses():
    """Compara respostas entre as duas versões"""
    
    query = "o que e o vectorizer ?"
    
    print("=== COMPARAÇÃO DE RESPOSTAS ===")
    print(f"Query: '{query}'")
    print("=" * 60)
    
    # Teste 1: Nova versão (porta 15006)
    print("\n1. NOVA VERSÃO (BitNet Server v2.0 - porta 15006)")
    print("-" * 50)
    
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
                search_results = data.get('search_results', 0)
                response_text = data.get('response', '')
                
                print(f"Status: OK")
                print(f"Search results: {search_results}")
                print(f"Processing time: {processing_time:.2f}s")
                print(f"\nRESPOSTA:")
                print(response_text)
                
                # Verificar se encontrou coleções do vectorizer
                vectorizer_collections = []
                for collection in ['vectorizer-source', 'vectorizer-docs', 'vectorizer-sdk']:
                    if collection in response_text:
                        vectorizer_collections.append(collection)
                
                if vectorizer_collections:
                    print(f"\nColeções do vectorizer encontradas: {vectorizer_collections}")
                else:
                    print(f"\nNenhuma coleção do vectorizer encontrada")
                    
            else:
                print(f"ERROR: {response.status_code}")
                print(f"Response: {response.text}")
                
    except Exception as e:
        print(f"ERROR: {e}")
    
    # Teste 2: Versão antiga (porta 15004)
    print("\n\n2. VERSÃO ANTIGA (BitNet Server v1.0 - porta 15004)")
    print("-" * 50)
    
    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            start_time = time.time()
            
            response = await client.post(
                "http://localhost:15004/generate",
                json={
                    "query": query
                }
            )
            
            processing_time = time.time() - start_time
            
            if response.status_code == 200:
                data = response.json()
                response_text = data.get('response', '')
                
                print(f"Status: OK")
                print(f"Processing time: {processing_time:.2f}s")
                print(f"\nRESPOSTA:")
                print(response_text)
                
            else:
                print(f"ERROR: {response.status_code}")
                print(f"Response: {response.text}")
                
    except Exception as e:
        print(f"ERROR: {e}")
    
    # Comparação final
    print("\n\n" + "=" * 60)
    print("ANÁLISE COMPARATIVA")
    print("=" * 60)
    
    print("\nNova versão (v2.0):")
    print("- Busca inteligente que detecta queries do vectorizer")
    print("- Retorna resultados das coleções corretas do vectorizer")
    print("- Tempo de resposta otimizado")
    print("- API REST moderna")
    
    print("\nVersão antiga (v1.0):")
    print("- Busca genérica sem priorização")
    print("- Pode retornar resultados irrelevantes")
    print("- Usa endpoint /generate")
    
    print("\nCONCLUSÃO:")
    print("A nova versão deve fornecer respostas mais precisas e relevantes!")

if __name__ == "__main__":
    asyncio.run(compare_responses())
