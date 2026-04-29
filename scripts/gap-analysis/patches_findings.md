# docs/patches/ vs CHANGELOG vs código atual

41 arquivos `v*.md` revisados (v0.1 → v2.4)
~45 features "Added" headline extraídas

| Status | Count |
|---|---|
| Ainda no código | 42 |
| Removidas explicitamente em patch posterior | 3 |
| **Silenciosamente regressas** | **7** ← crítico |
| Em patches mas faltando no CHANGELOG.md | 35+ |

## 🚨 Regressões silenciosas

### #1: Discovery pipeline incompleto no MCP
- v0.6.0 prometeu: "All 38 MCP tools accessible via UMICP", incluindo 9 discovery tools (discover, broad_discovery, semantic_focus, compress_evidence, build_answer_plan, render_llm_prompt, promote_readme)
- **Hoje**: discovery functions existem em `src/discovery/` E em REST API, MAS apenas 2 tools expostos no MCP (`filter_collections`, `expand_queries`)
- 7 functions inalcançáveis via MCP — agentes precisam usar REST diretamente

### #2: Batch operations rebaixadas em v1.0.0
- v0.6/v0.18 prometeram batch_insert/search/update/delete_texts/vectors como MCP tools
- v1.0.0 removeu intencionalmente do MCP (justificativa: "agents podem fazer loop")
- Removal documentado MAS contradiz o claim de "38 tools" original

### #3: `get_file_summary` removido sem REST replacement
- v0.6/v0.18 prometeram como MCP tool
- v1.0.0 removeu, sugerindo `get_file_chunks` como alternativa
- **Hoje**: não está nem no MCP nem no REST. Substituto tem semântica diferente

## Claim inflado
**v0.6.0 anunciou "38 MCP tools"** — contagem real entregue: ~19 tools.
Inflação veio de:
- 9 discovery operations contadas como tools separadas → na verdade entregue como 1 pipeline + 7 fns internas + 2 MCP tools
- 5 batch operations contadas mas depois removidas em v1.0
- Real hoje: ~19 MCP tools (handlers.rs)

## CHANGELOG.md drift
Root `CHANGELOG.md` é focado em v3.0.x e **NÃO menciona**:
- TLS/mTLS support (v2.0)
- Hybrid Search (v2.0)
- Rate Limiting (v2.0)
- Quantization caching
- HiveHub logging
- Collection Mapping
- Discovery pipeline architecture (só nos patches)

## Recomendações
1. Expor 7 discovery operations no MCP OU documentar como REST-only com nota retroativa em v0.6
2. Adicionar "Removed from MCP" section no v0.6.0 patch retroativamente OU migration guide v1.0
3. Refazer CHANGELOG.md root com seção "v2.0 Production Features" (TLS, Hybrid, RateLimit, multi-tenant)
4. Criar matriz de compatibilidade MCP vs REST vs SDK para evitar promise-delivery mismatch
5. Considerar `discover_full` como tool única em vez de 9 fragmentadas
