//! Data online di thread latar: cuaca, kalender (Google OAuth atau iCal), dan GitHub. Hasil dibagikan lewat `Shared`.
use crate::net::{self, Req};
use island_core::github::{self, Snapshot};
use island_core::gcal;
use island_core::ics::{self, Event};
use island_core::json::{self, Value};
use island_core::settings::Settings;
use island_core::timeutil::local_day;
use island_core::weather;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Default, Clone)]
pub struct Shared {
    pub weather: String,
    pub ics_events: Vec<Event>,
    pub gcal_events: Vec<Event>,
    pub gcal_connected: bool,
    pub gcal_status: String,
    pub gh: Option<Snapshot>,
    pub gh_status: String,
    pub offline: bool,
}

pub type Handle = Arc<Mutex<Shared>>;
pub type Cfg = Arc<Mutex<Settings>>;

#[derive(Clone, Copy)]
pub enum Kick { Weather, Calendar, Github, GcalConnect, GcalDisconnect }

pub struct Online { pub shared: Handle, tx: mpsc::Sender<Kick> }
impl Online {
    pub fn kick(&self, k: Kick) { let _ = self.tx.send(k); }
}

/* ---------- token Google (disimpan di folder data pengguna) ---------- */

#[derive(Clone, Default, Debug, PartialEq)]
struct Tokens { access: String, refresh: String, exp: i64 }

fn token_path(data: &Path) -> PathBuf { data.join("gcal_token.json") }
fn load_tokens(data: &Path) -> Tokens {
    let Some(v) = std::fs::read_to_string(token_path(data)).ok().and_then(|s| json::parse(&s)) else { return Tokens::default() };
    Tokens { access: v.get("access").text().into(), refresh: v.get("refresh").text().into(), exp: v.get("exp").num().unwrap_or(0.0) as i64 }
}
fn save_tokens(data: &Path, t: &Tokens) {
    let v = Value::obj(vec![("access", Value::s(&t.access)), ("refresh", Value::s(&t.refresh)), ("exp", Value::n(t.exp as f64))]);
    let _ = std::fs::write(token_path(data), v.pretty());
}

fn now_s() -> i64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0) }

/// Pastikan access token masih berlaku (refresh bila perlu). Err = teks status untuk UI.
fn fresh_access(data: &Path, cfg: &Settings) -> Result<String, String> {
    let mut t = load_tokens(data);
    if t.refresh.is_empty() { return Err("Belum terhubung".into()); }
    if !t.access.is_empty() && t.exp > now_s() + 60 { return Ok(t.access); }
    let body = gcal::refresh_body(&cfg.gcal_client_id, &cfg.gcal_client_secret, &t.refresh);
    let (code, text) = net::send(&Req::post(gcal::TOKEN_URL, &body).header("Content-Type: application/x-www-form-urlencoded")).ok_or("Tidak ada koneksi")?;
    let tok = gcal::parse_token(&text).map_err(|e| if code == 400 || code == 401 { "Izin kedaluwarsa, hubungkan ulang".to_string() } else { e })?;
    t.access = tok.access.clone(); t.exp = now_s() + tok.expires_in; if let Some(r) = tok.refresh { t.refresh = r; }
    save_tokens(data, &t);
    Ok(tok.access)
}

fn fetch_gcal(data: &Path, cfg: &Settings, sh: &Handle) {
    match fresh_access(data, cfg) {
        Err(e) => { let mut s = sh.lock().unwrap(); s.gcal_connected = false; s.gcal_status = e; }
        Ok(tok) => {
            let now = now_s();
            let url = gcal::events_url(now - 3600, now + 8 * 86400);
            match net::send(&Req::get(&url).header(format!("Authorization: Bearer {tok}"))) {
                None => sh.lock().unwrap().gcal_status = "Tidak ada koneksi".into(),
                Some((_, text)) => {
                    let mut s = sh.lock().unwrap();
                    match gcal::parse_events(&text, crate::win::api::tz_hours()) {
                        Ok(ev) => { s.gcal_events = ev; s.gcal_connected = true; s.gcal_status = "Tersinkron".into(); }
                        Err(e) => { s.gcal_connected = true; s.gcal_status = format!("Galat: {e}"); }
                    }
                }
            }
        }
    }
}

/// Alur OAuth loopback + PKCE. Memblokir (jalankan di thread sendiri). Mengembalikan teks status.
fn gcal_connect(data: &Path, cfg: &Settings) -> Result<(), String> {
    if cfg.gcal_client_id.trim().is_empty() { return Err("Isi Client ID di Pengaturan dulu".into()); }
    let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(|e| e.to_string())?;
    listener.set_nonblocking(true).ok();
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let redirect = format!("http://127.0.0.1:{port}");
    let verifier = gcal::pkce_verifier();
    let state = gcal::b64url(&gcal::random_bytes(12));
    let url = gcal::auth_url(cfg.gcal_client_id.trim(), &redirect, &gcal::pkce_challenge(&verifier), &state);
    crate::win::api::shell_open(&url);
    let deadline = Instant::now() + Duration::from_secs(180);
    let (mut conn, line) = loop {
        match listener.accept() {
            Ok((mut c, _)) => {
                c.set_nonblocking(false).ok();
                c.set_read_timeout(Some(Duration::from_secs(5))).ok();
                let mut l = String::new();
                if BufReader::new(&mut c).read_line(&mut l).is_ok() && l.contains("code=") || l.contains("error=") { break (c, l); }
                let _ = c.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
            }
            Err(_) => { if Instant::now() > deadline { return Err("Waktu habis menunggu izin Google".into()); } std::thread::sleep(Duration::from_millis(200)); }
        }
    };
    let result = gcal::parse_redirect(&line);
    let page = |msg: &str| format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\n\r\n<html><body style=\"font-family:Segoe UI,sans-serif;text-align:center;padding-top:80px\"><h2>{msg}</h2><p>Kamu bisa menutup tab ini dan kembali ke Gravi Island.</p></body></html>");
    let (code, st) = match result { Ok(x) => x, Err(e) => { let _ = conn.write_all(page("Izin ditolak").as_bytes()); return Err(format!("Ditolak: {e}")); } };
    if st != state { let _ = conn.write_all(page("Permintaan tidak valid").as_bytes()); return Err("State tidak cocok".into()); }
    let body = gcal::exchange_body(cfg.gcal_client_id.trim(), cfg.gcal_client_secret.trim(), &code, &verifier, &redirect);
    let (_, text) = net::send(&Req::post(gcal::TOKEN_URL, &body).header("Content-Type: application/x-www-form-urlencoded")).ok_or("Tidak ada koneksi")?;
    match gcal::parse_token(&text) {
        Ok(t) => {
            save_tokens(data, &Tokens { access: t.access, refresh: t.refresh.unwrap_or_default(), exp: now_s() + t.expires_in });
            let _ = conn.write_all(page("Berhasil terhubung ✓").as_bytes());
            Ok(())
        }
        Err(e) => { let _ = conn.write_all(page("Gagal").as_bytes()); Err(e) }
    }
}

fn fetch_github(cfg: &Settings, sh: &Handle) {
    let user = cfg.gh_user.trim().to_string();
    if !github::valid_user(&user) { let mut s = sh.lock().unwrap(); s.gh = None; s.gh_status = String::new(); return; }
    let tz = crate::win::api::tz_hours();
    let now = now_s();
    let today = local_day(now, tz);
    let auth = if cfg.gh_token.trim().is_empty() { None } else { Some(format!("Authorization: Bearer {}", cfg.gh_token.trim())) };

    let mut heat = None; let mut repos = vec![]; let mut login = user.clone();
    if let Some(a) = &auth {
        let body = github::graphql_body();
        if let Some((200, t)) = net::send(&Req::post(github::GRAPHQL_URL, &body).header(a.clone()).header("Content-Type: application/json")) {
            if let Some((l, h, r)) = github::parse_graphql(&t) { login = l; heat = Some(h); repos = r; }
        }
    }
    if heat.is_none() { if let Some(t) = net::get_text(&github::contrib_url(&user)) { heat = Some(github::parse_contrib_html(&t)); } }

    let url = github::events_url(&login);
    let mut ev_req = Req::get(&url).header("Accept: application/vnd.github+json");
    if let Some(a) = &auth { ev_req = ev_req.header(a.clone()); }
    let ev = net::send(&ev_req);
    let (commits, ev_repos) = match &ev { Some((200, t)) => github::parse_events(t, tz, today), _ => (vec![], vec![]) };
    if repos.is_empty() { repos = ev_repos; }
    let status = match (&heat, &ev) {
        (None, None) => "Tidak ada koneksi".to_string(),
        (_, Some((403, _))) | (_, Some((429, _))) => "Batas API GitHub tercapai".to_string(),
        (_, Some((404, _))) => "User tidak ditemukan".to_string(),
        _ => "Diperbarui".to_string(),
    };
    let snap = github::snapshot(&login, heat.unwrap_or_default(), commits, repos, tz, now);
    let mut s = sh.lock().unwrap();
    s.gh = Some(snap); s.gh_status = status;
}

fn fetch_weather_and_ics(cfg: &Settings, sh: &Handle) {
    let w = net::get_text(&weather::url(cfg.lat, cfg.lon)).and_then(|b| weather::parse(&b)).map(|(t, d)| format!("{:.0}°C {}", t, d));
    let ev = if cfg.ical.starts_with("http") { net::get_text(&cfg.ical).map(|b| ics::upcoming(&b, now_s(), 7 * 86400, crate::win::api::tz_hours())) } else { None };
    let mut s = sh.lock().unwrap();
    s.offline = w.is_none();
    if let Some(w) = w { s.weather = w; }
    if let Some(ev) = ev { s.ics_events = ev; }
}

/// Mulai thread latar. Mengembalikan handle data + pengirim "dorongan" untuk menyegarkan manual.
pub fn start(cfg: Cfg, data: PathBuf) -> Online {
    let shared: Handle = Arc::new(Mutex::new(Shared::default()));
    let (tx, rx) = mpsc::channel::<Kick>();
    let sh = shared.clone();
    let have_token = load_tokens(&data).refresh.len() > 0;
    sh.lock().unwrap().gcal_status = if have_token { "Menyinkronkan…".into() } else { "Belum terhubung".into() };
    std::thread::spawn(move || {
        let mut last = [Instant::now() - Duration::from_secs(3600); 3]; // cuaca, kalender, github
        let every = [Duration::from_secs(900), Duration::from_secs(900), Duration::from_secs(600)];
        let mut forced = [true, true, true];
        loop {
            let c = cfg.lock().unwrap().clone();
            if forced[0] || last[0].elapsed() >= every[0] { fetch_weather_and_ics(&c, &sh); last[0] = Instant::now(); forced[0] = false; }
            if forced[1] || last[1].elapsed() >= every[1] { if load_tokens(&data).refresh.len() > 0 { fetch_gcal(&data, &c, &sh); } last[1] = Instant::now(); forced[1] = false; }
            if forced[2] || last[2].elapsed() >= every[2] { fetch_github(&c, &sh); last[2] = Instant::now(); forced[2] = false; }
            match rx.recv_timeout(Duration::from_secs(30)) {
                Ok(Kick::Weather) => forced[0] = true,
                Ok(Kick::Calendar) => forced[1] = true,
                Ok(Kick::Github) => { sh.lock().unwrap().gh_status = "Memuat…".into(); forced[2] = true }
                Ok(Kick::GcalDisconnect) => {
                    let _ = std::fs::remove_file(token_path(&data));
                    let mut s = sh.lock().unwrap(); s.gcal_events.clear(); s.gcal_connected = false; s.gcal_status = "Belum terhubung".into();
                }
                Ok(Kick::GcalConnect) => {
                    sh.lock().unwrap().gcal_status = "Menunggu izin di browser…".into();
                    let res = gcal_connect(&data, &c);
                    match res { Ok(()) => { forced[1] = true; } Err(e) => { sh.lock().unwrap().gcal_status = e; } }
                }
                Err(_) => {}
            }
        }
    });
    Online { shared, tx }
}
