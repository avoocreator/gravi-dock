//! Kontrol media (SMTC) lewat satu proses PowerShell yang hidup hanya saat dibutuhkan (hemat memori).
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

#[derive(Clone, Default, Debug, PartialEq)]
pub struct State { pub active: bool, pub playing: bool, pub title: String, pub artist: String, pub source: String, pub pos_s: f64, pub dur_s: f64 }

pub fn parse_line(line: &str) -> Option<State> {
    let v = island_core::json::parse(line)?;
    Some(State {
        active: v.get("a").truthy(), playing: v.get("playing").truthy(), title: v.get("title").text().to_string(), artist: v.get("artist").text().to_string(),
        source: friendly_source(v.get("src").text()), pos_s: v.get("pos").num().unwrap_or(0.0), dur_s: v.get("dur").num().unwrap_or(0.0),
    })
}

/// "Spotify.exe" / "chrome.exe" / "Microsoft.ZuneMusic_8wekyb3d8bbwe!Microsoft.ZuneMusic" -> nama pendek.
pub fn friendly_source(id: &str) -> String {
    let l = id.to_lowercase();
    for (k, n) in [("spotify", "Spotify"), ("chrome", "Chrome"), ("msedge", "Edge"), ("firefox", "Firefox"), ("brave", "Brave"), ("zune", "Media Player"), ("vlc", "VLC"), ("applemusic", "Apple Music"), ("youtube", "YouTube Music")] {
        if l.contains(k) { return n.to_string(); }
    }
    id.trim_end_matches(".exe").split(['!', '_']).next().unwrap_or("").to_string()
}

pub fn fmt_pos(pos: f64, dur: f64) -> String {
    let f = |s: f64| { let s = s.max(0.0) as i64; format!("{}:{:02}", s / 60, s % 60) };
    if dur > 0.0 { format!("{} / {}", f(pos), f(dur)) } else { f(pos) }
}

pub struct Media { child: Option<Child>, pub state: Arc<Mutex<State>>, script: std::path::PathBuf }

impl Media {
    pub fn new(dir: &std::path::Path) -> Self {
        let script = dir.join("media.ps1");
        let _ = std::fs::create_dir_all(dir);
        let _ = std::fs::write(&script, include_str!("media.ps1"));
        Media { child: None, state: Arc::new(Mutex::new(State::default())), script }
    }
    pub fn running(&self) -> bool { self.child.is_some() }
    pub fn start(&mut self) {
        if self.child.is_some() || !cfg!(windows) { return; }
        let mut c = Command::new("powershell.exe");
        c.args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File"]).arg(&self.script)
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null());
        #[cfg(windows)]
        { use std::os::windows::process::CommandExt; c.creation_flags(0x0800_0000); }
        let Ok(mut ch) = c.spawn() else { return };
        if let Some(out) = ch.stdout.take() {
            let st = self.state.clone();
            std::thread::spawn(move || {
                for line in BufReader::new(out).lines().map_while(Result::ok) {
                    if let Some(s) = parse_line(&line) { *st.lock().unwrap() = s; }
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
    pub fn snapshot(&self) -> State { self.state.lock().unwrap().clone() }
}
impl Drop for Media { fn drop(&mut self) { self.stop(); } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_helper_output() {
        let s = parse_line(r#"{"a":true,"playing":true,"title":"Lagu","artist":"A","src":"Spotify.exe","pos":61.5,"dur":200}"#).unwrap();
        assert_eq!((s.title.as_str(), s.source.as_str(), s.pos_s, s.dur_s), ("Lagu", "Spotify", 61.5, 200.0));
        assert_eq!(fmt_pos(61.5, 200.0), "1:01 / 3:20");
        assert_eq!(fmt_pos(5.0, 0.0), "0:05");
        assert!(parse_line("bukan json").is_none());
        assert_eq!(friendly_source("Microsoft.ZuneMusic_8wekyb3d8bbwe!Microsoft.ZuneMusic"), "Media Player");
        assert_eq!(friendly_source("Foo.exe"), "Foo");
    }
}
