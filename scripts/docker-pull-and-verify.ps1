# Baixa a imagem do Docker Hub, sobe o container e verifica se iniciou (health + logs).
# Uso: .\scripts\docker-pull-and-verify.ps1 [-Tag latest]
# Env opcional: $env:VECTORIZER_DOCKER_IMAGE (default: hivellm/vectorizer)

param([string]$Tag = "latest")

$ErrorActionPreference = "Stop"
$IMAGE = if ($env:VECTORIZER_DOCKER_IMAGE) { $env:VECTORIZER_DOCKER_IMAGE } else { "hivellm/vectorizer" }
$FULL_IMAGE = "${IMAGE}:${Tag}"
$CONTAINER_NAME = "vectorizer-verify-$PID"
$PORT = if ($env:VECTORIZER_PORT) { $env:VECTORIZER_PORT } else { "15002" }
$MAX_WAIT = 30

Write-Host "=============================================="
Write-Host "  Vectorizer - Pull e verificação de startup"
Write-Host "=============================================="
Write-Host "  Imagem: $FULL_IMAGE"
Write-Host "  Porta:  $PORT"
Write-Host "=============================================="
Write-Host ""

Write-Host "[1/4] Baixando imagem do Docker Hub..."
docker pull $FULL_IMAGE

Write-Host ""
Write-Host "[2/4] Subindo container (nome: $CONTAINER_NAME)..."
docker run -d --name $CONTAINER_NAME `
  -p "${PORT}:15002" `
  -e VECTORIZER_HOST=0.0.0.0 `
  -e VECTORIZER_PORT=15002 `
  $FULL_IMAGE

try {
  Write-Host ""
  Write-Host "[3/4] Aguardando serviço (até ${MAX_WAIT}s) e checando /health..."
  $ok = $false
  for ($i = 1; $i -le $MAX_WAIT; $i++) {
    try {
      $r = Invoke-WebRequest -Uri "http://127.0.0.1:${PORT}/health" -UseBasicParsing -TimeoutSec 2
      if ($r.StatusCode -eq 200) {
        Write-Host "  OK: /health retornou 200 após ${i}s"
        $ok = $true
        break
      }
    } catch {}
    Start-Sleep -Seconds 1
  }
  if (-not $ok) {
    Write-Host "  ERRO: /health não respondeu 200 em ${MAX_WAIT}s"
    Write-Host ""
    Write-Host "  Últimas linhas do log:"
    docker logs --tail 30 $CONTAINER_NAME 2>&1
    exit 1
  }

  Write-Host ""
  Write-Host "  Log (últimas 15 linhas):"
  docker logs --tail 15 $CONTAINER_NAME 2>&1
  Write-Host ""
  Write-Host "=============================================="
  Write-Host "  Container iniciou corretamente."
  Write-Host "  Health: http://127.0.0.1:${PORT}/health"
  Write-Host "  Dashboard: http://127.0.0.1:${PORT}"
  Write-Host "=============================================="
} finally {
  Write-Host ""
  Write-Host "[4/4] Parando e removendo container..."
  docker rm -f $CONTAINER_NAME 2>$null
}
