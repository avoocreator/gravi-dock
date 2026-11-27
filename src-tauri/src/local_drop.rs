use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::UdpSocket;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, oneshot, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedDropFile {
    pub id: String,
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivedDropItem {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub content_or_path: String,
    pub size_bytes: u64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalDropStatus {
    pub running: bool,
    pub url: String,
    pub port: u16,
    pub ip: String,
    pub connected_clients: usize,
    pub shared_files: Vec<SharedDropFile>,
    pub received_items: Vec<ReceivedDropItem>,
    pub save_dir: String,
}

pub struct LocalDropState {
    pub running: Mutex<bool>,
    pub port: Mutex<u16>,
    pub ip: Mutex<String>,
    pub connected_clients: AtomicUsize,
    pub shutdown_tx: Mutex<Option<oneshot::Sender<()>>>,
    pub shared_files: Mutex<Vec<SharedDropFile>>,
    pub received_items: Mutex<Vec<ReceivedDropItem>>,
    pub sse_tx: broadcast::Sender<String>,
}

impl LocalDropState {
    pub fn new() -> Self {
        let (sse_tx, _) = broadcast::channel(64);
        Self {
            running: Mutex::new(false),
            port: Mutex::new(0),
            ip: Mutex::new(detect_lan_ip()),
            connected_clients: AtomicUsize::new(0),
            shutdown_tx: Mutex::new(None),
            shared_files: Mutex::new(Vec::new()),
            received_items: Mutex::new(Vec::new()),
            sse_tx,
        }
    }
}

fn detect_lan_ip() -> String {
    if let Ok(sock) = UdpSocket::bind("0.0.0.0:0") {
        if sock.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = sock.local_addr() {
                return addr.ip().to_string();
            }
        }
    }
    "127.0.0.1".to_string()
}

fn drop_save_dir() -> PathBuf {
    let base = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("Downloads").join("NobiDrop");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn sanitize_filename(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        format!("file-{}", chrono::Utc::now().timestamp())
    } else {
        trimmed.to_string()
    }
}

fn url_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(hex);
                i += 3;
                continue;
            }
        } else if bytes[i] == b'+' {
            out.push(b' ');
            i += 1;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

fn mobile_web_ui(files: &[SharedDropFile]) -> String {
    let files_json = serde_json::to_string(files).unwrap_or_else(|_| "[]".to_string());
    format!(
        r#"<!DOCTYPE html>
<html lang="id"><head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Nobi iDrop · Instant Transfer</title>
<style>
*{{box-sizing:border-box;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif}}
body{{margin:0;background:#09090f;color:#f4f4f8;padding:16px;max-width:480px;margin:0 auto}}
.hdr{{display:flex;justify-content:space-between;align-items:center;margin-bottom:14px}}
h1{{font-size:18px;margin:0}}.badge{{font-size:11px;padding:4px 10px;border-radius:999px;background:rgba(74,222,128,.16);color:#4ade80;border:1px solid rgba(74,222,128,.35)}}
.card{{background:#13131d;border:1px solid rgba(255,255,255,.09);border-radius:16px;padding:14px;margin-bottom:12px}}
.lbl{{font-size:11px;text-transform:uppercase;letter-spacing:.06em;color:#9ca3af;margin-bottom:8px;font-weight:600;display:flex;justify-content:space-between;align-items:center}}
.btn{{display:block;width:100%;padding:11px;border-radius:12px;border:0;background:#38bdf8;color:#08080c;font-weight:700;font-size:14px;cursor:pointer;margin-top:8px}}
.btn:disabled{{opacity:.5}}
textarea,input[type=file]{{width:100%;background:#09090f;color:#fff;border:1px solid rgba(255,255,255,.14);border-radius:12px;padding:10px;font-size:14px}}
.file-row{{display:flex;justify-content:space-between;align-items:center;padding:10px 12px;border-radius:11px;background:rgba(255,255,255,.05);color:#fff;text-decoration:none;margin-bottom:6px;font-size:13px}}
.sz{{color:#38bdf8;font-size:11px;font-family:monospace;margin-left:8px;white-space:nowrap}}
.empty{{font-size:12px;color:#6b7280;text-align:center;padding:8px}}
.prog{{height:5px;border-radius:99px;background:rgba(255,255,255,.1);overflow:hidden;margin-top:8px;display:none}}
.prog>div{{height:100%;background:#4ade80;width:0%;transition:width .12s}}
#st{{font-size:12px;color:#4ade80;margin-top:8px;min-height:16px}}
label.tog{{font-size:11px;color:#cbd5e1;display:flex;align-items:center;gap:6px;text-transform:none;letter-spacing:0;font-weight:500}}
</style></head><body>
<div class="hdr">
  <h1>Nobi iDrop</h1>
  <span class="badge" id="liveBadge">● Terhubung</span>
</div>

<div class="card">
  <div class="lbl">
    <span>Masuk dari Laptop (iDrop)</span>
    <label class="tog"><input type="checkbox" id="autoSave" checked> Auto-simpan</label>
  </div>
  <div id="fileList"></div>
  <div id="autoSt" style="font-size:11px;color:#38bdf8;margin-top:4px"></div>
</div>

<div class="card">
  <div class="lbl"><span>Kirim File ke Laptop (Langsung ke Storage)</span></div>
  <input type="file" id="fi" multiple>
  <div class="prog" id="progWrap"><div id="progBar"></div></div>
  <button class="btn" id="upBtn" onclick="sendFiles()">Kirim Cepat ke Laptop</button>
  <div id="st"></div>
</div>

<div class="card">
  <div class="lbl"><span>Kirim Teks / Link ke Laptop</span></div>
  <textarea id="tx" rows="2" placeholder="Paste link atau catatan..."></textarea>
  <button class="btn" style="background:#4ade80" onclick="sendText()">Kirim Teks</button>
</div>

<script>
let files = {files_json};
function fmtSz(b){{return b>=1048576?(b/1048576).toFixed(1)+' MB':Math.max(1,Math.round(b/1024))+' KB';}}
function renderFiles(){{
  const el=document.getElementById('fileList');
  if(!files.length){{el.innerHTML='<div class="empty">Drop file ke Nobi Island di laptop → otomatis tersimpan ke HP!</div>';return;}}
  el.innerHTML=files.map(f=>`<a class="file-row" href="/download/${{f.id}}" download="${{f.name}}"><span style="overflow:hidden;text-overflow:ellipsis;white-space:nowrap">${{f.name}}</span><span class="sz">${{fmtSz(f.size_bytes)}}</span></a>`).join('');
}}
function triggerAutoSave(f){{
  const a=document.createElement('a');
  a.href='/download/'+f.id;
  a.download=f.name;
  document.body.appendChild(a);
  a.click();
  a.remove();
  document.getElementById('autoSt').textContent='⚡ Otomatis menyimpan '+f.name+' ke penyimpanan HP...';
}}
renderFiles();

const ev=new EventSource('/events');
ev.onmessage=(e)=>{{
  try{{
    const msg=JSON.parse(e.data);
    if(msg.type==='new_file'&&msg.file){{
      if(!files.some(x=>x.id===msg.file.id)) files.unshift(msg.file);
      renderFiles();
      if(document.getElementById('autoSave').checked) triggerAutoSave(msg.file);
    }} else if(msg.type==='remove_file'){{
      files=files.filter(x=>x.id!==msg.id);
      renderFiles();
    }}
  }}catch{{}}
}};
ev.onerror=()=>{{document.getElementById('liveBadge').textContent='○ Menghubungkan...';}};
ev.onopen=()=>{{document.getElementById('liveBadge').textContent='● iDrop Aktif';}};

function uploadOne(f, onProg){{
  return new Promise((resolve,reject)=>{{
    const xhr=new XMLHttpRequest();
    xhr.open('POST','/upload?name='+encodeURIComponent(f.name));
    xhr.upload.onprogress=(e)=>{{if(e.lengthComputable) onProg(Math.round((e.loaded/e.total)*100));}};
    xhr.onload=()=>xhr.status===200?resolve():reject(new Error('HTTP '+xhr.status));
    xhr.onerror=()=>reject(new Error('Network error'));
    xhr.send(f);
  }});
}}
async function sendFiles(){{
  const inp=document.getElementById('fi'),st=document.getElementById('st'),btn=document.getElementById('upBtn');
  const pw=document.getElementById('progWrap'),pb=document.getElementById('progBar');
  if(!inp.files.length){{st.textContent='Pilih file dulu.';return;}}
  btn.disabled=true; pw.style.display='block';
  for(const f of inp.files){{
    pb.style.width='0%';
    st.textContent='Mengirim '+f.name+'...';
    try{{
      await uploadOne(f,(p)=>{{pb.style.width=p+'%'; st.textContent='Mengirim '+f.name+' ('+p+'%)';}});
    }}catch(err){{
      st.textContent='Gagal mengirim '+f.name; btn.disabled=false; return;
    }}
  }}
  pb.style.width='100%';
  st.textContent='✓ Langsung tersimpan ke Downloads/NobiDrop!';
  inp.value=''; btn.disabled=false;
}}
async function sendText(){{
  const tx=document.getElementById('tx'),st=document.getElementById('st');
  if(!tx.value.trim())return;
  const r=await fetch('/text',{{method:'POST',body:tx.value}});
  if(r.ok){{st.textContent='✓ Teks terkirim ke Nobi Island!';tx.value='';}}
}}
</script></body></html>"#,
        files_json = files_json
    )
}

async fn handle_http_connection(
    mut stream: tokio::net::TcpStream,
    app: AppHandle,
    state: Arc<LocalDropState>,
) {
    let _ = stream.set_nodelay(true);
    let mut header_buf = Vec::with_capacity(4096);
    let mut temp = [0u8; 8192];
    let header_end;

    loop {
        let Ok(n) = stream.read(&mut temp).await else {
            return;
        };
        if n == 0 {
            return;
        }
        header_buf.extend_from_slice(&temp[..n]);
        if let Some(pos) = header_buf.windows(4).position(|w| w == b"\r\n\r\n") {
            header_end = pos + 4;
            break;
        }
        if header_buf.len() > 64 * 1024 {
            return;
        }
    }

    let head_str = String::from_utf8_lossy(&header_buf[..header_end]).to_string();
    let mut lines = head_str.lines();
    let first_line = lines.next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }
    let method = parts[0];
    let target = parts[1];

    let mut headers: HashMap<String, String> = HashMap::new();
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            headers.insert(k.trim().to_lowercase(), v.trim().to_string());
        }
    }
    let content_length: usize = headers
        .get("content-length")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let leftover = &header_buf[header_end..];

    // SSE stream
    if method == "GET" && target == "/events" {
        let sse_hdr = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n: connected\n\n";
        if stream.write_all(sse_hdr.as_bytes()).await.is_err() {
            return;
        }
        state.connected_clients.fetch_add(1, Ordering::Relaxed);
        let _ = app.emit("nobi:drop-clients", state.connected_clients.load(Ordering::Relaxed));

        let mut rx = state.sse_tx.subscribe();
        loop {
            tokio::select! {
                msg = rx.recv() => {
                    match msg {
                        Ok(data) => {
                            let frame = format!("data: {}\n\n", data);
                            if stream.write_all(frame.as_bytes()).await.is_err() {
                                break;
                            }
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                        Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    }
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(15)) => {
                    if stream.write_all(b": ping\n\n").await.is_err() {
                        break;
                    }
                }
            }
        }
        state.connected_clients.fetch_sub(1, Ordering::Relaxed);
        let _ = app.emit("nobi:drop-clients", state.connected_clients.load(Ordering::Relaxed));
        return;
    }

    if method == "GET" && (target == "/" || target.starts_with("/?")) {
        let files = state.shared_files.lock().await.clone();
        let html = mobile_web_ui(&files);
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            html.len(),
            html
        );
        let _ = stream.write_all(resp.as_bytes()).await;
    } else if method == "GET" && target.starts_with("/download/") {
        // File download
        let id = target.trim_start_matches("/download/");
        let found = {
            let files = state.shared_files.lock().await;
            files.iter().find(|f| f.id == id).cloned()
        };
        if let Some(f) = found {
            if let Ok(mut file) = tokio::fs::File::open(&f.path).await {
                let len = file.metadata().await.map(|m| m.len()).unwrap_or(f.size_bytes);
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    f.name.replace('"', "_"),
                    len
                );
                if stream.write_all(header.as_bytes()).await.is_ok() {
                    let mut buf = vec![0u8; 256 * 1024];
                    loop {
                        match file.read(&mut buf).await {
                            Ok(0) => break,
                            Ok(n) => {
                                if stream.write_all(&buf[..n]).await.is_err() {
                                    break;
                                }
                            }
                            Err(_) => break,
                        }
                    }
                }
                return;
            }
        }
        let _ = stream
            .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\nConnection: close\r\n\r\nNot Found")
            .await;
    } else if method == "POST" && target.starts_with("/upload") {
        // File upload
        let raw_name = target
            .split("name=")
            .nth(1)
            .map(|s| s.split('&').next().unwrap_or(s))
            .map(url_decode)
            .unwrap_or_else(|| format!("upload-{}", chrono::Utc::now().timestamp()));
        let safe_name = sanitize_filename(&raw_name);
        let save_path = drop_save_dir().join(&safe_name);

        if let Ok(mut out_file) = tokio::fs::File::create(&save_path).await {
            let mut written: usize = 0;
            if !leftover.is_empty() {
                let to_write = leftover.len().min(content_length);
                if out_file.write_all(&leftover[..to_write]).await.is_ok() {
                    written += to_write;
                }
            }
            let mut buf = vec![0u8; 256 * 1024];
            while written < content_length {
                let remain = content_length - written;
                let max_read = remain.min(buf.len());
                match stream.read(&mut buf[..max_read]).await {
                    Ok(0) => break,
                    Ok(n) => {
                        if out_file.write_all(&buf[..n]).await.is_err() {
                            break;
                        }
                        written += n;
                    }
                    Err(_) => break,
                }
            }
            let _ = out_file.flush().await;

            let item = ReceivedDropItem {
                id: Uuid::new_v4().to_string(),
                kind: "file".into(),
                name: safe_name,
                content_or_path: save_path.to_string_lossy().to_string(),
                size_bytes: written as u64,
                timestamp: chrono::Utc::now().timestamp_millis(),
            };
            {
                let mut rec = state.received_items.lock().await;
                rec.insert(0, item.clone());
                rec.truncate(20);
            }
            let _ = app.emit("nobi:drop-received", &item);
            let _ = stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nOK")
                .await;
        } else {
            let _ = stream
                .write_all(b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 5\r\nConnection: close\r\n\r\nError")
                .await;
        }
    } else if method == "POST" && target == "/text" {
        let mut body = leftover.to_vec();
        let mut buf = [0u8; 4096];
        while body.len() < content_length && body.len() < 1024 * 1024 {
            let Ok(n) = stream.read(&mut buf).await else { break };
            if n == 0 { break; }
            body.extend_from_slice(&buf[..n]);
        }
        body.truncate(content_length);
        let text = String::from_utf8_lossy(&body).trim().to_string();
        if !text.is_empty() {
            let item = ReceivedDropItem {
                id: Uuid::new_v4().to_string(),
                kind: "text".into(),
                name: "Teks dari HP".into(),
                content_or_path: text.clone(),
                size_bytes: text.len() as u64,
                timestamp: chrono::Utc::now().timestamp_millis(),
            };
            {
                let mut rec = state.received_items.lock().await;
                rec.insert(0, item.clone());
                rec.truncate(20);
            }
            let _ = app.emit("nobi:drop-received", &item);
        }
        let _ = stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nOK")
            .await;
    } else {
        let _ = stream
            .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\nConnection: close\r\n\r\nNot Found")
            .await;
    }
}

async fn build_status(state: &LocalDropState) -> LocalDropStatus {
    let running = *state.running.lock().await;
    let port = *state.port.lock().await;
    let ip = state.ip.lock().await.clone();
    let connected_clients = state.connected_clients.load(Ordering::Relaxed);
    let shared_files = state.shared_files.lock().await.clone();
    let received_items = state.received_items.lock().await.clone();
    let url = if running && port > 0 {
        format!("http://{}:{}", ip, port)
    } else {
        String::new()
    };
    LocalDropStatus {
        running,
        url,
        port,
        ip,
        connected_clients,
        shared_files,
        received_items,
        save_dir: drop_save_dir().to_string_lossy().to_string(),
    }
}

#[tauri::command]
pub async fn get_local_drop_status(
    state: tauri::State<'_, crate::AppState>,
) -> Result<LocalDropStatus, String> {
    Ok(build_status(&state.local_drop).await)
}

#[tauri::command]
pub async fn start_local_drop(
    app: AppHandle,
    state: tauri::State<'_, crate::AppState>,
) -> Result<LocalDropStatus, String> {
    let ld = state.local_drop.clone();
    if *ld.running.lock().await {
        return Ok(build_status(&ld).await);
    }

    let listener = match TcpListener::bind("0.0.0.0:47899").await {
        Ok(l) => l,
        Err(_) => TcpListener::bind("0.0.0.0:0")
            .await
            .map_err(|e| format!("Gagal menyalakan server Local Drop: {}", e))?,
    };
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let ip = detect_lan_ip();

    let (tx, mut rx) = oneshot::channel::<()>();
    {
        *ld.running.lock().await = true;
        *ld.port.lock().await = port;
        *ld.ip.lock().await = ip;
        *ld.shutdown_tx.lock().await = Some(tx);
    }

    let ld_loop = ld.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut rx => {
                    break;
                }
                accept_res = listener.accept() => {
                    if let Ok((stream, _)) = accept_res {
                        let app_c = app.clone();
                        let ld_c = ld_loop.clone();
                        tokio::spawn(async move {
                            handle_http_connection(stream, app_c, ld_c).await;
                        });
                    }
                }
            }
        }
        *ld_loop.running.lock().await = false;
        *ld_loop.port.lock().await = 0;
        ld_loop.connected_clients.store(0, Ordering::Relaxed);
    });

    Ok(build_status(&ld).await)
}

#[tauri::command]
pub async fn stop_local_drop(
    state: tauri::State<'_, crate::AppState>,
) -> Result<LocalDropStatus, String> {
    let ld = state.local_drop.clone();
    if let Some(tx) = ld.shutdown_tx.lock().await.take() {
        let _ = tx.send(());
    }
    *ld.running.lock().await = false;
    *ld.port.lock().await = 0;
    ld.connected_clients.store(0, Ordering::Relaxed);
    Ok(build_status(&ld).await)
}

#[tauri::command]
pub async fn add_local_drop_files(
    state: tauri::State<'_, crate::AppState>,
    paths: Vec<String>,
) -> Result<LocalDropStatus, String> {
    let ld = state.local_drop.clone();
    {
        let mut files = ld.shared_files.lock().await;
        for path in paths {
            if files.iter().any(|f| f.path == path) {
                continue;
            }
            let p = PathBuf::from(&path);
            let name = p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());
            let size_bytes = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
            let item = SharedDropFile {
                id: Uuid::new_v4().to_string(),
                name,
                path,
                size_bytes,
            };
            if let Ok(evt) = serde_json::to_string(&serde_json::json!({
                "type": "new_file",
                "file": &item
            })) {
                let _ = ld.sse_tx.send(evt);
            }
            files.insert(0, item);
        }
    }
    Ok(build_status(&ld).await)
}

#[tauri::command]
pub async fn remove_local_drop_file(
    state: tauri::State<'_, crate::AppState>,
    id: String,
) -> Result<LocalDropStatus, String> {
    let ld = state.local_drop.clone();
    {
        let mut files = ld.shared_files.lock().await;
        files.retain(|f| f.id != id);
    }
    if let Ok(evt) = serde_json::to_string(&serde_json::json!({
        "type": "remove_file",
        "id": id
    })) {
        let _ = ld.sse_tx.send(evt);
    }
    Ok(build_status(&ld).await)
}
