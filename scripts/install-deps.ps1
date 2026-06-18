# Installs DTPF development dependencies on Windows.
# Usage: .\scripts\install-deps.ps1
#   -SkipRust      Skip Rust installation
#   -SkipVsTools   Skip Visual Studio Build Tools check/install prompt

param(
    [switch]$SkipRust,
    [switch]$SkipVsTools
)

$ErrorActionPreference = "Stop"
$Root = Split-Path $PSScriptRoot -Parent

function Write-Step([string]$Message) {
    Write-Host "`n==> $Message" -ForegroundColor Cyan
}

function Test-Command([string]$Name) {
    return [bool](Get-Command $Name -ErrorAction SilentlyContinue)
}

function Add-CargoToPath {
    $cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
    if (Test-Path $cargoBin) {
        $env:PATH = "$cargoBin;$env:PATH"
    }
}

function Install-Rust {
    if (Test-Command "cargo") {
        Write-Host "Rust already installed: $(cargo --version)"
        return $true
    }

    Write-Step "Installing Rust (rustup)"
    $rustup = Join-Path $env:TEMP "rustup-init.exe"
    Write-Host "Downloading rustup-init..."
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustup -UseBasicParsing

    Write-Host "Running rustup (this may take a few minutes)..."
    & $rustup -y --default-toolchain stable
    if ($LASTEXITCODE -ne 0) {
        Write-Host "rustup installation failed." -ForegroundColor Red
        return $false
    }

    Remove-Item $rustup -Force -ErrorAction SilentlyContinue
    Add-CargoToPath

    if (Test-Command "cargo") {
        Write-Host "Rust installed: $(cargo --version)" -ForegroundColor Green
        return $true
    }

    Write-Host "Rust installed but cargo not on PATH yet. Restart your terminal and run:" -ForegroundColor Yellow
    Write-Host '  Add to PATH: %USERPROFILE%\.cargo\bin'
    return $false
}

function Test-MsvcTools {
    if (Test-Command "cl") { return $true }
    if (Test-Command "link") { return $true }

    $vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vswhere) {
        $installPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
        if ($installPath) { return $true }
    }

    return $false
}

function Install-VsBuildTools {
    if (Test-MsvcTools) {
        Write-Host "MSVC build tools detected."
        return $true
    }

    Write-Host "MSVC C++ build tools not found (required for Tauri on Windows)." -ForegroundColor Yellow

    if (Test-Command "winget") {
        Write-Step "Installing Visual Studio 2022 Build Tools via winget"
        Write-Host "This is a large download and may take 10+ minutes..."
        winget install Microsoft.VisualStudio.2022.BuildTools `
            --accept-package-agreements --accept-source-agreements `
            --override '--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended'

        if (Test-MsvcTools) {
            Write-Host "Build tools installed." -ForegroundColor Green
            return $true
        }
    }

    Write-Host ""
    Write-Host "Install Visual Studio Build Tools manually:" -ForegroundColor Yellow
    Write-Host "  1. https://visualstudio.microsoft.com/visual-cpp-build-tools/"
    Write-Host '  2. Select workload: Desktop development with C++'
    Write-Host "  3. Restart terminal after install"
    Write-Host ""
    return $false
}

function Test-WebView2 {
    $key = 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'
    return Test-Path $key
}

Write-Host "DTPF dependency installer (Windows)" -ForegroundColor Green
Write-Host "Project root: $Root"

# Node.js
Write-Step "Checking Node.js"
if (-not (Test-Command "node")) {
    Write-Host "Node.js not found. Install Node.js 20+ from https://nodejs.org/" -ForegroundColor Red
    exit 1
}
Write-Host "Node: $(node --version)"

# pnpm via corepack
Write-Step "Checking pnpm"
if (-not (Test-Command "pnpm")) {
    Write-Host "Enabling pnpm via corepack..."
    corepack enable
    corepack prepare pnpm@10.20.0 --activate
}
if (-not (Test-Command "pnpm")) {
    Write-Host "pnpm not found. Run: corepack enable && corepack prepare pnpm@latest --activate" -ForegroundColor Red
    exit 1
}
Write-Host "pnpm: $(pnpm --version)"

# JS / workspace packages
Write-Step "Installing npm workspace packages and building SDK"
Push-Location $Root
try {
    node "$PSScriptRoot\setup.mjs"
    node "$PSScriptRoot\fix-windows-icons.mjs"
} finally {
    Pop-Location
}

# Rust
if (-not $SkipRust) {
    Write-Step "Checking Rust"
    Add-CargoToPath
    $rustOk = Install-Rust
    if ($rustOk) {
        Push-Location (Join-Path $Root "apps\desktop-agent\src-tauri")
        try {
            Write-Host "Running cargo check..."
            cargo check
            if ($LASTEXITCODE -ne 0) {
                Write-Host "cargo check failed - see errors above." -ForegroundColor Yellow
            }
        } finally {
            Pop-Location
        }
    }
}

# MSVC
if (-not $SkipVsTools) {
    Write-Step "Checking MSVC build tools"
    Install-VsBuildTools | Out-Null
}

# WebView2
Write-Step "Checking WebView2 Runtime"
if (Test-WebView2) {
    Write-Host "WebView2 Runtime: OK"
} else {
    Write-Host "WebView2 Runtime not detected. Install from:" -ForegroundColor Yellow
    Write-Host "  https://developer.microsoft.com/microsoft-edge/webview2/"
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "Setup complete!" -ForegroundColor Green
Write-Host "  Demo web app:  pnpm demo:dev"
Write-Host "  Desktop agent: pnpm agent:dev"
Write-Host ""
Write-Host "If cargo was just installed, restart this terminal first." -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
