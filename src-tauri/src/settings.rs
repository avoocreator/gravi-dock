use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    // Umum
    pub island_position: String,
    pub island_style: String,
    pub open_on_hover: bool,
    pub hover_close_delay_ms: u32,
    pub auto_hide_idle: bool,
    pub clock_24h: bool,
    pub show_seconds: bool,
    pub start_on_boot: bool,
    pub island_opacity: u8,

    // Karakter
    pub eye_tracking: bool,
    pub ear_animations: bool,
    pub squish_dizzy: bool,
    pub sound_enabled: bool,
    pub sound_volume: u8,

    // AI Monitor
    pub monitored_ais: Vec<String>,
    pub max_compact_logos: u8,
    pub poll_interval_secs: u8,

    // GitHub
    pub github_default_range: u32,
    pub github_default_items: u8,
    pub github_refresh_mins: u8,

    // Calendar
    pub calendar_reminder_mins: u8,
    pub calendar_week_start: String,
    pub calendar_ai_provider: String,
    pub calendar_ai_model: String,

    // Chat
    pub chat_default_provider: String,
    pub chat_default_model: String,
    pub ollama_endpoint: String,
    pub chat_system_prompt: String,
    pub max_drop_file_kb: u32,

    // Utilities
    pub pomo_focus_mins: u16,
    pub pomo_break_mins: u16,
    pub media_sync_enabled: bool,
    pub media_dance_enabled: bool,
    pub shortcut_toggle_island: String,
    pub shortcut_open_chat: String,

    // HUD
    pub hud_enabled: bool,
    pub hud_capslock: bool,
    pub hud_battery_charging: bool,
    pub hud_battery_low: bool,
    pub hud_wifi: bool,
    pub hud_ai_done: bool,
    pub hud_windows_notif: bool,
    pub notif_messaging_only: bool,
    pub hud_duration_ms: u32,
    pub hud_bt_low: bool,
    pub focus_habit_deadline: String,
    pub focus_habit_warn_mins: u8,
    pub ambient_auto_focus: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            island_position: "top-center".into(),
            island_style: "floating".into(),
            open_on_hover: true,
            hover_close_delay_ms: 450,
            auto_hide_idle: false,
            clock_24h: true,
            show_seconds: false,
            start_on_boot: false,
            island_opacity: 96,

            eye_tracking: true,
            ear_animations: true,
            squish_dizzy: true,
            sound_enabled: true,
            sound_volume: 70,

            monitored_ais: vec![
                "claude-desktop".into(),
                "gemini-desktop".into(),
                "chatgpt-desktop".into(),
                "antigravity-desktop".into(),
                "hermes-desktop".into(),
                "openbot-desktop".into(),
                "antigravity-cli".into(),
                "copilot-cli".into(),
            ],
            max_compact_logos: 4,
            poll_interval_secs: 3,

            github_default_range: 3,
            github_default_items: 3,
            github_refresh_mins: 10,

            calendar_reminder_mins: 15,
            calendar_week_start: "monday".into(),
            calendar_ai_provider: "ollama".into(),
            calendar_ai_model: "llama3.2".into(),

            chat_default_provider: "ollama".into(),
            chat_default_model: "llama3.2".into(),
            ollama_endpoint: "http://127.0.0.1:11434".into(),
            chat_system_prompt: "Kamu adalah Gravi, asisten developer yang ringkas dan membantu.".into(),
            max_drop_file_kb: 256,

            pomo_focus_mins: 25,
            pomo_break_mins: 5,
            media_sync_enabled: true,
            media_dance_enabled: true,
            shortcut_toggle_island: "Alt+Space".into(),
            shortcut_open_chat: "Alt+C".into(),

            hud_enabled: true,
            hud_capslock: true,
            hud_battery_charging: true,
            hud_battery_low: true,
            hud_wifi: true,
            hud_ai_done: true,
            hud_windows_notif: true,
            notif_messaging_only: false,
            hud_duration_ms: 3500,
            hud_bt_low: true,
            focus_habit_deadline: "21:00".into(),
            focus_habit_warn_mins: 30,
            ambient_auto_focus: false,
        }
    }
}

fn apply_autostart(enabled: bool) {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let exe_str = exe.to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let run_key = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
        if enabled {
            let quoted = format!("\"{}\"", exe_str);
            let _ = std::process::Command::new("reg")
                .args(["add", run_key, "/v", "Gravi", "/t", "REG_SZ", "/d", &quoted, "/f"])
                .creation_flags(CREATE_NO_WINDOW)
                .output();
        } else {
            let _ = std::process::Command::new("reg")
                .args(["delete", run_key, "/v", "Gravi", "/f"])
                .creation_flags(CREATE_NO_WINDOW)
                .output();
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
            let dir = home.join(".config").join("autostart");
            let file = dir.join("gravi.desktop");
            if enabled {
                if fs::create_dir_all(&dir).is_ok() {
                    let desktop = format!(
                        "[Desktop Entry]\nType=Application\nName=Gravi\nExec=\"{}\"\nX-GNOME-Autostart-enabled=true\n",
                        exe_str
                    );
                    let _ = fs::write(file, desktop);
                }
            } else {
                let _ = fs::remove_file(file);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
            let dir = home.join("Library").join("LaunchAgents");
            let file = dir.join("app.gravi.plist");
            if enabled {
                if fs::create_dir_all(&dir).is_ok() {
                    let plist = format!(
                        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>Label</key><string>app.gravi</string>
<key>ProgramArguments</key><array><string>{}</string></array>
<key>RunAtLoad</key><true/>
</dict></plist>
"#,
                        exe_str
                    );
                    let _ = fs::write(file, plist);
                }
            } else {
                let _ = fs::remove_file(file);
            }
        }
    }
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("settings.json"))
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> AppSettings {
    let Ok(path) = settings_path(&app) else {
        return AppSettings::default();
    };
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str::<AppSettings>(&s).ok())
        .unwrap_or_default()
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: AppSettings) -> Result<AppSettings, String> {
    let prev = get_settings(app.clone());
    let path = settings_path(&app)?;
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;

    if prev.start_on_boot != settings.start_on_boot {
        apply_autostart(settings.start_on_boot);
    }

    if let Some(w) = app.get_webview_window("main") {
        crate::position_island(&w, &settings.island_position, &settings.island_style);
    }
    let _ = app.emit("gravi:settings-updated", &settings);
    Ok(settings)
}

#[tauri::command]
pub fn reset_settings(app: AppHandle) -> Result<AppSettings, String> {
    let defaults = AppSettings::default();
    save_settings(app, defaults)
}
