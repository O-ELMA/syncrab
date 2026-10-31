$ErrorActionPreference = 'Stop'

$Dest = Join-Path $env:LOCALAPPDATA 'syncrab'

function Log($Message) { Write-Host "[syncrab] $Message" -ForegroundColor Cyan }
function Warn($Message) { Write-Host "[syncrab] $Message" -ForegroundColor Yellow }

Log "Removing scheduled tasks..."
& schtasks.exe /Delete /TN SyncrabBackup /F 2>&1 | Out-Null
Unregister-ScheduledTask -TaskName 'SyncrabWatcher' -Confirm:$false -ErrorAction SilentlyContinue

Log "Removing binaries and database from $Dest..."
if (Test-Path $Dest) {
    Remove-Item -Recurse -Force $Dest
} else {
    Warn "$Dest not found."
}

$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
$entries = @($userPath -split ';' | Where-Object { $_ -and $_ -ne $Dest })
if ($entries.Count -ne ($userPath -split ';' | Where-Object { $_ }).Count) {
    [Environment]::SetEnvironmentVariable('Path', ($entries -join ';'), 'User')
    Log "Removed $Dest from your user PATH."
}

Write-Host ""
Write-Host "Uninstallation complete." -ForegroundColor Green
