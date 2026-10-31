$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$Repo   = 'O-ELMA/syncrab'
$Target = 'x86_64-pc-windows-msvc'
$Asset  = "syncrab-$Target.zip"
$Base   = "https://github.com/$Repo/releases/latest/download"
$Dest   = Join-Path $env:LOCALAPPDATA 'syncrab'

function Log($Message) { Write-Host "[syncrab] $Message" -ForegroundColor Cyan }
function Warn($Message) { Write-Host "[syncrab] $Message" -ForegroundColor Yellow }

foreach ($task in @('SyncrabWatcher', 'SyncrabBackup')) {
    & schtasks.exe /End /TN $task 2>&1 | Out-Null
}

$tmp = Join-Path ([IO.Path]::GetTempPath()) ("syncrab-" + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmp | Out-Null

try {
    Log "Downloading $Asset..."
    Invoke-WebRequest "$Base/$Asset" -OutFile (Join-Path $tmp $Asset) -UseBasicParsing
    Invoke-WebRequest "$Base/SHA256SUMS" -OutFile (Join-Path $tmp 'SHA256SUMS') -UseBasicParsing

    $expected = Get-Content (Join-Path $tmp 'SHA256SUMS') |
        Where-Object { $_ -match "\s$([regex]::Escape($Asset))$" } |
        ForEach-Object { ($_ -split '\s+')[0] }

    if (-not $expected) { throw "No checksum published for $Asset" }

    $zip = Join-Path $tmp $Asset
    $actual = (Get-FileHash $zip -Algorithm SHA256).Hash
    if ($actual.ToLower() -ne $expected.Trim().ToLower()) {
        throw "Checksum mismatch for $Asset"
    }

    Log "Installing to $Dest..."
    New-Item -ItemType Directory -Force $Dest | Out-Null
    Expand-Archive $zip -DestinationPath (Join-Path $tmp 'extract') -Force
    Copy-Item (Join-Path $tmp 'extract/*') $Dest -Force
}
finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}

$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if (($userPath -split ';') -notcontains $Dest) {
    [Environment]::SetEnvironmentVariable('Path', "$($userPath.TrimEnd(';'));$Dest", 'User')
    $env:Path = "$env:Path;$Dest"
    Log "Added $Dest to your user PATH."
}

$backupExe = Join-Path $Dest 'syncrab_b.exe'
$watchExe  = Join-Path $Dest 'syncrab_w.exe'

& schtasks.exe /Create /SC HOURLY /TN SyncrabBackup /TR "`"$backupExe`" --silent" /F | Out-Null
Log "Hourly backup scheduled via Task Scheduler."

$action = New-ScheduledTaskAction -Execute $watchExe
$trigger = New-ScheduledTaskTrigger -AtLogOn
$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries `
    -DontStopIfGoingOnBatteries -RestartCount 3 -RestartInterval (New-TimeSpan -Minutes 1)

Register-ScheduledTask -TaskName 'SyncrabWatcher' -Action $action -Trigger $trigger `
    -Settings $settings -Force | Out-Null
Start-ScheduledTask -TaskName 'SyncrabWatcher'

Log "Watcher started and set to run at logon."
Write-Host ""
Write-Host "Installed syncrab, syncrab_b and syncrab_w in $Dest." -ForegroundColor Green
Write-Host "Open a new terminal, then run: syncrab"
Write-Host "Re-run this script any time to update."
