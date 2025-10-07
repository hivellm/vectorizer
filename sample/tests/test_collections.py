#!/usr/bin/env python3
"""
Teste de Coleções do Vectorizer
Verifica quais coleções estão disponíveis e se contêm dados do vectorizer
"""

import asyncio
import httpx
import json

async def test_collections():
    """Testa as coleções disponíveis"""
    base_url = "http://localhost:15002"
    
    print("=== TESTANDO COLEÇÕES DO VECTORIZER ===")
    
    async with httpx.AsyncClient(timeout=30.0) as client:
        # List collections
        print("\n1. Listando todas as coleções...")
        try:
            response = await client.get(f"{base_url}/collections")
            response.raise_for_status()
            result = response.json()
            collections = result.get("collections", [])
            
            print(f"Total de coleções: {len(collections)}")
            
            # Filter vectorizer collections
            vectorizer_collections = []
            for col in collections:
                if 'vectorizer' in col.get('name', '').lower():
                    vectorizer_collections.append(col)
            
            print(f"\nColeções do vectorizer encontradas: {len(vectorizer_collections)}")
            for col in vectorizer_collections:
                name = col.get('name', '')
                doc_count = col.get('document_count', 0)
                print(f"  - {name}: {doc_count} documentos")
            
            # Test search in vectorizer collections
            if vectorizer_collections:
                print(f"\n2. Testando busca nas coleções do vectorizer...")
                
                test_query = "me fale sobre o vectorizer"
                
                for col in vectorizer_collections[:3]:  # Test first 3
                    collection_name = col.get('name', '')
                    print(f"\n   Testando {collection_name}...")
                    
                    try:
                        search_response = await client.post(
                            f"{base_url}/collections/{collection_name}/search/text",
                            json={
                                "query": test_query,
                                "limit": 2
                            },
                            timeout=10.0
                        )
                        
                        if search_response.status_code == 200:
                            search_result = search_response.json()
                            results = search_result.get("results", [])
                            
                            print(f"   Resultados encontrados: {len(results)}")
                            
                            for i, item in enumerate(results):
                                score = item.get("score", 0.0)
                                payload = item.get("payload", {})
                                content = payload.get("content", "") if isinstance(payload, dict) else ""
                                
                                print(f"     {i+1}. Score: {score:.3f}")
                                print(f"        Content: {content[:100]}...")
                                
                                # Check if content is relevant
                                if 'vectorizer' in content.lower() or 'rust' in content.lower() or 'embedding' in content.lower():
                                    print(f"        *** CONTEÚDO RELEVANTE! ***")
                        else:
                            print(f"   Erro na busca: {search_response.status_code}")
                            
                    except Exception as e:
                        print(f"   Erro: {e}")
            else:
                print("\nNenhuma coleção do vectorizer encontrada!")
                
        except Exception as e:
            print(f"Erro ao listar coleções: {e}")

if __name__ == "__main__":
    asyncio.run(test_collections())
