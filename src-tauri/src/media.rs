use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaState {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub is_playing: bool,
    pub position_secs: u32,
    pub duration_secs: u32,
    pub player: String,
    pub artwork_url: Option<String>,
    pub has_media: bool,
}

impl Default for MediaState {
    fn default() -> Self {
        Self {
            title: "Tidak ada media aktif".to_string(),
            artist: "Putar musik di Spotify, YouTube, atau pemutar lain".to_string(),
            album: String::new(),
            is_playing: false,
            position_secs: 0,
            duration_secs: 0,
            player: String::new(),
            artwork_url: None,
            has_media: false,
        }
    }
}

#[cfg(target_os = "windows")]
fn encode_base64(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(((input.len() + 2) / 3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((triple >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

#[cfg(target_os = "windows")]
fn friendly_player_name(app_id: &str) -> String {
    let low = app_id.to_lowercase();
    if low.contains("spotify") {
        "Spotify".into()
    } else if low.contains("chrome") {
        "Chrome".into()
    } else if low.contains("msedge") || low.contains("edge") {
        "Edge".into()
    } else if low.contains("firefox") {
        "Firefox".into()
    } else if low.contains("brave") {
        "Brave".into()
    } else if low.contains("applemusic") || low.contains("itunes") {
        "Apple Music".into()
    } else if low.contains("vlc") {
        "VLC".into()
    } else if low.contains("foobar") {
        "foobar2000".into()
    } else if low.contains("musicbee") {
        "MusicBee".into()
    } else if low.contains("ytmusic") || low.contains("youtube") {
        "YouTube Music".into()
    } else if app_id.trim().is_empty() {
        "Windows Media".into()
    } else {
        app_id
            .split('!')
            .next()
            .unwrap_or(app_id)
            .split('.')
            .last()
            .unwrap_or(app_id)
            .trim_end_matches(".exe")
            .to_string()
    }
}

#[cfg(target_os = "windows")]
mod win_media {
    use super::{encode_base64, friendly_player_name, MediaState};
    use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    };
    use windows::Storage::Streams::DataReader;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        keybd_event, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_MEDIA_NEXT_TRACK,
        VK_MEDIA_PLAY_PAUSE, VK_MEDIA_PREV_TRACK, VK_VOLUME_DOWN, VK_VOLUME_MUTE, VK_VOLUME_UP,
    };

    fn current_session() -> Option<GlobalSystemMediaTransportControlsSession> {
        let mgr = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
            .ok()?
            .get()
            .ok()?;
        if let Ok(session) = mgr.GetCurrentSession() {
            return Some(session);
        }
        let sessions = mgr.GetSessions().ok()?;
        if sessions.Size().ok()? > 0 {
            sessions.GetAt(0).ok()
        } else {
            None
        }
    }

    fn send_vk(vk: VIRTUAL_KEY) {
        unsafe {
            keybd_event(vk.0 as u8, 0, KEYBD_EVENT_FLAGS(0), 0);
            keybd_event(vk.0 as u8, 0, KEYEVENTF_KEYUP, 0);
        }
    }

    pub fn query_state() -> MediaState {
        let Some(session) = current_session() else {
            return MediaState::default();
        };

        let is_playing = session
            .GetPlaybackInfo()
            .and_then(|info| info.PlaybackStatus())
            .map(|st| st == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing)
            .unwrap_or(false);

        let (position_secs, duration_secs) = session
            .GetTimelineProperties()
            .map(|tl| {
                let pos = tl.Position().map(|d| (d.Duration / 10_000_000).max(0) as u32).unwrap_or(0);
                let dur = tl.EndTime().map(|d| (d.Duration / 10_000_000).max(0) as u32).unwrap_or(0);
                (pos, dur)
            })
            .unwrap_or((0, 0));

        let app_id = session
            .SourceAppUserModelId()
            .map(|s| s.to_string())
            .unwrap_or_default();

        let props = session
            .TryGetMediaPropertiesAsync()
            .ok()
            .and_then(|op| op.get().ok());

        let (title, artist, album, artwork_url) = if let Some(p) = props {
            let title = p.Title().map(|s| s.to_string()).unwrap_or_default();
            let artist = p.Artist().map(|s| s.to_string()).unwrap_or_default();
            let album = p.AlbumTitle().map(|s| s.to_string()).unwrap_or_default();

            let artwork = p.Thumbnail().ok().and_then(|thumb| {
                let stream = thumb.OpenReadAsync().ok()?.get().ok()?;
                let size = stream.Size().ok()?;
                if size == 0 || size > 600_000 {
                    return None;
                }
                let content_type = stream
                    .ContentType()
                    .map(|s| s.to_string())
                    .ok()
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| "image/png".to_string());
                let input = stream.GetInputStreamAt(0).ok()?;
                let reader = DataReader::CreateDataReader(&input).ok()?;
                reader.LoadAsync(size as u32).ok()?.get().ok()?;
                let mut buf = vec![0u8; size as usize];
                reader.ReadBytes(&mut buf).ok()?;
                Some(format!("data:{};base64,{}", content_type, encode_base64(&buf)))
            });

            (title, artist, album, artwork)
        } else {
            (String::new(), String::new(), String::new(), None)
        };

        if title.trim().is_empty() && !is_playing {
            return MediaState::default();
        }

        MediaState {
            title: if title.trim().is_empty() {
                "Media Aktif".into()
            } else {
                title
            },
            artist,
            album,
            is_playing,
            position_secs,
            duration_secs,
            player: friendly_player_name(&app_id),
            artwork_url,
            has_media: true,
        }
    }

    pub fn control(action: &str) -> Result<(), String> {
        match action {
            "volume_up" => {
                send_vk(VK_VOLUME_UP);
                return Ok(());
            }
            "volume_down" => {
                send_vk(VK_VOLUME_DOWN);
                return Ok(());
            }
            "mute" => {
                send_vk(VK_VOLUME_MUTE);
                return Ok(());
            }
            _ => {}
        }

        if let Some(session) = current_session() {
            let handled = match action {
                "play_pause" => session
                    .TryTogglePlayPauseAsync()
                    .ok()
                    .and_then(|op| op.get().ok())
                    .unwrap_or(false),
                "next" => session
                    .TrySkipNextAsync()
                    .ok()
                    .and_then(|op| op.get().ok())
                    .unwrap_or(false),
                "previous" => session
                    .TrySkipPreviousAsync()
                    .ok()
                    .and_then(|op| op.get().ok())
                    .unwrap_or(false),
                _ => return Err("Aksi media tidak dikenal".into()),
            };
            if handled {
                return Ok(());
            }
        }

        // Win32 fallback
        match action {
            "play_pause" => send_vk(VK_MEDIA_PLAY_PAUSE),
            "next" => send_vk(VK_MEDIA_NEXT_TRACK),
            "previous" => send_vk(VK_MEDIA_PREV_TRACK),
            _ => return Err("Aksi media tidak dikenal".into()),
        }
        Ok(())
    }

    pub fn seek(position_secs: u32) -> Result<(), String> {
        if let Some(session) = current_session() {
            let ticks: i64 = (position_secs as i64) * 10_000_000;
            let _ = session
                .TryChangePlaybackPositionAsync(ticks)
                .ok()
                .and_then(|op| op.get().ok());
        }
        Ok(())
    }
}

#[tauri::command]
pub async fn get_media_state() -> Result<MediaState, String> {
    tokio::task::spawn_blocking(get_media_state_sync)
        .await
        .map_err(|e| e.to_string())?
}

fn get_media_state_sync() -> Result<MediaState, String> {
    #[cfg(target_os = "windows")]
    {
        return Ok(win_media::query_state());
    }

    #[cfg(target_os = "linux")]
    {
        let status = std::process::Command::new("playerctl")
            .args(["status"])
            .output()
            .map_err(|_| "playerctl belum terpasang (sudo apt install playerctl)".to_string())?;
        let status_str = String::from_utf8_lossy(&status.stdout).trim().to_string();
        if status_str.is_empty() || status_str.eq_ignore_ascii_case("Stopped") {
            return Ok(MediaState::default());
        }
        let is_playing = status_str.eq_ignore_ascii_case("Playing");

        let meta = std::process::Command::new("playerctl")
            .args([
                "metadata",
                "--format",
                "{{title}}|||{{artist}}|||{{album}}|||{{playerName}}|||{{mpris:length}}|||{{mpris:artUrl}}",
            ])
            .output()
            .map_err(|e| e.to_string())?;
        let raw = String::from_utf8_lossy(&meta.stdout).trim().to_string();
        if raw.is_empty() {
            return Ok(MediaState::default());
        }
        let parts: Vec<&str> = raw.split("|||").collect();
        let title = parts.first().copied().unwrap_or("").trim().to_string();
        let artist = parts.get(1).copied().unwrap_or("").trim().to_string();
        let album = parts.get(2).copied().unwrap_or("").trim().to_string();
        let player = parts.get(3).copied().unwrap_or("MPRIS").trim().to_string();
        let len_us: u64 = parts
            .get(4)
            .copied()
            .unwrap_or("0")
            .trim()
            .parse()
            .unwrap_or(0);
        let duration_secs = (len_us / 1_000_000) as u32;
        let art_raw = parts.get(5).copied().unwrap_or("").trim();
        let artwork_url = if art_raw.starts_with("http://") || art_raw.starts_with("https://") {
            Some(art_raw.to_string())
        } else {
            None
        };

        let pos_out = std::process::Command::new("playerctl")
            .args(["position"])
            .output()
            .ok();
        let position_secs = pos_out
            .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<f64>().ok())
            .map(|v| v.max(0.0) as u32)
            .unwrap_or(0);

        return Ok(MediaState {
            title: if title.is_empty() { "Media Aktif".into() } else { title },
            artist,
            album,
            is_playing,
            position_secs,
            duration_secs,
            player,
            artwork_url,
            has_media: true,
        });
    }

    #[cfg(target_os = "macos")]
    {
        let script = r#"
if application "Spotify" is running then
  tell application "Spotify"
    if player state is playing or player state is paused then
      return (name of current track) & "|||" & (artist of current track) & "|||" & (album of current track) & "|||" & (player state as string) & "|||" & (round (player position)) & "|||" & (round ((duration of current track) / 1000)) & "|||Spotify|||" & (artwork url of current track)
    end if
  end tell
end if
return "NONE"
"#;
        if let Ok(out) = std::process::Command::new("osascript")
            .args(["-e", script])
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if text != "NONE" && text.contains("|||") {
                let parts: Vec<&str> = text.split("|||").collect();
                let art = parts.get(7).copied().unwrap_or("").trim();
                return Ok(MediaState {
                    title: parts.first().copied().unwrap_or("").to_string(),
                    artist: parts.get(1).copied().unwrap_or("").to_string(),
                    album: parts.get(2).copied().unwrap_or("").to_string(),
                    is_playing: parts.get(3).copied().unwrap_or("") == "playing",
                    position_secs: parts.get(4).copied().unwrap_or("0").parse().unwrap_or(0),
                    duration_secs: parts.get(5).copied().unwrap_or("0").parse().unwrap_or(0),
                    player: parts.get(6).copied().unwrap_or("Spotify").to_string(),
                    artwork_url: if art.starts_with("http") { Some(art.to_string()) } else { None },
                    has_media: true,
                });
            }
        }
        return Ok(MediaState::default());
    }

    #[allow(unreachable_code)]
    Ok(MediaState::default())
}

#[tauri::command]
pub async fn media_control(action: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || media_control_sync(&action))
        .await
        .map_err(|e| e.to_string())?
}

fn media_control_sync(action: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        return win_media::control(action);
    }

    #[cfg(target_os = "linux")]
    {
        match action {
            "play_pause" => {
                std::process::Command::new("playerctl")
                    .arg("play-pause")
                    .spawn()
                    .map_err(|e| e.to_string())?;
            }
            "next" => {
                std::process::Command::new("playerctl")
                    .arg("next")
                    .spawn()
                    .map_err(|e| e.to_string())?;
            }
            "previous" => {
                std::process::Command::new("playerctl")
                    .arg("previous")
                    .spawn()
                    .map_err(|e| e.to_string())?;
            }
            "volume_up" => {
                let _ = std::process::Command::new("playerctl").args(["volume", "0.1+"]).spawn();
            }
            "volume_down" => {
                let _ = std::process::Command::new("playerctl").args(["volume", "0.1-"]).spawn();
            }
            "mute" => {
                let _ = std::process::Command::new("pactl")
                    .args(["set-sink-mute", "@DEFAULT_SINK@", "toggle"])
                    .spawn();
            }
            _ => return Err("Aksi media tidak dikenal".into()),
        }
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        let cmd = match action {
            "play_pause" => "tell application \"Spotify\" to playpause",
            "next" => "tell application \"Spotify\" to next track",
            "previous" => "tell application \"Spotify\" to previous track",
            "volume_up" => "set volume output volume ((output volume of (get volume settings)) + 8)",
            "volume_down" => "set volume output volume ((output volume of (get volume settings)) - 8)",
            "mute" => "set volume output muted not (output muted of (get volume settings))",
            _ => return Err("Aksi media tidak dikenal".into()),
        };
        std::process::Command::new("osascript")
            .args(["-e", cmd])
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Ok(())
}

#[tauri::command]
pub async fn media_seek(position_secs: u32) -> Result<(), String> {
    tokio::task::spawn_blocking(move || media_seek_sync(position_secs))
        .await
        .map_err(|e| e.to_string())?
}

fn media_seek_sync(position_secs: u32) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        return win_media::seek(position_secs);
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("playerctl")
            .args(["position", &position_secs.to_string()])
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        let cmd = format!("tell application \"Spotify\" to set player position to {}", position_secs);
        std::process::Command::new("osascript")
            .args(["-e", &cmd])
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Ok(())
}
