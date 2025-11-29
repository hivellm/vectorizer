# Vectorizer Performance Benchmarks Runner
# Usage: .\scripts\run-benchmarks.ps1

$timestamp = Get-Date -Format "yyyy-MM-dd_HH-mm-ss"
$outputDir = "benchmark/reports"

Write-Host "🚀 Vectorizer Performance Benchmarks" -ForegroundColor Cyan
Write-Host "Timestamp: $timestamp" -ForegroundColor Gray
Write-Host ""

if (-not (Test-Path $outputDir)) {
    New-Item -ItemType Directory -Path $outputDir -Force | Out-Null
}

# 1. Filter Benchmark
Write-Host "📊 Running Filter Benchmark..." -ForegroundColor Yellow
cargo bench --bench filter_benchmark 2>&1 | Tee-Object -FilePath "$outputDir/filter_benchmark_$timestamp.txt" | Out-Null
Write-Host "✅ Filter benchmark completed" -ForegroundColor Green
Write-Host ""

# 2. gRPC vs REST Benchmark
Write-Host "📊 Running gRPC vs REST Benchmark..." -ForegroundColor Yellow
cargo run --release --bin benchmark_grpc_vs_rest --features benchmarks 2>&1 | Tee-Object -FilePath "$outputDir/grpc_vs_rest_$timestamp.txt" | Out-Null
Write-Host "✅ gRPC vs REST benchmark completed" -ForegroundColor Green
Write-Host ""

Write-Host "✅ All benchmarks completed!" -ForegroundColor Green
Write-Host "📄 Reports saved to: $outputDir" -ForegroundColor Cyan
