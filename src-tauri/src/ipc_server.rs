use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Mutex};
use tokio::time::{timeout, Duration};
use uuid::Uuid;

pub const IPC_ADDR: &str = "127.0.0.1:47891";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    pub id: String,
    pub agent: String,
    pub tool: String,
    pub command: String,
    pub cwd: Option<String>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookEvent {
    pub event_type: String,
    pub agent: String,
    pub tool: Option<String>,
    pub command: Option<String>,
    pub summary: Option<String>,
    pub file_path: Option<String>,
    pub added: Option<u32>,
    pub removed: Option<u32>,
    pub cwd: Option<String>,
}

pub struct IpcState {
    pending: Mutex<HashMap<String, (PermissionRequest, oneshot::Sender<String>)>>,
}

impl IpcState {
    pub fn new() -> Self {
        Self {
            pending: Mutex::new(HashMap::new()),
        }
    }

}

pub async fn start_server(app: AppHandle, state: Arc<IpcState>) {
    let listener = match TcpListener::bind(IPC_ADDR).await {
        Ok(l) => l,
        Err(err) => {
            eprintln!("[gravi-ipc] failed to bind {}: {}", IPC_ADDR, err);
            return;
        }
    };

    loop {
        let Ok((stream, _)) = listener.accept().await else {
            continue;
        };
        let app_clone = app.clone();
        let state_clone = state.clone();

        tokio::spawn(async move {
            let (reader, mut writer) = stream.into_split();
            let mut buf_reader = BufReader::new(reader);
            let mut line = String::new();
            if buf_reader.read_line(&mut line).await.is_err() || line.trim().is_empty() {
                return;
            }

            let Ok(event) = serde_json::from_str::<HookEvent>(line.trim()) else {
                let _ = writer.write_all(b"{\"decision\":\"allow\"}\n").await;
                return;
            };

            if event.event_type == "permission" {
                let req_id = Uuid::new_v4().to_string();
                let req = PermissionRequest {
                    id: req_id.clone(),
                    agent: event.agent.clone(),
                    tool: event.tool.unwrap_or_else(|| "shell".into()),
                    command: event.command.unwrap_or_else(|| "(perintah tidak disebut)".into()),
                    cwd: event.cwd.clone(),
                    timestamp: chrono::Utc::now().timestamp_millis(),
                };

                let (tx, rx) = oneshot::channel::<String>();
                {
                    let mut guard = state_clone.pending.lock().await;
                    guard.insert(req_id.clone(), (req.clone(), tx));
                }

                let _ = app_clone.emit("gravi:permission-request", &req);

                let decision = match timeout(Duration::from_secs(300), rx).await {
                    Ok(Ok(d)) => d,
                    _ => {
                        let mut guard = state_clone.pending.lock().await;
                        guard.remove(&req_id);
                        "deny".to_string()
                    }
                };

                let resp = format!("{{\"decision\":\"{}\"}}\n", decision);
                let _ = writer.write_all(resp.as_bytes()).await;
            } else {
                let _ = app_clone.emit("gravi:agent-event", &event);
                if event.event_type == "done" {
                    let cfg = crate::settings::get_settings(app_clone.clone());
                    if cfg.hud_enabled && cfg.hud_ai_done {
                        let _ = app_clone.emit(
                            "gravi:hud-event",
                            crate::system_hud::HudEventPayload {
                                kind: "ai_done".into(),
                                title: format!("{} Selesai", event.agent),
                                subtitle: event
                                    .summary
                                    .clone()
                                    .unwrap_or_else(|| "Tugas CLI selesai".into()),
                                mood: "celebrate".into(),
                                accent: "emerald".into(),
                                badge: Some("Selesai".into()),
                                provider: None,
                                app_name: Some(event.agent.clone()),
                            },
                        );
                    }
                }
                let _ = writer.write_all(b"{\"ok\":true}\n").await;
            }
        });
    }
}

#[tauri::command]
pub async fn respond_permission(
    state: tauri::State<'_, crate::AppState>,
    id: String,
    decision: String,
) -> Result<(), String> {
    let mut guard = state.ipc.pending.lock().await;
    if let Some((_, tx)) = guard.remove(&id) {
        let normalized = match decision.as_str() {
            "allow" | "always" | "deny" => decision,
            _ => "deny".to_string(),
        };
        let _ = tx.send(normalized);
        Ok(())
    } else {
        Err("Permintaan izin sudah tidak aktif atau kedaluwarsa.".into())
    }
}

#[tauri::command]
pub async fn get_pending_permissions(
    state: tauri::State<'_, crate::AppState>,
) -> Result<Vec<PermissionRequest>, String> {
    let guard = state.ipc.pending.lock().await;
    Ok(guard.values().map(|(req, _)| req.clone()).collect())
}
