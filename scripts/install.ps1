# Pasang Antigravity Island: salin island.exe, daftarkan "island" untuk Win+R, shortcut Start Menu, (opsional) autostart.
param([switch]$Autostart, [switch]$Uninstall)
$ErrorActionPreference = 'Stop'
$dir = Join-Path $env:LOCALAPPDATA 'AntigravityIsland'
$exe = Join-Path $dir 'island.exe'
$start = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs\Antigravity Island.lnk'
$boot = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs\Startup\Antigravity Island.lnk'

if ($Uninstall) {
  Get-Process island -ErrorAction SilentlyContinue | Stop-Process -Force
  Remove-Item $start, $boot -ErrorAction SilentlyContinue
  $p = [Environment]::GetEnvironmentVariable('Path', 'User') -split ';' | Where-Object { $_ -and $_ -ne $dir }
  [Environment]::SetEnvironmentVariable('Path', ($p -join ';'), 'User')
  Remove-Item $exe -ErrorAction SilentlyContinue
  Write-Host 'Dihapus (settings.json dibiarkan).'; exit
}

$src = @((Join-Path $PSScriptRoot '..\target\release\island.exe'), (Join-Path $PSScriptRoot 'island.exe')) | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $src) { throw 'island.exe tidak ditemukan. Jalankan dulu: cargo build --release' }
New-Item -ItemType Directory -Force $dir | Out-Null
Get-Process island -ErrorAction SilentlyContinue | Stop-Process -Force
Copy-Item $src $exe -Force

$p = [Environment]::GetEnvironmentVariable('Path', 'User')
if (($p -split ';') -notcontains $dir) { [Environment]::SetEnvironmentVariable('Path', ($p.TrimEnd(';') + ';' + $dir), 'User') }

$sh = New-Object -ComObject WScript.Shell
function Mk($path) { $l = $sh.CreateShortcut($path); $l.TargetPath = $exe; $l.WorkingDirectory = $dir; $l.Save() }
Mk $start
if ($Autostart) { Mk $boot }
Start-Process $exe
Write-Host "Terpasang. Tekan Win+R lalu ketik: island   (atau Ctrl+Alt+A saat sudah berjalan)"
if ($Autostart) { Write-Host 'Autostart aktif.' }
