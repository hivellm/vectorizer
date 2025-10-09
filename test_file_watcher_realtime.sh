#!/bin/bash

# Teste prático do File Watcher com reindexação automática
# Este script testa se o File Watcher detecta mudanças e reindexa automaticamente

echo "🧪 Teste do File Watcher - Reindexação Automática"
echo "=================================================="

# Usar diretório docs que já está no workspace
TEST_DIR="./docs"
mkdir -p "$TEST_DIR"

echo "📁 Usando diretório docs (já configurado no workspace): $TEST_DIR"

# Criar arquivo de teste inicial
TEST_FILE="$TEST_DIR/test_watcher_document.md"
cat > "$TEST_FILE" << 'EOF'
# Documento de Teste

Este é um documento de teste para verificar se o File Watcher
detecta mudanças e reindexa automaticamente.

## Conteúdo Inicial

- Item 1
- Item 2
- Item 3
EOF

echo "📄 Arquivo de teste criado: $TEST_FILE"

# Iniciar o servidor em background
echo "🚀 Iniciando servidor Vectorizer..."
RUST_LOG=info cargo run --bin vectorizer -- --host 0.0.0.0 --port 8080 > server.log 2>&1 &
SERVER_PID=$!

# Aguardar o servidor iniciar (precisa carregar 22 coleções)
echo "⏳ Aguardando servidor iniciar (carregando 22 coleções)..."
sleep 45

# Verificar se o servidor está rodando
if ! kill -0 $SERVER_PID 2>/dev/null; then
    echo "❌ Erro: Servidor não iniciou corretamente"
    cat server.log
    exit 1
fi

echo "✅ Servidor iniciado com PID: $SERVER_PID"

# Aguardar um pouco mais para o File Watcher inicializar
sleep 3

# Modificar o arquivo
echo "📝 Modificando arquivo de teste..."
cat > "$TEST_FILE" << 'EOF'
# Documento de Teste - MODIFICADO

Este é um documento de teste MODIFICADO para verificar se o File Watcher
detecta mudanças e reindexa automaticamente.

## Conteúdo Modificado

- Item 1 (modificado)
- Item 2 (modificado)
- Item 3 (modificado)
- Item 4 (novo)
EOF

echo "✅ Arquivo modificado"

# Aguardar o File Watcher processar a mudança
echo "⏳ Aguardando File Watcher processar mudança..."
sleep 10

# Verificar logs do servidor
echo "📋 Verificando logs do servidor..."
if grep -q "Successfully indexed file" server.log; then
    echo "✅ File Watcher detectou e reindexou o arquivo!"
else
    echo "⚠️  File Watcher pode não ter detectado a mudança"
fi

# Verificar se há erros
if grep -q "error\|Error\|ERROR" server.log; then
    echo "⚠️  Encontrados erros nos logs:"
    grep -i "error" server.log | tail -5
fi

# Criar novo arquivo
echo "📄 Criando novo arquivo..."
NEW_FILE="$TEST_DIR/test_watcher_new_document.md"
cat > "$NEW_FILE" << 'EOF'
# Novo Documento

Este é um novo documento criado para testar
a detecção de novos arquivos.
EOF

echo "✅ Novo arquivo criado: $NEW_FILE"

# Aguardar processamento
sleep 10

# Verificar se o novo arquivo foi indexado
if grep -q "test_watcher_new_document.md" server.log; then
    echo "✅ File Watcher detectou o novo arquivo!"
else
    echo "⚠️  File Watcher pode não ter detectado o novo arquivo"
fi

# Deletar arquivo
echo "🗑️  Deletando arquivo..."
rm "$TEST_FILE"
echo "✅ Arquivo deletado"

# Aguardar processamento
sleep 10

# Verificar se a deleção foi detectada
if grep -q "Removed file" server.log; then
    echo "✅ File Watcher detectou a deleção do arquivo!"
else
    echo "⚠️  File Watcher pode não ter detectado a deleção"
fi

# Parar o servidor
echo "🛑 Parando servidor..."
kill $SERVER_PID
wait $SERVER_PID 2>/dev/null

# Limpar arquivos de teste
echo "🧹 Limpando arquivos de teste..."
rm -f "$TEST_FILE" "$NEW_FILE" 2>/dev/null || true

# Mostrar logs do servidor antes de limpar
echo ""
echo "📋 LOGS DO SERVIDOR:"
echo "==================="
if [ -f server.log ]; then
    cat server.log
else
    echo "Log não encontrado"
fi

# Não limpar o log para debug
# rm -f server.log

echo ""
echo "🎉 Teste concluído!"
echo "Verifique os logs acima para confirmar se o File Watcher está funcionando corretamente."
