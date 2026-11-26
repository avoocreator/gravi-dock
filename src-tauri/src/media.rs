use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaState {
    pub title: String,
    pub artist: String,
    pub is_playing: bool,
    pub position_secs: u32,
    pub duration_secs: u32,
    pub player: String,
}

#[tauri::command]
pub fn get_media_state() -> Result<MediaState, String> {
    #[cfg(target_os = "linux")]
    {
        let status = std::process::Command::new("playerctl")
            .args(["status"])
            .output()
            .map_err(|_| "playerctl belum terpasang (sudo apt install playerctl)".to_string())?;
        let is_playing = String::from_utf8_lossy(&status.stdout)
            .trim()
            .eq_ignore_ascii_case("Playing");

        let meta = std::process::Command::new("playerctl")
            .args(["metadata", "--format", "{{title}}|||{{artist}}|||{{playerName}}"])
            .output()
            .map_err(|e| e.to_string())?;
        let raw = String::from_utf8_lossy(&meta.stdout).trim().to_string();
        let parts: Vec<&str> = raw.split("|||").collect();
        return Ok(MediaState {
            title: parts.first().copied().unwrap_or("Tidak ada media").to_string(),
            artist: parts.get(1).copied().unwrap_or("").to_string(),
            is_playing,
            position_secs: 0,
            duration_secs: 200,
            player: parts.get(2).copied().unwrap_or("MPRIS").to_string(),
        });
    }

    #[cfg(target_os = "windows")]
    {
        // Lightweight PowerShell GSMTC query; falls back cleanly if no media session is active
        let script = r#"
Add-Type -AssemblyName System.Runtime.WindowsRuntime
$mgr = [Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager,Windows.Media.Control,ContentType=WindowsRuntime]::RequestAsync().GetAwaiter().GetResult()
$s = $mgr.GetCurrentSession()
if ($null -eq $s) { Write-Output "NONE"; exit }
$p = $s.TryGetMediaPropertiesAsync().GetAwaiter().GetResult()
$pb = $s.GetPlaybackInfo().PlaybackStatus
Write-Output "$($p.Title)|||$($p.Artist)|||$pb"
"#;
        if let Ok(out) = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if text != "NONE" && text.contains("|||") {
                let parts: Vec<&str> = text.split("|||").collect();
                return Ok(MediaState {
                    title: parts.first().copied().unwrap_or("").to_string(),
                    artist: parts.get(1).copied().unwrap_or("").to_string(),
                    is_playing: parts.get(2).copied().unwrap_or("") == "Playing",
                    position_secs: 0,
                    duration_secs: 210,
                    player: "Windows Media".to_string(),
                });
            }
        }
        return Err("Tidak ada sesi media aktif di Windows.".into());
    }

    #[allow(unreachable_code)]
    Err("Kontrol media OS tidak tersedia.".into())
}

#[tauri::command]
pub fn media_control(action: String) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let cmd = match action.as_str() {
            "play_pause" => "play-pause",
            "next" => "next",
            "previous" => "previous",
            _ => return Err("Aksi media tidak dikenal".into()),
        };
        std::process::Command::new("playerctl")
            .arg(cmd)
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        let vk = match action.as_str() {
            "play_pause" => "0xB3",
            "next" => "0xB0",
            "previous" => "0xB1",
            _ => return Err("Aksi media tidak dikenal".into()),
        };
        let ps = format!(
            "$s = New-Object -ComObject WScript.Shell; Add-Type -TypeDefinition 'using System;using System.Runtime.InteropServices;public class K{{[DllImport(\"user32.dll\")]public static extern void keybd_event(byte bVk,byte bScan,uint dwFlags,UIntPtr dwExtraInfo);}}'; [K]::keybd_event({},0,0,[UIntPtr]::Zero); [K]::keybd_event({},0,2,[UIntPtr]::Zero)",
            vk, vk
        );
        std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps])
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Ok(())
}
