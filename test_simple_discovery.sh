#!/bin/bash

# Teste simples do sistema de descoberta
echo "🧪 Teste Simples do Sistema de Descoberta"
echo "========================================="

# Configurações
TEST_DIR="/tmp/simple_discovery_test"
SERVER_PORT=8083

# Limpar ambiente anterior
echo "🧹 Limpando ambiente anterior..."
rm -rf "$TEST_DIR"
pkill -f vectorizer

# Criar diretório de teste
echo "📁 Criando diretório de teste..."
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# Criar apenas 2 arquivos de teste
echo "📄 Criando arquivos de teste..."
echo "Conteúdo do arquivo de teste 1" > "test1.txt"
echo "Conteúdo do arquivo de teste 2" > "test2.txt"

echo "📄 Arquivos criados:"
ls -la

# Iniciar servidor com logs detalhados
echo "🚀 Iniciando servidor Vectorizer..."
RUST_LOG=info /home/kleberson/Projetos/Skynet/vectorizer/target/release/vectorizer --port $SERVER_PORT &
SERVER_PID=$!

# Aguardar inicialização
echo "⏳ Aguardando inicialização do servidor..."
sleep 15

# Verificar se servidor está funcionando
echo "🔍 Verificando saúde do servidor..."
if curl -s http://localhost:$SERVER_PORT/health > /dev/null; then
    echo "✅ Servidor funcionando"
else
    echo "❌ Servidor não está respondendo"
    kill $SERVER_PID 2>/dev/null
    exit 1
fi

# Verificar coleções
echo "📊 Verificando coleções..."
COLLECTIONS=$(curl -s http://localhost:$SERVER_PORT/collections | grep -o '"total_collections":[0-9]*' | cut -d: -f2)
echo "📈 Total de coleções: $COLLECTIONS"

# Aguardar mais tempo para descoberta
echo "⏳ Aguardando sistema de descoberta..."
sleep 10

# Verificar novamente as coleções
COLLECTIONS_AFTER=$(curl -s http://localhost:$SERVER_PORT/collections | grep -o '"total_collections":[0-9]*' | cut -d: -f2)
echo "📈 Total de coleções após descoberta: $COLLECTIONS_AFTER"

if [ "$COLLECTIONS_AFTER" -gt "$COLLECTIONS" ]; then
    echo "✅ Sistema de descoberta funcionando - novas coleções criadas"
else
    echo "⚠️ Sistema de descoberta não criou novas coleções"
fi

# Parar servidor
echo "🛑 Parando servidor..."
kill $SERVER_PID 2>/dev/null
sleep 2

echo "✅ Teste concluído!"
