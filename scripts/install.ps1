# Pasang Gravi Island: salin biner release ke %LOCALAPPDATA%\GraviIsland, daftarkan PATH, buat shortcut Start Menu, (opsional) autostart.
param([switch]$Autostart, [switch]$Uninstall)
$ErrorActionPreference = 'Stop'
$dir = Join-Path $env:LOCALAPPDATA 'GraviIsland'
$exe = Join-Path $dir 'gravi-island.exe'
$hook = Join-Path $dir 'gravi-hook.exe'
$start = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs\Gravi Island.lnk'
$boot = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs\Startup\Gravi Island.lnk'

if ($Uninstall) {
  Get-Process gravi-island -ErrorAction SilentlyContinue | Stop-Process -Force
  Remove-Item $start, $boot -ErrorAction SilentlyContinue
  $p = [Environment]::GetEnvironmentVariable('Path', 'User') -split ';' | Where-Object { $_ -and $_ -ne $dir }
  [Environment]::SetEnvironmentVariable('Path', ($p -join ';'), 'User')
  Remove-Item $exe, $hook -ErrorAction SilentlyContinue
  Write-Host 'Gravi Island berhasil dihapus (settings.json dibiarkan aman).'
  exit
}

$srcExe = @((Join-Path $PSScriptRoot '..\target\release\gravi-island.exe'), (Join-Path $PSScriptRoot 'gravi-island.exe')) | Where-Object { Test-Path $_ } | Select-Object -First 1
$srcHook = @((Join-Path $PSScriptRoot '..\target\release\gravi-hook.exe'), (Join-Path $PSScriptRoot 'gravi-hook.exe')) | Where-Object { Test-Path $_ } | Select-Object -First 1

if (-not $srcExe) {
  throw 'gravi-island.exe tidak ditemukan di target\release. Jalankan dulu: cargo build --release'
}

New-Item -ItemType Directory -Force $dir | Out-Null
Get-Process gravi-island -ErrorAction SilentlyContinue | Stop-Process -Force
Copy-Item $srcExe $exe -Force
if ($srcHook) { Copy-Item $srcHook $hook -Force }

$p = [Environment]::GetEnvironmentVariable('Path', 'User')
if (($p -split ';') -notcontains $dir) {
  [Environment]::SetEnvironmentVariable('Path', ($p.TrimEnd(';') + ';' + $dir), 'User')
}

$sh = New-Object -ComObject WScript.Shell
function Mk($path) {
  $l = $sh.CreateShortcut($path)
  $l.TargetPath = $exe
  $l.WorkingDirectory = $dir
  $l.Save()
}

Mk $start
if ($Autostart) { Mk $boot }

Start-Process $exe
Write-Host "Sukses terpasang di: $dir"
Write-Host "Sekarang bisa dijalankan lewat Start Menu atau Win+R (ketik: gravi-island)"
Write-Host "Hotkey toggle: Ctrl+Alt+A"
if ($Autostart) { Write-Host 'Autostart saat Windows menyala: Aktif' }
