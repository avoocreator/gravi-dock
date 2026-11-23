//! Kontrol media (SMTC) lewat satu proses PowerShell yang hidup hanya saat dibutuhkan.
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct State { pub active: bool, pub playing: bool, pub title: String, pub artist: String }

pub struct Media { child: Option<Child>, pub state: Arc<Mutex<State>>, script: std::path::PathBuf }

impl Media {
    pub fn new(dir: &std::path::Path) -> Self {
        let script = dir.join("media.ps1");
        let _ = std::fs::write(&script, include_str!("media.ps1"));
        Media { child: None, state: Arc::new(Mutex::new(State::default())), script }
    }
    pub fn running(&self) -> bool { self.child.is_some() }
    pub fn start(&mut self) {
        if self.child.is_some() { return; }
        let mut c = Command::new("powershell.exe");
        c.args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File"]).arg(&self.script)
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null());
        #[cfg(windows)]
        { use std::os::windows::process::CommandExt; c.creation_flags(0x08000000); }
        let Ok(mut ch) = c.spawn() else { return };
        if let Some(out) = ch.stdout.take() {
            let st = self.state.clone();
            std::thread::spawn(move || {
                for line in BufReader::new(out).lines().map_while(Result::ok) {
                    if let Some(v) = island_core::json::parse(&line) {
                        *st.lock().unwrap() = State { active: v.get("a").truthy(), playing: v.get("playing").truthy(), title: v.get("title").text().to_string(), artist: v.get("artist").text().to_string() };
                    }
                }
            });
        }
        self.child = Some(ch);
    }
    pub fn stop(&mut self) {
        if let Some(mut c) = self.child.take() { let _ = c.kill(); let _ = c.wait(); }
        *self.state.lock().unwrap() = State::default();
    }
    pub fn send(&mut self, cmd: &str) {
        if let Some(c) = self.child.as_mut() { if let Some(i) = c.stdin.as_mut() { let _ = writeln!(i, "{{\"cmd\":\"{}\"}}", cmd); let _ = i.flush(); } }
    }
}
impl Drop for Media { fn drop(&mut self) { self.stop(); } }
