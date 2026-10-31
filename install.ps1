$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$Repo   = 'O-ELMA/syncrab'
$Target = 'x86_64-pc-windows-msvc'
$Asset  = "syncrab-$Target.zip"
$Base   = "https://github.com/$Repo/releases/latest/download"
$Dest   = Join-Path $env:LOCALAPPDATA 'syncrab'
$Bin    = Join-Path $Dest 'bin'

function Log($Message) { Write-Host "[syncrab] $Message" -ForegroundColor Cyan }
function Warn($Message) { Write-Host "[syncrab] $Message" -ForegroundColor Yellow }

foreach ($task in @('SyncrabWatcher', 'SyncrabBackup')) {
    & schtasks.exe /End /TN $task 2>&1 | Out-Null
}

$tmp = Join-Path ([IO.Path]::GetTempPath()) ("syncrab-" + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmp | Out-Null

try {
    Log "Looking for a prebuilt Windows build..."
    $haveAsset = $false
    try {
        Invoke-WebRequest "$Base/$Asset" -OutFile (Join-Path $tmp $Asset) -UseBasicParsing
        $haveAsset = $true
    }
    catch {
        Warn "No prebuilt Windows binary in the latest release."
    }

    if ($haveAsset) {
        Log "Installing the prebuilt binaries to $Bin..."
        Expand-Archive (Join-Path $tmp $Asset) -DestinationPath (Join-Path $tmp 'extract') -Force
        New-Item -ItemType Directory -Force $Bin | Out-Null
        Copy-Item (Join-Path $tmp 'extract/*') $Bin -Force
    }
    else {
        if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
            throw "No prebuilt Windows binary and cargo is missing. Install Rust from https://rustup.rs and re-run."
        }

        Log "Compiling from source (this takes a few minutes)..."
        & cargo install --git "https://github.com/$Repo" --root $Dest --bins --force
        if ($LASTEXITCODE -ne 0) { throw "cargo install failed" }
    }
}
finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}

$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if (($userPath -split ';') -notcontains $Bin) {
    [Environment]::SetEnvironmentVariable('Path', "$($userPath.TrimEnd(';'));$Bin", 'User')
    $env:Path = "$env:Path;$Bin"
    Log "Added $Bin to your user PATH."
}

$backupExe = Join-Path $Bin 'syncrab_b.exe'
$watchExe  = Join-Path $Bin 'syncrab_w.exe'

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
Write-Host "Installed syncrab, syncrab_b and syncrab_w in $Bin." -ForegroundColor Green
Write-Host "Open a new terminal, then run: syncrab"
Write-Host "Re-run this script any time to update."
