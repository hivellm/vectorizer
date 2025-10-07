# Testes do BitNet Server v2.0

Esta pasta contém todos os testes para o BitNet Server v2.0.

## Arquivos de Teste

- `simple_test.py` - Teste básico e limpo (recomendado)
- `test_final_version.py` - Teste da versão final
- `test_bitnet_v2.py` - Teste da versão v2
- `test_collections.py` - Teste de coleções do vectorizer
- `test_specific_search.py` - Teste específico da busca
- `final_test.py` - Teste completo
- `comprehensive_test.py` - Teste abrangente (com emojis - pode ter problemas de encoding)

## Como Executar os Testes

```bash
# Navegar para a pasta de testes
cd f:\Node\hivellm\vectorizer\sample\tests

# Executar teste simples (recomendado)
python simple_test.py

# Executar outros testes
python test_final_version.py
python test_collections.py
```

## Pré-requisitos

- BitNet Server v2.0 rodando na porta 15006
- Vectorizer rodando na porta 15002
- Dependências Python instaladas (httpx, websockets)

## Resultados Esperados

### Teste Simples
- ✅ Health check: OK
- ✅ Vectorizer query: OK (deve encontrar coleções do vectorizer)
- ⚠️ Non-vectorizer query: PARTIAL (pode encontrar coleções do vectorizer também)

### Status do Servidor
- **URL**: http://localhost:15006
- **Interface Web**: http://localhost:15006
- **API**: http://localhost:15006/api/chat
- **WebSocket**: ws://localhost:15006/ws

## Notas

- O teste de query não relacionada ao vectorizer pode retornar resultados do vectorizer porque a busca é baseada em relevância semântica
- Todos os testes foram movidos para esta pasta para manter o diretório raiz limpo
- Use `simple_test.py` para testes rápidos e confiáveis
