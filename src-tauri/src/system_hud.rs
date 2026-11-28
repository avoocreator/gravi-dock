use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HudEventPayload {
    pub kind: String,
    pub title: String,
    pub subtitle: String,
    pub mood: String,
    pub accent: String,
    pub badge: Option<String>,
    pub provider: Option<String>,
    pub app_name: Option<String>,
}

fn read_capslock() -> Option<bool> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_CAPITAL};
        let state = unsafe { GetKeyState(VK_CAPITAL.0 as i32) };
        return Some((state & 1) != 0);
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = std::fs::read_dir("/sys/class/leds") {
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().to_lowercase();
                if name.contains("capslock") {
                    if let Ok(val) = std::fs::read_to_string(e.path().join("brightness")) {
                        return Some(val.trim() == "1");
                    }
                }
            }
        }
        return None;
    }

    #[allow(unreachable_code)]
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BatterySnapshot {
    charging: bool,
    percent: u8,
}

fn read_battery() -> Option<BatterySnapshot> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
        let mut status = SYSTEM_POWER_STATUS::default();
        if unsafe { GetSystemPowerStatus(&mut status) }.is_ok() {
            if status.BatteryLifePercent <= 100 && status.ACLineStatus != 255 {
                return Some(BatterySnapshot {
                    charging: status.ACLineStatus == 1,
                    percent: status.BatteryLifePercent,
                });
            }
        }
        return None;
    }

    #[cfg(target_os = "linux")]
    {
        for bat in ["BAT0", "BAT1"] {
            let base = std::path::Path::new("/sys/class/power_supply").join(bat);
            if let (Ok(cap), Ok(st)) = (
                std::fs::read_to_string(base.join("capacity")),
                std::fs::read_to_string(base.join("status")),
            ) {
                if let Ok(pct) = cap.trim().parse::<u8>() {
                    let charging = st.trim().eq_ignore_ascii_case("Charging")
                        || st.trim().eq_ignore_ascii_case("Full");
                    return Some(BatterySnapshot {
                        charging,
                        percent: pct.min(100),
                    });
                }
            }
        }
        return None;
    }

    #[allow(unreachable_code)]
    None
}

fn read_wifi_ssid() -> Option<Option<String>> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let out = std::process::Command::new("netsh")
            .args(["wlan", "show", "interfaces"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&out.stdout);
        let mut connected = false;
        let mut ssid: Option<String> = None;
        for line in text.lines() {
            let trimmed = line.trim();
            if let Some((k, v)) = trimmed.split_once(':') {
                let key = k.trim().to_lowercase();
                let val = v.trim();
                if key == "state" {
                    connected = val.eq_ignore_ascii_case("connected");
                } else if key == "ssid" && !val.is_empty() {
                    ssid = Some(val.to_string());
                }
            }
        }
        return Some(if connected { ssid } else { None });
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(out) = std::process::Command::new("iwgetid").arg("-r").output() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            return Some(if s.is_empty() { None } else { Some(s) });
        }
        return None;
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(out) = std::process::Command::new("networksetup")
            .args(["-getairportnetwork", "en0"])
            .output()
        {
            let s = String::from_utf8_lossy(&out.stdout);
            if let Some((_, name)) = s.split_once(':') {
                let ssid = name.trim();
                if !ssid.is_empty() && !ssid.contains("not associated") {
                    return Some(Some(ssid.to_string()));
                }
            }
            return Some(None);
        }
        return None;
    }

    #[allow(unreachable_code)]
    None
}

pub async fn start_hud_watcher(app: AppHandle) {
    let mut last_caps = read_capslock();
    let mut last_bat = read_battery();
    let mut low_bat_warned = last_bat.map(|b| b.percent <= 20 && !b.charging).unwrap_or(false);
    let mut full_bat_warned = last_bat.map(|b| b.percent >= 100 && b.charging).unwrap_or(false);
    let mut last_wifi = tokio::task::spawn_blocking(read_wifi_ssid)
        .await
        .ok()
        .flatten();
    let mut tick: u32 = 0;

    loop {
        sleep(Duration::from_millis(200)).await;
        tick = tick.wrapping_add(1);

        let cfg = crate::settings::get_settings(app.clone());
        if !cfg.hud_enabled {
            continue;
        }

        // CapsLock
        if cfg.hud_capslock {
            let cur_caps = read_capslock();
            if let (Some(prev), Some(curr)) = (last_caps, cur_caps) {
                if prev != curr {
                    let _ = app.emit(
                        "gravi:hud-event",
                        HudEventPayload {
                            kind: "capslock".into(),
                            title: "Caps Lock".into(),
                            subtitle: if curr {
                                "Huruf besar aktif".into()
                            } else {
                                "Huruf besar nonaktif".into()
                            },
                            mood: if curr { "capslock".into() } else { "idle".into() },
                            accent: if curr { "sky".into() } else { "violet".into() },
                            badge: Some(if curr { "ON".into() } else { "OFF".into() }),
                            provider: None,
                            app_name: None,
                        },
                    );
                }
            }
            if cur_caps.is_some() {
                last_caps = cur_caps;
            }
        }

        // Battery
        if tick % 6 == 0 {
            if let Some(curr) = read_battery() {
                if let Some(prev) = last_bat {
                    if !prev.charging && curr.charging && cfg.hud_battery_charging {
                        low_bat_warned = false;
                        let _ = app.emit(
                            "gravi:hud-event",
                            HudEventPayload {
                                kind: "battery_charging".into(),
                                title: "Mengisi Daya".into(),
                                subtitle: format!("Charger terpasang · {}%", curr.percent),
                                mood: "charging".into(),
                                accent: "emerald".into(),
                                badge: Some(format!("{}%", curr.percent)),
                                provider: None,
                                app_name: None,
                            },
                        );
                    } else if prev.charging && !curr.charging && cfg.hud_battery_charging {
                        full_bat_warned = false;
                        let _ = app.emit(
                            "gravi:hud-event",
                            HudEventPayload {
                                kind: "battery_unplugged".into(),
                                title: "Mode Baterai".into(),
                                subtitle: format!("Charger dilepas · sisa {}%", curr.percent),
                                mood: "idle".into(),
                                accent: "amber".into(),
                                badge: Some(format!("{}%", curr.percent)),
                                provider: None,
                                app_name: None,
                            },
                        );
                    }
                }

                if curr.charging && curr.percent >= 100 && !full_bat_warned && cfg.hud_battery_charging {
                    full_bat_warned = true;
                    let _ = app.emit(
                        "gravi:hud-event",
                        HudEventPayload {
                            kind: "battery_full".into(),
                            title: "Baterai Penuh".into(),
                            subtitle: "Daya 100% · Siap dicabut".into(),
                            mood: "battery_full".into(),
                            accent: "emerald".into(),
                            badge: Some("100%".into()),
                            provider: None,
                            app_name: None,
                        },
                    );
                }

                if !curr.charging && curr.percent <= 20 && !low_bat_warned && cfg.hud_battery_low {
                    low_bat_warned = true;
                    let _ = app.emit(
                        "gravi:hud-event",
                        HudEventPayload {
                            kind: "battery_low".into(),
                            title: "Baterai Lemah".into(),
                            subtitle: format!("Sisa {}% · Segera colokkan charger", curr.percent),
                            mood: "low_battery".into(),
                            accent: "rose".into(),
                            badge: Some(format!("{}%", curr.percent)),
                            provider: None,
                            app_name: None,
                        },
                    );
                } else if curr.percent > 25 {
                    low_bat_warned = false;
                }

                last_bat = Some(curr);
            }
        }

        // Wi-Fi
        if tick % 15 == 0 && cfg.hud_wifi {
            let cur_wifi = tokio::task::spawn_blocking(read_wifi_ssid)
                .await
                .ok()
                .flatten();
            if let (Some(prev), Some(curr)) = (&last_wifi, &cur_wifi) {
                if prev != curr {
                    match curr {
                        Some(ssid) => {
                            let _ = app.emit(
                                "gravi:hud-event",
                                HudEventPayload {
                                    kind: "wifi_connected".into(),
                                    title: "Wi-Fi Terhubung".into(),
                                    subtitle: ssid.clone(),
                                    mood: "wifi".into(),
                                    accent: "sky".into(),
                                    badge: Some("Wi-Fi".into()),
                                    provider: None,
                                    app_name: None,
                                },
                            );
                        }
                        None => {
                            let _ = app.emit(
                                "gravi:hud-event",
                                HudEventPayload {
                                    kind: "wifi_disconnected".into(),
                                    title: "Wi-Fi Terputus".into(),
                                    subtitle: prev
                                        .clone()
                                        .map(|s| format!("Terputus dari {}", s))
                                        .unwrap_or_else(|| "Tidak ada koneksi Wi-Fi".into()),
                                    mood: "wifi_off".into(),
                                    accent: "rose".into(),
                                    badge: Some("Offline".into()),
                                    provider: None,
                                    app_name: None,
                                },
                            );
                        }
                    }
                }
            }
            if cur_wifi.is_some() {
                last_wifi = cur_wifi;
            }
        }
    }
}

#[tauri::command]
pub fn trigger_test_hud(app: AppHandle, kind: String) {
    let payload = match kind.as_str() {
        "capslock" => HudEventPayload {
            kind: "capslock".into(),
            title: "Caps Lock".into(),
            subtitle: "Huruf besar aktif".into(),
            mood: "capslock".into(),
            accent: "sky".into(),
            badge: Some("ON".into()),
            provider: None,
            app_name: None,
        },
        "battery_charging" => HudEventPayload {
            kind: "battery_charging".into(),
            title: "Mengisi Daya".into(),
            subtitle: "Charger terpasang · 78%".into(),
            mood: "charging".into(),
            accent: "emerald".into(),
            badge: Some("78%".into()),
            provider: None,
            app_name: None,
        },
        "battery_low" => HudEventPayload {
            kind: "battery_low".into(),
            title: "Baterai Lemah".into(),
            subtitle: "Sisa 15% · Segera colokkan charger".into(),
            mood: "low_battery".into(),
            accent: "rose".into(),
            badge: Some("15%".into()),
            provider: None,
            app_name: None,
        },
        "wifi_connected" => HudEventPayload {
            kind: "wifi_connected".into(),
            title: "Wi-Fi Terhubung".into(),
            subtitle: "Gravi-Fiber-5G".into(),
            mood: "wifi".into(),
            accent: "sky".into(),
            badge: Some("Wi-Fi".into()),
            provider: None,
            app_name: None,
        },
        _ => HudEventPayload {
            kind: "ai_done".into(),
            title: "Claude Selesai".into(),
            subtitle: "Tugas AI selesai — klik untuk fokus".into(),
            mood: "celebrate".into(),
            accent: "emerald".into(),
            badge: Some("Selesai".into()),
            provider: Some("claude".into()),
            app_name: Some("Claude".into()),
        },
    };
    let _ = app.emit("gravi:hud-event", payload);
}
