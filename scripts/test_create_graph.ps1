# Test script to create collection with graph enabled

$json = Get-Content 'scripts\create_graph_collection.json' -Raw

try {
    Write-Host "Creating collection with graph enabled..."
    $response = Invoke-RestMethod -Uri 'http://127.0.0.1:15002/collections' -Method Post -Body $json -ContentType 'application/json'
    Write-Host "Success! Collection created:"
    $response | ConvertTo-Json -Depth 5
    
    Write-Host "`nVerifying collection..."
    Start-Sleep -Seconds 1
    $info = Invoke-RestMethod -Uri 'http://127.0.0.1:15002/collections/graph-enabled-collection' -Method Get
    Write-Host "Collection info:"
    $info | ConvertTo-Json -Depth 5
} catch {
    Write-Host "Error: $($_.Exception.Message)"
    if ($_.Exception.Response) {
        $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
        $responseBody = $reader.ReadToEnd()
        Write-Host "Response: $responseBody"
    }
}

