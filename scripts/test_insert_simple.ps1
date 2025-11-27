# Teste simples de inserção
$collection = "test_insert"
$vector = @()
1..384 | ForEach-Object { $vector += 0.1 }

$points = @()
1..5 | ForEach-Object { 
    $points += @{
        id = "test_vec$_"
        vector = $vector
    }
}

$json = @{
    points = $points
} | ConvertTo-Json -Depth 10

Write-Host "=== Enviando 5 vetores ==="
Write-Host "Collection: $collection"
Write-Host "JSON size: $($json.Length) bytes"

$start = Get-Date
try {
    $response = Invoke-RestMethod -Uri "http://localhost:15002/qdrant/collections/$collection/points" `
        -Method PUT `
        -ContentType "application/json" `
        -Body $json `
        -TimeoutSec 10
    
    $end = Get-Date
    $duration = ($end - $start).TotalSeconds
    
    Write-Host "✅ Sucesso em $duration segundos"
    Write-Host "Response: $($response | ConvertTo-Json -Compress)"
} catch {
    $end = Get-Date
    $duration = ($end - $start).TotalSeconds
    Write-Host "❌ Erro após $duration segundos: $($_.Exception.Message)"
}

Write-Host "`n=== Verificando collection ==="
try {
    $collectionInfo = Invoke-RestMethod -Uri "http://localhost:15002/collections/$collection"
    Write-Host "Vector count: $($collectionInfo.vector_count)"
} catch {
    Write-Host "Erro ao verificar: $($_.Exception.Message)"
}

