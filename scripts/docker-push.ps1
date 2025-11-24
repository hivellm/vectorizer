# Script para fazer push da imagem Docker para Docker Hub
# Uso: .\scripts\docker-push.ps1 -Username SEU_USERNAME -Tag 1.5.0

param(
    [Parameter(Mandatory=$true)]
    [string]$Username,
    
    [Parameter(Mandatory=$false)]
    [string]$Tag = "latest",
    
    [Parameter(Mandatory=$false)]
    [string]$Repository = "vectorizer"
)

$ImageName = "vectorizer"
$FullTag = "${Username}/${Repository}:${Tag}"
$SourceTag = "${ImageName}:${Tag}"

Write-Host "🚀 Preparando push para Docker Hub..." -ForegroundColor Cyan
Write-Host "   Username: $Username" -ForegroundColor Yellow
Write-Host "   Repository: $Repository" -ForegroundColor Yellow
Write-Host "   Tag: $Tag" -ForegroundColor Yellow
Write-Host "   Tag completa: $FullTag" -ForegroundColor Yellow
Write-Host ""

# Verificar se a imagem existe
Write-Host "📦 Verificando se a imagem existe..." -ForegroundColor Cyan
$imageExists = docker images -q "${ImageName}:${Tag}" 2>$null
if (-not $imageExists) {
    Write-Host "❌ Imagem ${SourceTag} não encontrada!" -ForegroundColor Red
    Write-Host "   Construa a imagem primeiro com:" -ForegroundColor Yellow
    Write-Host "   docker build -t ${SourceTag} ." -ForegroundColor Yellow
    exit 1
}

Write-Host "✅ Imagem encontrada: ${SourceTag}" -ForegroundColor Green
Write-Host ""

# Criar tag com o formato correto para Docker Hub
Write-Host "🏷️  Criando tag para Docker Hub..." -ForegroundColor Cyan
docker tag "${SourceTag}" "${FullTag}"
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Erro ao criar tag!" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Tag criada: ${FullTag}" -ForegroundColor Green
Write-Host ""

# Verificar login
Write-Host "🔐 Verificando login no Docker Hub..." -ForegroundColor Cyan
$loginCheck = docker info 2>&1 | Select-String -Pattern "Username"
if (-not $loginCheck) {
    Write-Host "⚠️  Você precisa fazer login primeiro!" -ForegroundColor Yellow
    Write-Host "   Execute: docker login" -ForegroundColor Yellow
    Write-Host ""
    $login = Read-Host "Deseja fazer login agora? (s/n)"
    if ($login -eq "s" -or $login -eq "S") {
        docker login
        if ($LASTEXITCODE -ne 0) {
            Write-Host "❌ Login falhou!" -ForegroundColor Red
            exit 1
        }
    } else {
        Write-Host "❌ Login necessário para fazer push!" -ForegroundColor Red
        exit 1
    }
}

Write-Host "✅ Login verificado" -ForegroundColor Green
Write-Host ""

# Fazer push
Write-Host "📤 Fazendo push para Docker Hub..." -ForegroundColor Cyan
docker push "${FullTag}"
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Erro ao fazer push!" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "✅ Push concluído com sucesso!" -ForegroundColor Green
Write-Host "   Imagem disponível em: docker.io/${FullTag}" -ForegroundColor Cyan
Write-Host ""
Write-Host "Para usar a imagem:" -ForegroundColor Yellow
Write-Host "   docker pull ${FullTag}" -ForegroundColor White
Write-Host "   docker run -d -p 15002:15002 ${FullTag}" -ForegroundColor White

