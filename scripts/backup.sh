#!/usr/bin/env bash
set -euo pipefail

# Usage:
#   ./scripts/backup.sh [DATA_DIR] [OUT_DIR]
# Defaults:
#   DATA_DIR = data
#   OUT_DIR  = backups

DATA_DIR="${1:-data}"
OUT_DIR="${2:-backups}"

if [[ ! -d "$DATA_DIR" ]]; then
  echo "Erro: diretório '$DATA_DIR' não encontrado" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"

TS=$(date +%Y%m%d_%H%M%S)
ARCHIVE_NAME="vectorizer_data_${TS}.tar.gz"

# Compacta todo o conteúdo do diretório de dados
tar -czf "${OUT_DIR}/${ARCHIVE_NAME}" -C "$DATA_DIR" .

echo "✅ Backup criado: ${OUT_DIR}/${ARCHIVE_NAME}"
#!/usr/bin/env bash
set -euo pipefail

DATA_DIR=
