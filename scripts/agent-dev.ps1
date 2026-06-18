# Start the Tauri desktop agent (Windows).
# Adds Cargo and Node.js to PATH for this session — no terminal restart needed.
$ErrorActionPreference = "Stop"

$Root = Split-Path $PSScriptRoot -Parent
$AgentDir = Join-Path $Root "apps\desktop-agent"

function Add-ToPathIfExists([string]$Dir) {
    if ($Dir -and (Test-Path $Dir) -and ($env:PATH -notlike "*$Dir*")) {
        $env:PATH = "$Dir;$env:PATH"
    }
}

Add-ToPathIfExists (Join-Path $env:USERPROFILE ".cargo\bin")
Add-ToPathIfExists "${env:ProgramFiles}\nodejs"
Add-ToPathIfExists "${env:ProgramFiles(x86)}\nodejs"
Add-ToPathIfExists (Join-Path $env:USERPROFILE "AppData\Roaming\npm")
Add-ToPathIfExists (Join-Path $env:LOCALAPPDATA "fnm_multishells")
Add-ToPathIfExists (Join-Path $env:USERPROFILE "scoop\shims")

if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
    Write-Host "Node.js not found on PATH." -ForegroundColor Red
    Write-Host "Install from https://nodejs.org/ and add it to your system PATH."
    Write-Host "Typical location: C:\Program Files\nodejs"
    exit 1
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Rust (cargo) not found on PATH." -ForegroundColor Red
    Write-Host "Run: pnpm install-deps"
    exit 1
}

$TauriCandidates = @(
    (Join-Path $AgentDir "node_modules\.bin\tauri.CMD")
    (Join-Path $AgentDir "node_modules\.bin\tauri.cmd")
    (Join-Path $Root "node_modules\.bin\tauri.CMD")
)
$Tauri = $TauriCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1

if (-not $Tauri) {
    Write-Host "Tauri CLI not found. Run: pnpm install" -ForegroundColor Red
    exit 1
}

Write-Host "Using node $(node --version), cargo $(cargo --version)" -ForegroundColor DarkGray

Push-Location $AgentDir
try {
    & $Tauri dev
    exit $LASTEXITCODE
} finally {
    Pop-Location
}
