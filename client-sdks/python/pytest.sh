#!/bin/bash
# Script para executar os testes do SDK Python

echo "🧪 Executando testes do SDK Python..."
cd "$(dirname "$0")"
python tests/run_tests.py "$@"

