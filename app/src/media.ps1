# Pembantu media (SMTC) untuk Gravi Island. Dijalankan HANYA saat tab Musik dibuka, dihentikan setelah ditutup.
# Keluar: satu baris JSON per detik. Masuk (stdin): {"cmd":"play_pause|next|prev"}.
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding($false)
function Emit($o) { [Console]::Out.WriteLine(($o | ConvertTo-Json -Compress)); [Console]::Out.Flush() }
try {
  Add-Type -AssemblyName System.Runtime.WindowsRuntime
  $asTask = ([System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object { $_.Name -eq 'AsTask' -and $_.GetParameters().Count -eq 1 -and $_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1' })[0]
  [void][Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager, Windows.Media.Control, ContentType = WindowsRuntime]
  function Await($op, [Type]$t) { $task = $asTask.MakeGenericMethod($t).Invoke($null, @($op)); [void]$task.Wait(-1); $task.Result }
  $mgr = Await ([Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager]::RequestAsync()) ([Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager])
} catch { Emit @{ a = $false; playing = $false; title = ''; artist = ''; src = ''; pos = 0; dur = 0 }; exit }
$rd = [Console]::In.ReadLineAsync()
while ($true) {
  if ($rd.IsCompleted) {
    $line = $rd.Result
    if ($null -eq $line) { exit }
    try {
      $c = $line | ConvertFrom-Json; $s = $mgr.GetCurrentSession()
      if ($s) { switch ($c.cmd) {
        'play_pause' { [void](Await ($s.TryTogglePlayPauseAsync()) ([bool])) }
        'next' { [void](Await ($s.TrySkipNextAsync()) ([bool])) }
        'prev' { [void](Await ($s.TrySkipPreviousAsync()) ([bool])) } } }
    } catch {}
    $rd = [Console]::In.ReadLineAsync()
  }
  try {
    $s = $mgr.GetCurrentSession()
    if ($null -eq $s) { Emit @{ a = $false; playing = $false; title = ''; artist = ''; src = ''; pos = 0; dur = 0 } }
    else {
      $p = Await ($s.TryGetMediaPropertiesAsync()) ([Windows.Media.Control.GlobalSystemMediaTransportControlsSessionMediaProperties])
      $tl = $s.GetTimelineProperties()
      Emit @{ a = $true; playing = ($s.GetPlaybackInfo().PlaybackStatus.ToString() -eq 'Playing'); title = [string]$p.Title; artist = [string]$p.Artist; src = [string]$s.SourceAppUserModelId; pos = [math]::Round($tl.Position.TotalSeconds, 1); dur = [math]::Round($tl.EndTime.TotalSeconds, 1) }
    }
  } catch {}
  Start-Sleep -Milliseconds 1000
}
