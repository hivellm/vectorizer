#!/bin/bash

# Script de restart do vectorizer
# Limpa a pasta data e reinicia o serviço

set -e  # Para o script em caso de erro

echo "🔄 Iniciando processo de restart do vectorizer..."

# Verifica se estamos no diretório correto
if [ ! -f "scripts/stop.sh" ] || [ ! -f "scripts/start.sh" ]; then
    echo "❌ Erro: Scripts stop.sh ou start.sh não encontrados."
    echo "   Certifique-se de executar este script a partir do diretório vectorizer/"
    exit 1
fi

# Limpa a pasta data
echo "🧹 Limpando pasta data..."
if [ -d "data" ]; then
    rm -rf data/*
    echo "✅ Pasta data limpa com sucesso"
else
    echo "⚠️  Pasta data não encontrada, criando..."
    mkdir -p data
fi

# Para o serviço
echo "🛑 Parando o serviço..."
./scripts/stop.sh

# Aguarda um momento para garantir que o serviço parou completamente
sleep 2

# Inicia o serviço
echo "🚀 Iniciando o serviço..."
./scripts/start.sh --workspace vectorize-workspace.yml

echo "✅ Restart concluído com sucesso!"
