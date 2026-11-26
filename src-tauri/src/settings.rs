use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    // 1. Umum & Island
    pub island_position: String, // "top-center" | "top-left" | "top-right"
    pub auto_hide_idle: bool,
    pub clock_24h: bool,
    pub show_seconds: bool,
    pub start_on_boot: bool,
    pub island_opacity: u8, // 75..100
    pub demo_mode: bool,

    // 2. Karakter & Suara
    pub eye_tracking: bool,
    pub ear_animations: bool,
    pub squish_dizzy: bool,
    pub sound_enabled: bool,
    pub sound_volume: u8, // 0..100

    // 3. AI Monitor & Hooks
    pub monitored_ais: Vec<String>,
    pub max_compact_logos: u8, // 1..6
    pub poll_interval_secs: u8,

    // 4. GitHub
    pub github_default_range: u32, // 3 | 6 | 12
    pub github_default_items: u8,  // 3 | 5 | 10
    pub github_refresh_mins: u8,

    // 5. Google Calendar
    pub calendar_reminder_mins: u8,
    pub calendar_week_start: String, // "monday" | "sunday"

    // 6. Mini Chat & File Drop
    pub chat_default_provider: String,
    pub chat_default_model: String,
    pub ollama_endpoint: String,
    pub chat_system_prompt: String,
    pub max_drop_file_kb: u32,

    // 7. Media, Fokus & Pintasan
    pub pomo_focus_mins: u16,
    pub pomo_break_mins: u16,
    pub media_sync_enabled: bool,
    pub media_dance_enabled: bool,
    pub shortcut_toggle_island: String,
    pub shortcut_open_chat: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            island_position: "top-center".into(),
            auto_hide_idle: false,
            clock_24h: true,
            show_seconds: false,
            start_on_boot: false,
            island_opacity: 96,
            demo_mode: false,

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

            chat_default_provider: "ollama".into(),
            chat_default_model: "llama3.2".into(),
            ollama_endpoint: "http://127.0.0.1:11434".into(),
            chat_system_prompt: "Kamu adalah Nobi, asisten developer yang ringkas dan membantu.".into(),
            max_drop_file_kb: 256,

            pomo_focus_mins: 25,
            pomo_break_mins: 5,
            media_sync_enabled: true,
            media_dance_enabled: true,
            shortcut_toggle_island: "Alt+Space".into(),
            shortcut_open_chat: "Alt+C".into(),
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
    let path = settings_path(&app)?;
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    let _ = app.emit("nobi:settings-updated", &settings);
    Ok(settings)
}

#[tauri::command]
pub fn reset_settings(app: AppHandle) -> Result<AppSettings, String> {
    let defaults = AppSettings::default();
    save_settings(app, defaults)
}
