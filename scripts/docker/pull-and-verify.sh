#!/usr/bin/env bash
# Baixa a imagem do Docker Hub, sobe o container e verifica se iniciou (health + logs).
#
# Uso: ./scripts/docker-pull-and-verify.sh [tag]
#   tag = versão (default: latest). Ex.: v2.4.1, latest
#
# Env opcional: VECTORIZER_DOCKER_IMAGE (default: hivellm/vectorizer)
#   Ex.: VECTORIZER_DOCKER_IMAGE=seuuser/vectorizer ./scripts/docker-pull-and-verify.sh latest

set -e

IMAGE="${VECTORIZER_DOCKER_IMAGE:-hivehub/vectorizer}"
TAG="${1:-latest}"
FULL_IMAGE="${IMAGE}:${TAG}"
CONTAINER_NAME="vectorizer-verify-$$"
PORT="${VECTORIZER_PORT:-15002}"
MAX_WAIT=30

echo "=============================================="
echo "  Vectorizer - Pull e verificação de startup"
echo "=============================================="
echo "  Imagem: ${FULL_IMAGE}"
echo "  Porta:  ${PORT}"
echo "=============================================="
echo ""

echo "[1/4] Baixando imagem do Docker Hub..."
docker pull "${FULL_IMAGE}"

echo ""
echo "[2/4] Subindo container (nome: ${CONTAINER_NAME})..."
docker run -d --name "${CONTAINER_NAME}" \
  -p "${PORT}:15002" \
  -e VECTORIZER_HOST=0.0.0.0 \
  -e VECTORIZER_PORT=15002 \
  "${FULL_IMAGE}"

cleanup() {
  echo ""
  echo "[4/4] Parando e removendo container..."
  docker rm -f "${CONTAINER_NAME}" 2>/dev/null || true
}
trap cleanup EXIT

echo ""
echo "[3/4] Aguardando serviço (até ${MAX_WAIT}s) e checando /health..."
for i in $(seq 1 "${MAX_WAIT}"); do
  if curl -s -o /dev/null -w "%{http_code}" "http://127.0.0.1:${PORT}/health" 2>/dev/null | grep -q 200; then
    echo "  OK: /health retornou 200 após ${i}s"
    break
  fi
  if [ "$i" -eq "${MAX_WAIT}" ]; then
    echo "  ERRO: /health não respondeu 200 em ${MAX_WAIT}s"
    echo ""
    echo "  Últimas linhas do log:"
    docker logs --tail 30 "${CONTAINER_NAME}" 2>&1
    exit 1
  fi
  sleep 1
done

echo ""
echo "  Log (últimas 15 linhas):"
docker logs --tail 15 "${CONTAINER_NAME}" 2>&1
echo ""
echo "=============================================="
echo "  Container iniciou corretamente."
echo "  Health: http://127.0.0.1:${PORT}/health"
echo "  Dashboard: http://127.0.0.1:${PORT}"
echo "=============================================="
