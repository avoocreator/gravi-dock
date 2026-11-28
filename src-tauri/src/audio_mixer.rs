use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sysinfo::{Pid, ProcessesToUpdate, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioAppSession {
    pub app_key: String,
    pub display_name: String,
    pub volume_percent: u8,
    pub muted: bool,
}

fn prettify_app_name(raw: &str) -> String {
    let base = raw.trim_end_matches(".exe").trim_end_matches(".EXE");
    let low = base.to_lowercase();
    match low.as_str() {
        "chrome" => "Google Chrome".into(),
        "msedge" => "Microsoft Edge".into(),
        "firefox" => "Firefox".into(),
        "brave" => "Brave".into(),
        "opera" => "Opera".into(),
        "spotify" => "Spotify".into(),
        "discord" => "Discord".into(),
        "whatsapp" => "WhatsApp".into(),
        "telegram" => "Telegram".into(),
        "vlc" => "VLC Media Player".into(),
        "code" => "VS Code".into(),
        "obs64" | "obs" => "OBS Studio".into(),
        "zoom" => "Zoom".into(),
        "ms-teams" | "teams" => "Microsoft Teams".into(),
        "system" => "System Sounds".into(),
        _ => {
            let mut chars = base.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => "Aplikasi".into(),
            }
        }
    }
}

#[cfg(target_os = "windows")]
mod win_mixer {
    use super::{prettify_app_name, AudioAppSession};
    use std::collections::HashMap;
    use sysinfo::{Pid, ProcessesToUpdate, System};
    use windows::core::Interface;
    use windows::Win32::Media::Audio::{
        eConsole, eRender, IAudioSessionControl2, IAudioSessionManager2, IMMDeviceEnumerator,
        ISimpleAudioVolume, MMDeviceEnumerator,
    };
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED,
    };

    fn with_sessions<F>(mut f: F) -> Result<(), String>
    where
        F: FnMut(u32, &str, &ISimpleAudioVolume),
    {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                    .map_err(|e| format!("WASAPI init gagal: {}", e))?;
            let device = enumerator
                .GetDefaultAudioEndpoint(eRender, eConsole)
                .map_err(|e| format!("Perangkat audio default tidak ditemukan: {}", e))?;
            let manager: IAudioSessionManager2 = device
                .Activate(CLSCTX_ALL, None)
                .map_err(|e| format!("Gagal membuka AudioSessionManager: {}", e))?;
            let session_enum = manager
                .GetSessionEnumerator()
                .map_err(|e| format!("Gagal membaca sesi audio: {}", e))?;
            let count = session_enum.GetCount().unwrap_or(0);

            let mut sys = System::new();
            sys.refresh_processes(ProcessesToUpdate::All, false);

            for i in 0..count {
                let Ok(ctrl) = session_enum.GetSession(i) else {
                    continue;
                };
                let Ok(ctrl2) = ctrl.cast::<IAudioSessionControl2>() else {
                    continue;
                };
                let Ok(simple_vol) = ctrl.cast::<ISimpleAudioVolume>() else {
                    continue;
                };
                let pid = ctrl2.GetProcessId().unwrap_or(0);
                let proc_name = if pid == 0 {
                    "system".to_string()
                } else {
                    sys.process(Pid::from_u32(pid))
                        .map(|p| p.name().to_string_lossy().to_string())
                        .unwrap_or_else(|| format!("pid-{}", pid))
                };
                f(pid, &proc_name, &simple_vol);
            }
        }
        Ok(())
    }

    pub fn list_sessions() -> Result<Vec<AudioAppSession>, String> {
        let mut grouped: HashMap<String, AudioAppSession> = HashMap::new();
        with_sessions(|_pid, proc_name, simple_vol| {
            let key = proc_name.trim_end_matches(".exe").to_lowercase();
            if key == "gravi" {
                return;
            }
            let vol = unsafe { simple_vol.GetMasterVolume().unwrap_or(1.0) };
            let muted = unsafe { simple_vol.GetMute().map(|b| b.as_bool()).unwrap_or(false) };
            let pct = (vol.clamp(0.0, 1.0) * 100.0).round() as u8;
            grouped.entry(key.clone()).or_insert_with(|| AudioAppSession {
                app_key: key,
                display_name: prettify_app_name(proc_name),
                volume_percent: pct,
                muted,
            });
        })?;

        let mut list: Vec<AudioAppSession> = grouped.into_values().collect();
        list.sort_by(|a, b| {
            let a_sys = a.app_key == "system";
            let b_sys = b.app_key == "system";
            a_sys.cmp(&b_sys).then(a.display_name.cmp(&b.display_name))
        });
        Ok(list)
    }

    pub fn set_volume(app_key: &str, volume_percent: u8) -> Result<(), String> {
        let target = app_key.to_lowercase();
        let vol_f32 = (volume_percent.min(100) as f32) / 100.0;
        with_sessions(|_pid, proc_name, simple_vol| {
            let key = proc_name.trim_end_matches(".exe").to_lowercase();
            if key == target {
                unsafe {
                    let _ = simple_vol.SetMasterVolume(vol_f32, std::ptr::null());
                }
            }
        })
    }

    pub fn set_mute(app_key: &str, muted: bool) -> Result<(), String> {
        let target = app_key.to_lowercase();
        with_sessions(|_pid, proc_name, simple_vol| {
            let key = proc_name.trim_end_matches(".exe").to_lowercase();
            if key == target {
                unsafe {
                    let _ = simple_vol.SetMute(muted, std::ptr::null());
                }
            }
        })
    }
}

#[tauri::command]
pub async fn list_audio_sessions() -> Result<Vec<AudioAppSession>, String> {
    tokio::task::spawn_blocking(|| {
        #[cfg(target_os = "windows")]
        {
            return win_mixer::list_sessions();
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = (HashMap::<String, u8>::new(), Pid::from_u32(0), ProcessesToUpdate::All, System::new(), prettify_app_name(""));
            Ok(Vec::new())
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn set_app_audio_volume(app_key: String, volume_percent: u8) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        #[cfg(target_os = "windows")]
        {
            return win_mixer::set_volume(&app_key, volume_percent);
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = (app_key, volume_percent);
            Ok(())
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn set_app_audio_mute(app_key: String, muted: bool) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        #[cfg(target_os = "windows")]
        {
            return win_mixer::set_mute(&app_key, muted);
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = (app_key, muted);
            Ok(())
        }
    })
    .await
    .map_err(|e| e.to_string())?
}
