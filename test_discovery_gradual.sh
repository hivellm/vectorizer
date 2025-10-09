#!/bin/bash

# Teste gradual do sistema de descoberta
echo "🧪 Teste Gradual do Sistema de Descoberta"
echo "=========================================="

# Configurações
TEST_DIR="/tmp/vectorizer_discovery_test"
SERVER_PORT=8082
LOG_FILE="discovery_gradual_test.log"

# Limpar ambiente anterior
echo "🧹 Limpando ambiente anterior..."
rm -rf "$TEST_DIR"
pkill -f vectorizer

# Criar diretório de teste
echo "📁 Criando diretório de teste..."
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# Criar arquivos de teste
echo "📄 Criando arquivos de teste..."
for i in {1..5}; do
    echo "Conteúdo do arquivo de teste $i" > "test_file_$i.txt"
done

echo "📄 Arquivos criados:"
ls -la

# Iniciar servidor
echo "🚀 Iniciando servidor Vectorizer..."
RUST_LOG=info ../target/release/vectorizer --port $SERVER_PORT > "../$LOG_FILE" 2>&1 &
SERVER_PID=$!

# Aguardar inicialização
echo "⏳ Aguardando inicialização do servidor..."
sleep 10

# Verificar se servidor está funcionando
echo "🔍 Verificando saúde do servidor..."
if curl -s http://localhost:$SERVER_PORT/health > /dev/null; then
    echo "✅ Servidor funcionando"
else
    echo "❌ Servidor não está respondendo"
    kill $SERVER_PID 2>/dev/null
    exit 1
fi

# Verificar logs de descoberta
echo "📊 Verificando logs de descoberta..."
if grep -q "Starting file discovery" "../$LOG_FILE"; then
    echo "✅ Sistema de descoberta iniciado"
    
    # Verificar se processou arquivos
    if grep -q "Processing file" "../$LOG_FILE"; then
        echo "✅ Arquivos sendo processados"
        
        # Contar arquivos processados
        PROCESSED_COUNT=$(grep -c "Processing file" "../$LOG_FILE")
        echo "📈 Arquivos processados: $PROCESSED_COUNT"
        
        # Verificar se indexou arquivos
        if grep -q "Indexed file" "../$LOG_FILE"; then
            INDEXED_COUNT=$(grep -c "Indexed file" "../$LOG_FILE")
            echo "✅ Arquivos indexados: $INDEXED_COUNT"
        else
            echo "⚠️ Nenhum arquivo foi indexado"
        fi
    else
        echo "⚠️ Nenhum arquivo foi processado"
    fi
else
    echo "⚠️ Sistema de descoberta não foi iniciado"
fi

# Teste de modificação em tempo real
echo "🔄 Testando modificação em tempo real..."
echo "Conteúdo modificado" > "test_file_1.txt"
sleep 5

# Verificar se detectou mudança
if grep -q "File change detected" "../$LOG_FILE"; then
    echo "✅ File Watcher detectou mudanças em tempo real"
else
    echo "⚠️ File Watcher não detectou mudanças"
fi

# Parar servidor
echo "🛑 Parando servidor..."
kill $SERVER_PID 2>/dev/null
sleep 2

# Mostrar resumo dos logs
echo "📋 Resumo dos logs:"
echo "==================="
grep -E "(discovery|Discovery|Processing file|Indexed file|File change detected)" "../$LOG_FILE" | tail -10

echo "✅ Teste concluído!"
echo "📄 Log completo disponível em: $LOG_FILE"
