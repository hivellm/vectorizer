#!/bin/bash
#
# Script para testar correção do payload no Metal Native
# Executa testes de validação para garantir que dados estão sendo retornados
#

set -e

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  🧪 TESTE: Metal Native Payload Retrieval Fix              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Cores para output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Função para imprimir cabeçalhos
print_header() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

# Verificar se vectorizer está rodando
print_header "1. Verificando servidor Vectorizer"

if pgrep -x "vectorizer" > /dev/null; then
    echo -e "${GREEN}✓${NC} Servidor vectorizer está rodando"
    VECTORIZER_PID=$(pgrep -x "vectorizer")
    echo "  PID: $VECTORIZER_PID"
else
    echo -e "${RED}✗${NC} Servidor vectorizer NÃO está rodando"
    echo ""
    echo -e "${YELLOW}⚠️  Iniciando servidor...${NC}"
    
    # Iniciar servidor em background
    cd "$(dirname "$0")/.."
    nohup ./target/release/vectorizer > /tmp/vectorizer-test.log 2>&1 &
    VECTORIZER_PID=$!
    echo "  Aguardando inicialização..."
    sleep 3
    
    if pgrep -x "vectorizer" > /dev/null; then
        echo -e "${GREEN}✓${NC} Servidor iniciado com sucesso (PID: $VECTORIZER_PID)"
    else
        echo -e "${RED}✗${NC} Falha ao iniciar servidor"
        echo "  Verifique logs em: /tmp/vectorizer-test.log"
        exit 1
    fi
fi

# Verificar coleções disponíveis
print_header "2. Verificando coleções disponíveis"

COLLECTIONS=$(curl -s http://localhost:15002/api/v1/collections 2>/dev/null || echo "")

if [ -n "$COLLECTIONS" ]; then
    echo -e "${GREEN}✓${NC} API respondendo"
    echo ""
    echo "Coleções encontradas:"
    echo "$COLLECTIONS" | jq -r '.collections[] | "  - \(.name): \(.vector_count) vectors"' 2>/dev/null || echo "$COLLECTIONS"
else
    echo -e "${YELLOW}⚠️${NC}  API não respondeu (servidor pode ainda estar inicializando)"
fi

# Executar testes
print_header "3. Executando testes de payload"

echo "📋 Teste 1: Validação de payload em busca direta"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

cargo test --test test_metal_native_payload test_metal_native_payload_retrieval --features metal-native -- --nocapture

RESULT_1=$?

echo ""
echo "📋 Teste 2: Validação de estrutura de payload"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

cargo test --test test_metal_native_payload test_payload_structure --features metal-native -- --nocapture

RESULT_2=$?

# Resultado final
print_header "4. Resumo dos Testes"

if [ $RESULT_1 -eq 0 ] && [ $RESULT_2 -eq 0 ]; then
    echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                                                              ║${NC}"
    echo -e "${GREEN}║  ✓ TODOS OS TESTES PASSARAM!                                ║${NC}"
    echo -e "${GREEN}║                                                              ║${NC}"
    echo -e "${GREEN}║  Correções validadas:                                        ║${NC}"
    echo -e "${GREEN}║  ✓ Payloads sendo retornados corretamente                   ║${NC}"
    echo -e "${GREEN}║  ✓ Campo 'content' presente em todos os resultados          ║${NC}"
    echo -e "${GREEN}║  ✓ Intelligent search funcionando                           ║${NC}"
    echo -e "${GREEN}║  ✓ Multi-collection search funcionando                      ║${NC}"
    echo -e "${GREEN}║                                                              ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    exit 0
else
    echo -e "${RED}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║                                                              ║${NC}"
    echo -e "${RED}║  ✗ ALGUNS TESTES FALHARAM                                   ║${NC}"
    echo -e "${RED}║                                                              ║${NC}"
    echo -e "${RED}║  Verifique os logs acima para detalhes                      ║${NC}"
    echo -e "${RED}║                                                              ║${NC}"
    echo -e "${RED}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    if [ $RESULT_1 -ne 0 ]; then
        echo -e "${RED}✗${NC} Teste 1 (Payload Retrieval) falhou"
    fi
    
    if [ $RESULT_2 -ne 0 ]; then
        echo -e "${RED}✗${NC} Teste 2 (Payload Structure) falhou"
    fi
    
    echo ""
    exit 1
fi

