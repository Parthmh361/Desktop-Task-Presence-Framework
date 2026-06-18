# DTPF development setup for Windows (PowerShell)
$ErrorActionPreference = "Stop"

Write-Host "Setting up DTPF development environment..." -ForegroundColor Cyan

if (-not (Get-Command pnpm -ErrorAction SilentlyContinue)) {
    Write-Host "pnpm not found. Enable it with:" -ForegroundColor Yellow
    Write-Host "  corepack enable"
    Write-Host "  corepack prepare pnpm@latest --activate"
    exit 1
}

node "$PSScriptRoot\setup.mjs"

Write-Host ""
Write-Host "Windows prerequisites (if the agent build fails):" -ForegroundColor Cyan
Write-Host "  1. Rust: https://rustup.rs"
Write-Host "  2. Visual Studio Build Tools -> Desktop development with C++"
Write-Host "  3. WebView2 Runtime (usually pre-installed on Windows 10/11)"
