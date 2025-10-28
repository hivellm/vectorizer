@echo off
REM Script para executar os testes do SDK Python

echo 🧪 Executando testes do SDK Python...
cd /d "%~dp0"
python tests\run_tests.py %*

