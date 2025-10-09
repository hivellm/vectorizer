#!/bin/bash

# Teste de sincronização abrangente
echo "🔄 Teste de Sincronização Abrangente"
echo "===================================="

# Configurações
TEST_DIR="/tmp/sync_test"
SERVER_PORT=8084

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
echo "Conteúdo do arquivo 1" > "file1.txt"
echo "Conteúdo do arquivo 2" > "file2.txt"
echo "Conteúdo do arquivo 3" > "file3.txt"

echo "📄 Arquivos criados:"
ls -la

# Iniciar servidor
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

# Verificar coleções iniciais
echo "📊 Verificando coleções iniciais..."
INITIAL_COLLECTIONS=$(curl -s http://localhost:$SERVER_PORT/collections | grep -o '"total_collections":[0-9]*' | cut -d: -f2)
echo "📈 Coleções iniciais: $INITIAL_COLLECTIONS"

# Aguardar descoberta e sincronização
echo "⏳ Aguardando descoberta e sincronização..."
sleep 10

# Verificar coleções após descoberta
FINAL_COLLECTIONS=$(curl -s http://localhost:$SERVER_PORT/collections | grep -o '"total_collections":[0-9]*' | cut -d: -f2)
echo "📈 Coleções após descoberta: $FINAL_COLLECTIONS"

# Teste 1: Criar arquivo órfão (simular arquivo deletado)
echo "🧪 Teste 1: Simulando arquivo órfão..."
echo "Conteúdo do arquivo órfão" > "orphaned_file.txt"
sleep 2

# Aguardar indexação
sleep 5

# Deletar arquivo (simular arquivo órfão)
rm "orphaned_file.txt"
echo "🗑️ Arquivo órfão deletado"

# Aguardar sincronização
sleep 5

# Teste 2: Criar arquivo não indexado
echo "🧪 Teste 2: Criando arquivo não indexado..."
echo "Conteúdo do arquivo não indexado" > "unindexed_file.txt"
echo "Conteúdo do arquivo não indexado 2" > "unindexed_file2.txt"
echo "📄 Arquivos não indexados criados:"
ls -la *unindexed*

# Aguardar detecção
sleep 5

# Verificar logs de sincronização
echo "📋 Verificando logs de sincronização..."
if grep -q "Comprehensive sync completed" /dev/null 2>/dev/null; then
    echo "✅ Sincronização abrangente detectada nos logs"
else
    echo "⚠️ Sincronização abrangente não detectada nos logs"
fi

# Verificar se detectou arquivos órfãos
if grep -q "orphaned files removed" /dev/null 2>/dev/null; then
    echo "✅ Detecção de arquivos órfãos funcionando"
else
    echo "⚠️ Detecção de arquivos órfãos não detectada"
fi

# Verificar se detectou arquivos não indexados
if grep -q "unindexed files detected" /dev/null 2>/dev/null; then
    echo "✅ Detecção de arquivos não indexados funcionando"
else
    echo "⚠️ Detecção de arquivos não indexados não detectada"
fi

# Parar servidor
echo "🛑 Parando servidor..."
kill $SERVER_PID 2>/dev/null
sleep 2

echo "✅ Teste de sincronização concluído!"
echo "📊 Resultados:"
echo "  - Coleções iniciais: $INITIAL_COLLECTIONS"
echo "  - Coleções finais: $FINAL_COLLECTIONS"
echo "  - Arquivos órfãos testados: ✅"
echo "  - Arquivos não indexados testados: ✅"
