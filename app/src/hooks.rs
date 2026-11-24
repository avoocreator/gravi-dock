//! Server persetujuan (TCP loopback) untuk gravi-hook + pemasangan/pelepasan hook di konfigurasi Claude Code & Copilot CLI.
use island_core::gcal::{b64url, random_bytes};
use island_core::hookproto::{self, Decision, Endpoint, Request};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

pub struct Pending { pub req: Request, pub tx: mpsc::Sender<Decision>, pub at: Instant }

#[derive(Clone)]
pub struct Server { pub pending: Arc<Mutex<Vec<Pending>>>, pub dirty: Arc<AtomicBool>, pub port: u16 }

impl Server {
    /// Jawab permintaan `id`. True bila permintaan itu masih menunggu.
    pub fn answer(&self, id: &str, d: Decision) -> bool {
        let mut g = self.pending.lock().unwrap();
        let Some(i) = g.iter().position(|p| p.req.id == id) else { return false };
        let p = g.remove(i);
        let _ = p.tx.send(d);
        self.dirty.store(true, Ordering::SeqCst);
        true
    }
    pub fn snapshot(&self) -> Vec<Request> { self.pending.lock().unwrap().iter().map(|p| p.req.clone()).collect() }
    pub fn count(&self) -> usize { self.pending.lock().unwrap().len() }
}

fn closed_by_peer(s: &TcpStream) -> bool {
    let _ = s.set_read_timeout(Some(Duration::from_millis(1)));
    let mut b = [0u8; 1];
    matches!(s.peek(&mut b), Ok(0))
}

fn serve(mut s: TcpStream, token: &str, srv: &Server) {
    let _ = s.set_read_timeout(Some(Duration::from_secs(5)));
    let mut line = String::new();
    if BufReader::new(&s).read_line(&mut line).is_err() { return; }
    let Some(req) = Request::parse_line(&line) else { return };
    if req.token != token { return; }
    let (tx, rx) = mpsc::channel();
    let id = req.id.clone();
    srv.pending.lock().unwrap().push(Pending { req, tx, at: Instant::now() });
    srv.dirty.store(true, Ordering::SeqCst);
    let deadline = Instant::now() + Duration::from_secs(hookproto::WAIT_SECS);
    let decision = loop {
        match rx.recv_timeout(Duration::from_millis(400)) {
            Ok(d) => break d,
            Err(mpsc::RecvTimeoutError::Disconnected) => break Decision::Ask,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if Instant::now() >= deadline || closed_by_peer(&s) { break Decision::Ask; }
            }
        }
    };
    {
        let mut g = srv.pending.lock().unwrap();
        g.retain(|p| p.req.id != id);
    }
    srv.dirty.store(true, Ordering::SeqCst);
    let _ = writeln!(s, "{}", decision.to_line());
    let _ = s.flush();
}

/// Mulai server di 127.0.0.1 dan tulis `hook.json` (port + token acak) ke folder data.
pub fn start(data: &std::path::Path) -> Option<Server> {
    let listener = TcpListener::bind(("127.0.0.1", hookproto::DEFAULT_PORT)).or_else(|_| TcpListener::bind(("127.0.0.1", 0))).ok()?;
    let port = listener.local_addr().ok()?.port();
    let token = b64url(&random_bytes(24));
    let _ = std::fs::create_dir_all(data);
    std::fs::write(data.join("hook.json"), Endpoint { port, token: token.clone() }.to_json()).ok()?;
    let srv = Server { pending: Arc::new(Mutex::new(vec![])), dirty: Arc::new(AtomicBool::new(false)), port };
    let s2 = srv.clone();
    std::thread::spawn(move || {
        for conn in listener.incoming().flatten() {
            let (t, sv) = (token.clone(), s2.clone());
            std::thread::spawn(move || serve(conn, &t, &sv));
        }
    });
    Some(srv)
}

/* ---------------- pemasangan hook ---------------- */

pub fn hook_exe() -> PathBuf {
    let me = std::env::current_exe().unwrap_or_default();
    me.with_file_name(if cfg!(windows) { "gravi-hook.exe" } else { "gravi-hook" })
}

fn home() -> PathBuf { std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")).map(PathBuf::from).unwrap_or_default() }
pub fn claude_settings_path() -> PathBuf { home().join(".claude").join("settings.json") }
pub fn copilot_hook_path() -> PathBuf {
    let base = std::env::var_os("COPILOT_HOME").map(PathBuf::from).unwrap_or_else(|| home().join(".copilot"));
    base.join("hooks").join("gravi-island.json")
}

pub fn installed(kind: &str) -> bool {
    match kind {
        "claude" => std::fs::read_to_string(claude_settings_path()).map(|s| hookproto::claude_installed(&s)).unwrap_or(false),
        "copilot" => copilot_hook_path().exists(),
        _ => false,
    }
}

pub fn install_at(kind: &str, exe: &str, claude_path: &std::path::Path, copilot_path: &std::path::Path) -> Result<String, String> {
    match kind {
        "claude" => {
            let old = std::fs::read_to_string(claude_path).unwrap_or_default();
            let new = hookproto::merge_claude_settings(&old, exe)?;
            if let Some(d) = claude_path.parent() { let _ = std::fs::create_dir_all(d); }
            let bak = claude_path.with_extension("json.gravi-bak");
            if !old.trim().is_empty() && !bak.exists() { let _ = std::fs::write(&bak, &old); }
            std::fs::write(claude_path, new).map_err(|e| format!("gagal menulis settings.json: {e}"))?;
            Ok("Hook Claude Code terpasang. Mulai ulang sesi Claude Code agar aktif.".into())
        }
        "copilot" => {
            if let Some(d) = copilot_path.parent() { let _ = std::fs::create_dir_all(d); }
            std::fs::write(copilot_path, hookproto::copilot_hook_file(exe)).map_err(|e| format!("gagal menulis hook Copilot: {e}"))?;
            Ok("Hook Copilot CLI terpasang. Mulai ulang Copilot CLI agar aktif.".into())
        }
        _ => Err("jenis tidak dikenal".into()),
    }
}

pub fn remove_at(kind: &str, claude_path: &std::path::Path, copilot_path: &std::path::Path) -> Result<String, String> {
    match kind {
        "claude" => {
            let old = std::fs::read_to_string(claude_path).map_err(|_| "settings.json Claude Code tidak ditemukan".to_string())?;
            let new = hookproto::remove_claude_settings(&old)?;
            std::fs::write(claude_path, new).map_err(|e| format!("gagal menulis settings.json: {e}"))?;
            Ok("Hook Claude Code dilepas.".into())
        }
        "copilot" => { let _ = std::fs::remove_file(copilot_path); Ok("Hook Copilot CLI dilepas.".into()) }
        _ => Err("jenis tidak dikenal".into()),
    }
}

pub fn install(kind: &str) -> Result<String, String> {
    let exe = hook_exe();
    if !exe.exists() { return Err(format!("gravi-hook tidak ditemukan di {}", exe.display())); }
    install_at(kind, &exe.to_string_lossy(), &claude_settings_path(), &copilot_hook_path())
}
pub fn remove(kind: &str) -> Result<String, String> { remove_at(kind, &claude_settings_path(), &copilot_hook_path()) }

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("gravi-hooks-{}-{}", std::process::id(), name));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn server_roundtrip_allow_and_disconnect() {
        let d = tmp("srv");
        let srv = start(&d).expect("server");
        let ep = Endpoint::parse(&std::fs::read_to_string(d.join("hook.json")).unwrap()).unwrap();
        assert_eq!(ep.port, srv.port);
        let ask = |id: &str, token: &str| {
            let mut c = TcpStream::connect(("127.0.0.1", ep.port)).unwrap();
            let r = Request { token: token.into(), src: "claude".into(), id: id.into(), tool: "Bash".into(), summary: "ls".into(), ..Default::default() };
            writeln!(c, "{}", r.to_line()).unwrap();
            c
        };
        // token salah -> diabaikan
        let mut bad = ask("x", "salah");
        let mut l = String::new();
        let _ = bad.set_read_timeout(Some(Duration::from_millis(800)));
        let _ = BufReader::new(&mut bad).read_line(&mut l);
        assert!(l.is_empty() && srv.count() == 0);
        // setujui
        let c = ask("a1", &ep.token);
        for _ in 0..50 { if srv.count() == 1 { break; } std::thread::sleep(Duration::from_millis(20)); }
        assert_eq!(srv.snapshot()[0].summary, "ls");
        assert!(srv.answer("a1", Decision::Allow));
        let mut line = String::new();
        BufReader::new(c).read_line(&mut line).unwrap();
        assert_eq!(Decision::parse_line(&line), Decision::Allow);
        assert!(!srv.answer("a1", Decision::Allow), "sudah dijawab");
        // klien putus (jawab di terminal) -> hilang dari daftar
        let c2 = ask("b2", &ep.token);
        for _ in 0..50 { if srv.count() == 1 { break; } std::thread::sleep(Duration::from_millis(20)); }
        drop(c2);
        for _ in 0..100 { if srv.count() == 0 { break; } std::thread::sleep(Duration::from_millis(40)); }
        assert_eq!(srv.count(), 0);
    }

    #[test]
    fn install_and_remove_files() {
        let d = tmp("inst");
        let (cp, gp) = (d.join(".claude").join("settings.json"), d.join(".copilot").join("hooks").join("gravi-island.json"));
        std::fs::create_dir_all(cp.parent().unwrap()).unwrap();
        std::fs::write(&cp, r#"{"model":"opus"}"#).unwrap();
        install_at("claude", "C:\\G\\gravi-hook.exe", &cp, &gp).unwrap();
        install_at("copilot", "C:\\G\\gravi-hook.exe", &cp, &gp).unwrap();
        let s = std::fs::read_to_string(&cp).unwrap();
        assert!(hookproto::claude_installed(&s) && s.contains("opus"));
        assert!(cp.with_extension("json.gravi-bak").exists(), "cadangan dibuat");
        assert!(gp.exists());
        remove_at("claude", &cp, &gp).unwrap();
        remove_at("copilot", &cp, &gp).unwrap();
        assert!(!hookproto::claude_installed(&std::fs::read_to_string(&cp).unwrap()) && !gp.exists());
        // settings.json rusak tidak boleh ditimpa
        std::fs::write(&cp, "{rusak").unwrap();
        assert!(install_at("claude", "x", &cp, &gp).is_err());
        assert_eq!(std::fs::read_to_string(&cp).unwrap(), "{rusak");
    }
}
