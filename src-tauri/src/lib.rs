mod ai_chat;
mod ai_monitor;
mod ai_providers;
mod audio_mixer;
mod calendar;
mod github;
mod ipc_server;
mod local_drop;
mod media;
mod network_bt;
mod notifications;
mod secrets;
mod settings;
mod system_hud;
mod system_stats;

use serde::Serialize;
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder,
};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize)]
struct CursorPosPayload {
    x: f64,
    y: f64,
}

pub struct AppState {
    pub ipc: Arc<ipc_server::IpcState>,
    pub monitor_cache: Arc<Mutex<Vec<ai_monitor::AiSession>>>,
    pub hit_rect: Arc<std::sync::Mutex<(f64, f64, f64, f64)>>,
    pub local_drop: Arc<local_drop::LocalDropState>,
}

pub(crate) fn position_island(window: &tauri::WebviewWindow, align: &str, style: &str) {
    if let Ok(Some(monitor)) = window.current_monitor() {
        let mon_size = monitor.size();
        let mon_pos = monitor.position();
        if let Ok(win_size) = window.outer_size() {
            let edge_margin = if style == "attached" { 0 } else { 16 };
            let x = match align {
                "top-left" => mon_pos.x + edge_margin,
                "top-right" => mon_pos.x + (mon_size.width as i32 - win_size.width as i32) - edge_margin,
                _ => mon_pos.x + ((mon_size.width as i32 - win_size.width as i32) / 2),
            };
            let y = if style == "attached" {
                mon_pos.y
            } else {
                mon_pos.y + 8
            };
            let _ = window.set_position(PhysicalPosition::new(x, y));
        }
    }
}

fn spawn_or_focus_settings(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(existing) = app.get_webview_window("settings") {
        let _ = existing.show();
        let _ = existing.unminimize();
        let _ = existing.set_focus();
        return Ok(());
    }

    WebviewWindowBuilder::new(
        app,
        "settings",
        WebviewUrl::App("index.html".into()),
    )
    .title("Pengaturan · Nobi Dynamic Island")
    .inner_size(840.0, 580.0)
    .min_inner_size(680.0, 460.0)
    .center()
    .resizable(true)
    .decorations(true)
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn open_settings_window(app: tauri::AppHandle) -> Result<(), String> {
    spawn_or_focus_settings(&app)
}

#[tauri::command]
fn recenter_island(
    app: tauri::AppHandle,
    window: tauri::WebviewWindow,
    position: Option<String>,
    style: Option<String>,
) {
    let saved = settings::get_settings(app);
    let align = position.as_deref().unwrap_or(&saved.island_position);
    let st = style.as_deref().unwrap_or(&saved.island_style);
    position_island(&window, align, st);
}

#[tauri::command]
fn update_hit_rect(
    state: tauri::State<'_, AppState>,
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
) {
    if let Ok(mut guard) = state.hit_rect.lock() {
        *guard = (left, top, right, bottom);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let ipc_state = Arc::new(ipc_server::IpcState::new());
    let monitor_cache = Arc::new(Mutex::new(Vec::new()));
    let hit_rect_state = Arc::new(std::sync::Mutex::new((170.0, 0.0, 370.0, 52.0)));
    let local_drop_state = Arc::new(local_drop::LocalDropState::new());

    let app_state = AppState {
        ipc: ipc_state.clone(),
        monitor_cache: monitor_cache.clone(),
        hit_rect: hit_rect_state.clone(),
        local_drop: local_drop_state.clone(),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(app_state)
        .setup(move |app| {
            let handle = app.handle().clone();
            let saved = settings::get_settings(handle.clone());

            if let Some(window) = app.get_webview_window("main") {
                position_island(&window, &saved.island_position, &saved.island_style);
            }

            let show_item = MenuItem::with_id(app, "show", "Tampilkan Nobi Island", true, None::<&str>)?;
            let settings_item = MenuItem::with_id(app, "settings", "Pengaturan...", true, None::<&str>)?;
            let center_item = MenuItem::with_id(app, "recenter", "Posisikan Ulang", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Keluar", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &settings_item, &center_item, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("Nobi Dynamic Island")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.emit("nobi:toggle-expand", ());
                        }
                    }
                    "settings" => {
                        let app_handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = spawn_or_focus_settings(&app_handle);
                        });
                    }
                    "recenter" => {
                        let cfg = settings::get_settings(app.clone());
                        if let Some(w) = app.get_webview_window("main") {
                            position_island(&w, &cfg.island_position, &cfg.island_style);
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            let ipc_handle = handle.clone();
            let ipc_ref = ipc_state.clone();
            tauri::async_runtime::spawn(async move {
                ipc_server::start_server(ipc_handle, ipc_ref).await;
            });

            let mon_handle = handle.clone();
            let mon_cache = monitor_cache.clone();
            tauri::async_runtime::spawn(async move {
                ai_monitor::start_watcher(mon_handle, mon_cache).await;
            });

            let sys_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                system_stats::start_system_watcher(sys_handle).await;
            });

            let hud_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                system_hud::start_hud_watcher(hud_handle).await;
            });

            let notif_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                notifications::start_notification_watcher(notif_handle).await;
            });

            let bt_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                network_bt::start_bt_watcher(bt_handle).await;
            });

            // Unified cursor hit-test & global desktop eye-tracking emitter (~40 Hz)
            let hit_handle = handle.clone();
            let hit_rect = hit_rect_state.clone();
            tauri::async_runtime::spawn(async move {
                let mut last_ignore: Option<bool> = None;
                let mut last_cursor: Option<(f64, f64)> = None;
                loop {
                    if let Some(w) = hit_handle.get_webview_window("main") {
                        if let (Ok(cursor), Ok(win_pos), Ok(scale)) =
                            (w.cursor_position(), w.inner_position(), w.scale_factor())
                        {
                            let lx = (cursor.x - win_pos.x as f64) / scale;
                            let ly = (cursor.y - win_pos.y as f64) / scale;

                            let moved = match last_cursor {
                                Some((px, py)) => (lx - px).abs() >= 1.5 || (ly - py).abs() >= 1.5,
                                None => true,
                            };
                            if moved {
                                last_cursor = Some((lx, ly));
                                let _ = w.emit("nobi:cursor-pos", CursorPosPayload { x: lx, y: ly });
                            }

                            let (l, t, r, b) = hit_rect
                                .lock()
                                .map(|g| *g)
                                .unwrap_or((170.0, 0.0, 370.0, 52.0));
                            let pad = 6.0;
                            let inside = lx >= (l - pad)
                                && lx <= (r + pad)
                                && ly >= (t - pad)
                                && ly <= (b + pad);
                            let should_ignore = !inside;
                            if last_ignore != Some(should_ignore)
                                && w.set_ignore_cursor_events(should_ignore).is_ok()
                            {
                                last_ignore = Some(should_ignore);
                                let _ = w.emit(
                                    if should_ignore {
                                        "nobi:cursor-leave"
                                    } else {
                                        "nobi:cursor-enter"
                                    },
                                    (),
                                );
                            }
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(25)).await;
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_settings_window,
            recenter_island,
            update_hit_rect,
            settings::get_settings,
            settings::save_settings,
            settings::reset_settings,
            system_stats::get_system_stats,
            ai_monitor::get_ai_sessions,
            ai_monitor::focus_ai_window,
            ai_providers::detect_ai_providers,
            ai_providers::fetch_provider_models,
            ipc_server::respond_permission,
            ipc_server::get_pending_permissions,
            github::fetch_github_summary,
            calendar::fetch_calendar_events,
            media::get_media_state,
            media::media_control,
            media::media_seek,
            ai_chat::send_chat_message,
            secrets::save_secret,
            secrets::get_secret_status,
            system_hud::trigger_test_hud,
            notifications::reply_os_notification,
            notifications::trigger_test_notification,
            local_drop::get_local_drop_status,
            local_drop::start_local_drop,
            local_drop::stop_local_drop,
            local_drop::add_local_drop_files,
            local_drop::remove_local_drop_file,
            network_bt::get_hardware_status,
            network_bt::get_wifi_credentials,
            network_bt::open_bluetooth_settings,
            audio_mixer::list_audio_sessions,
            audio_mixer::set_app_audio_volume,
            audio_mixer::set_app_audio_mute
        ])
        .run(tauri::generate_context!())
        .expect("error while running Nobi Island");
}
