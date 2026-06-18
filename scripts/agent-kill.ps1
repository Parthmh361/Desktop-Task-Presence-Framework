# Stop orphaned DTPF dev processes (Vite on 1420, tauri dev, cargo run).
$ErrorActionPreference = "SilentlyContinue"

Write-Host "Stopping processes on port 1420..."
$lines = netstat -ano | Select-String ":1420"
foreach ($line in $lines) {
    $parts = ($line -split '\s+') | Where-Object { $_ -ne '' }
    if ($parts.Count -ge 5) {
        $pid = $parts[-1]
        if ($pid -match '^\d+$') {
            Stop-Process -Id $pid -Force -ErrorAction SilentlyContinue
        }
    }
}

Get-Process -Name "dtpf-agent","cargo","node" -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -like "*Desktop-Task-Presence-Framework*" -or $_.MainWindowTitle -like "*DTPF*" } |
    Stop-Process -Force -ErrorAction SilentlyContinue

Write-Host "Done."
