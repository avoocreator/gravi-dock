mod ai_chat;
mod ai_monitor;
mod calendar;
mod github;
mod ipc_server;
mod media;
mod secrets;
mod settings;
mod system_stats;

use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder,
};
use tokio::sync::Mutex;

pub struct AppState {
    pub ipc: Arc<ipc_server::IpcState>,
    pub monitor_cache: Arc<Mutex<Vec<ai_monitor::AiSession>>>,
}

fn position_island(window: &tauri::WebviewWindow, align: &str) {
    if let Ok(Some(monitor)) = window.current_monitor() {
        let mon_size = monitor.size();
        let mon_pos = monitor.position();
        if let Ok(win_size) = window.outer_size() {
            let x = match align {
                "top-left" => mon_pos.x + 16,
                "top-right" => mon_pos.x + (mon_size.width as i32 - win_size.width as i32) - 16,
                _ => mon_pos.x + ((mon_size.width as i32 - win_size.width as i32) / 2),
            };
            let y = mon_pos.y + 8;
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
    .inner_size(820.0, 560.0)
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
fn set_click_through(window: tauri::WebviewWindow, ignore: bool) -> Result<(), String> {
    window
        .set_ignore_cursor_events(ignore)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn recenter_island(window: tauri::WebviewWindow, position: Option<String>) {
    position_island(&window, position.as_deref().unwrap_or("top-center"));
}

#[tauri::command]
fn sync_island_bounds(
    window: tauri::WebviewWindow,
    width: f64,
    height: f64,
    position: Option<String>,
) -> Result<(), String> {
    let w = width.clamp(110.0, 640.0);
    let h = height.clamp(24.0, 760.0);
    let scale = window.scale_factor().unwrap_or(1.0);
    let phys_w = (w * scale).round() as i32;

    window
        .set_size(tauri::LogicalSize::new(w, h))
        .map_err(|e| e.to_string())?;

    let align = position.as_deref().unwrap_or("top-center");
    if let Ok(Some(monitor)) = window.current_monitor() {
        let mon_size = monitor.size();
        let mon_pos = monitor.position();
        let x = match align {
            "top-left" => mon_pos.x + 16,
            "top-right" => mon_pos.x + (mon_size.width as i32 - phys_w) - 16,
            _ => mon_pos.x + ((mon_size.width as i32 - phys_w) / 2),
        };
        let y = mon_pos.y + 8;
        let _ = window.set_position(PhysicalPosition::new(x, y));
    }
    Ok(())
}

#[tauri::command]
fn get_ipc_info(state: tauri::State<'_, AppState>) -> ipc_server::IpcInfo {
    state.ipc.info()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let ipc_state = Arc::new(ipc_server::IpcState::new());
    let monitor_cache = Arc::new(Mutex::new(Vec::new()));

    let app_state = AppState {
        ipc: ipc_state.clone(),
        monitor_cache: monitor_cache.clone(),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(app_state)
        .setup(move |app| {
            let handle = app.handle().clone();
            let saved = settings::get_settings(handle.clone());

            if let Some(window) = app.get_webview_window("main") {
                position_island(&window, &saved.island_position);
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
                            position_island(&w, &cfg.island_position);
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

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_settings_window,
            set_click_through,
            recenter_island,
            sync_island_bounds,
            get_ipc_info,
            settings::get_settings,
            settings::save_settings,
            settings::reset_settings,
            system_stats::get_system_stats,
            ai_monitor::get_ai_sessions,
            ai_monitor::focus_ai_window,
            ipc_server::respond_permission,
            ipc_server::get_pending_permissions,
            github::fetch_github_summary,
            calendar::fetch_calendar_events,
            media::get_media_state,
            media::media_control,
            ai_chat::send_chat_message,
            secrets::save_secret,
            secrets::get_secret_status,
            secrets::delete_secret
        ])
        .run(tauri::generate_context!())
        .expect("error while running Nobi Island");
}
