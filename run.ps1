# AI-Cleaner 線上一鍵極速執行腳本 (PowerShell One-Liner)
$ErrorActionPreference = "SilentlyContinue"

Write-Host "================================================================" -ForegroundColor Cyan
Write-Host "   🚀 AI-Cleaner 線上免安裝極速加速器 (Web / Booster)" -ForegroundColor Green
Write-Host "================================================================" -ForegroundColor Cyan

$RepoUrl = "https://github.com/cconecone-cmyk/ai-cleaner"
$ExeUrl = "https://github.com/cconecone-cmyk/ai-cleaner/releases/latest/download/ai-cleaner.exe"
$TempDir = [System.IO.Path]::Combine($env:TEMP, "ai-cleaner")
$ExePath = [System.IO.Path]::Combine($TempDir, "ai-cleaner.exe")

if (!(Test-Path $TempDir)) {
    New-Item -ItemType Directory -Force -Path $TempDir | Out-Null
}

Write-Host "⏳ 正在從 GitHub 取得最新版 AI-Cleaner..." -ForegroundColor Yellow
try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    Invoke-WebRequest -Uri $ExeUrl -OutFile $ExePath -UseBasicParsing
} catch {
    Write-Host "❌ 無法從 Releases 下載預編譯檔，請至專案網頁手動下載：" -ForegroundColor Red
    Write-Host "👉 $RepoUrl" -ForegroundColor Cyan
    exit 1
}

Write-Host "✨ 下載完成，立即啟動 AI-Cleaner..." -ForegroundColor Green
Start-Process -FilePath $ExePath -ArgumentList "--web" -Wait
