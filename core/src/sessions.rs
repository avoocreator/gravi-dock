//! Pelacak sesi AI dari berkas riwayat lokal (hanya-baca): Claude Code, Copilot CLI, Antigravity CLI, Claude Desktop.
//! "Berjalan" = berkas ditulis baru-baru ini dan kejadian terakhirnya belum menandai giliran selesai.
use crate::json::{self, Value};
use std::collections::HashSet;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const RECENT_MS: i64 = 36 * 3600 * 1000;
const KEEP_IDLE_MS: i64 = 24 * 3600 * 1000;
const MID_TURN_MS: i64 = 45_000; // giliran belum selesai tapi berkas diam selama ini -> berhenti
const QUIET_RUN_MS: i64 = 6_000; // format tak dikenal: berjalan bila ditulis dalam 6 dtk terakhir
const MAX_PER_PROVIDER: usize = 30;
const HEAD: u64 = 64 * 1024;
const TAIL: u64 = 96 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State { Running, NeedsAction, Idle }

#[derive(Clone, Debug)]
pub struct Session {
    pub id: String,
    pub provider: &'static str,
    pub title: String,
    pub state: State,
    pub updated_ms: i64,
    pub cwd: String,
    pub current: String,
}

#[derive(Default, Clone, Debug)]
struct Parsed { final_: Option<bool>, title: String, cwd: String, current: String, action: bool }

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind { Claude, Copilot, AgyTranscript, AgyLight, Desk }

struct Provider { id: &'static str, kind: Kind, roots: Vec<PathBuf>, depth: usize }

pub fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

fn clean(s: &str, n: usize) -> String {
    let joined = s.split_whitespace().collect::<Vec<_>>().join(" ");
    joined.chars().take(n).collect()
}
fn short(p: &str) -> String {
    let parts: Vec<&str> = p.split(['/', '\\']).filter(|x| !x.is_empty()).collect();
    parts[parts.len().saturating_sub(2)..].join("/")
}
fn tool_label(name: &str, input: &Value) -> String {
    let pick = |k: &str| input.get(k).text().to_string();
    let target = [pick("file_path"), pick("path"), pick("filePath"), pick("pattern"), pick("command"), pick("url"), pick("description")]
        .into_iter().find(|s| !s.is_empty()).unwrap_or_default();
    let t = if target.is_empty() { String::new() } else if !pick("command").is_empty() && target == pick("command") { format!(" {}", clean(&target, 44)) } else { format!(" {}", short(&target)) };
    clean(&format!("{name}{t}"), 56)
}
fn text_of(c: &Value) -> String {
    match c {
        Value::Str(s) => s.clone(),
        Value::Arr(a) => a.iter().find(|b| b.get("type").text() == "text").map(|b| b.get("text").text().to_string()).unwrap_or_default(),
        _ => String::new(),
    }
}
fn usable(t: &str) -> bool {
    let tt = t.trim_start();
    !t.trim().is_empty() && t.trim().chars().count() > 1 && !tt.starts_with('<') && !tt.starts_with("[Request interrupted") && !tt.starts_with("[Image")
}

/* ---------------------------------------------------------------- berkas */

fn walk(root: &Path, max_depth: usize, matcher: &dyn Fn(&str, &Path) -> bool, cutoff_ms: i64, out: &mut Vec<(PathBuf, i64, u64)>, depth: usize) {
    let Ok(rd) = fs::read_dir(root) else { return };
    for e in rd.flatten() {
        let p = e.path();
        let name = e.file_name().to_string_lossy().into_owned();
        let Ok(ft) = e.file_type() else { continue };
        if ft.is_dir() {
            if depth < max_depth && name != "node_modules" && name != "subagents" { walk(&p, max_depth, matcher, cutoff_ms, out, depth + 1); }
        } else if ft.is_file() && matcher(&name, &p) {
            if let Ok(md) = e.metadata() {
                let m = mtime_ms(&md);
                if m >= cutoff_ms { out.push((p, m, md.len())); }
            }
        }
    }
}
fn mtime_ms(md: &fs::Metadata) -> i64 {
    md.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_millis() as i64).unwrap_or(0)
}
fn read_slice(path: &Path, start: u64, len: u64) -> String {
    let Ok(mut f) = fs::File::open(path) else { return String::new() };
    if f.seek(SeekFrom::Start(start)).is_err() { return String::new(); }
    let mut buf = Vec::new();
    let _ = f.take(len).read_to_end(&mut buf);
    String::from_utf8_lossy(&buf).into_owned()
}
fn parse_lines(text: &str, drop_first: bool) -> Vec<Value> {
    let mut lines = text.split('\n');
    if drop_first { lines.next(); }
    lines.filter_map(|l| { let t = l.trim(); if t.is_empty() { None } else { json::parse(t) } }).collect()
}
fn head_tail(path: &Path, size: u64) -> (Vec<Value>, Vec<Value>) {
    let head = parse_lines(&read_slice(path, 0, HEAD.min(size)), false);
    let tail = if size > TAIL { parse_lines(&read_slice(path, size - TAIL, TAIL), true) } else { Vec::new() };
    let tail = if tail.is_empty() { head.clone() } else { tail };
    (head, tail)
}

/* ---------------------------------------------------------------- parser */

fn parse_claude(head: &[Value], tail: &[Value]) -> Parsed {
    let (mut title, mut cwd, mut first, mut custom) = (String::new(), String::new(), String::new(), String::new());
    for e in head.iter().chain(tail.iter()) {
        if cwd.is_empty() { cwd = e.get("cwd").text().to_string(); }
        match e.get("type").text() {
            "custom-title" => { let c = e.get("customTitle").text(); custom = if c.is_empty() { e.get("title").text().to_string() } else { c.to_string() }; }
            "ai-title" => { let c = e.get("aiTitle").text(); title = if c.is_empty() { e.get("title").text().to_string() } else { c.to_string() }; }
            "summary" if title.is_empty() => title = e.get("summary").text().to_string(),
            "user" if first.is_empty() && !e.get("isMeta").truthy() && !e.get("isSidechain").truthy() => {
                let t = text_of(e.get("message").get("content"));
                if usable(&t) { first = t; }
            }
            _ => {}
        }
    }
    let (mut final_, mut current) = (None, String::new());
    for e in tail.iter().rev() {
        if e.get("isSidechain").truthy() { continue; }
        match e.get("type").text() {
            "assistant" if !e.get("message").is_null() => {
                let m = e.get("message");
                let sr = m.get("stop_reason").text();
                if matches!(sr, "end_turn" | "stop_sequence" | "max_tokens") { final_ = Some(true); }
                else {
                    final_ = Some(false);
                    let tu = m.get("content").arr().iter().rev().find(|b| b.get("type").text() == "tool_use");
                    current = match tu { Some(b) => tool_label(b.get("name").text(), b.get("input")), None => "Menulis jawaban…".into() };
                }
                break;
            }
            "user" if !e.get("isMeta").truthy() => {
                let txt = text_of(e.get("message").get("content"));
                if txt.trim_start().starts_with("[Request interrupted") {
                    final_ = Some(true);
                } else {
                    final_ = Some(false);
                    current = "Berpikir…".into();
                }
                break;
            }
            _ => {}
        }
    }
    let t = if !custom.is_empty() { custom } else if !title.is_empty() { title } else { first };
    Parsed { final_, title: clean(&t, 80), cwd, current, action: false }
}

fn parse_copilot(head: &[Value], tail: &[Value], dir: &Path) -> Parsed {
    let (mut first, mut cwd) = (String::new(), String::new());
    for e in head {
        let d = if e.get("data").is_null() { e } else { e.get("data") };
        if cwd.is_empty() {
            for k in ["cwd", "workingDirectory"] { if cwd.is_empty() { cwd = d.get(k).text().to_string(); } }
        }
        let ty = e.get("type").text().to_lowercase();
        if first.is_empty() && ty.contains("user") && !ty.contains("tool") {
            for k in ["content", "text", "prompt", "message"] { if first.is_empty() { first = d.get(k).text().to_string(); } }
        }
    }
    let mut title = String::new();
    if let Ok(y) = fs::read_to_string(dir.join("workspace.yaml")) {
        for l in y.lines() {
            for k in ["name:", "summary:", "title:"] {
                if title.is_empty() && l.starts_with(k) { title = l[k.len()..].trim().trim_matches(|c| c == '"' || c == '\'').to_string(); }
            }
            if cwd.is_empty() { for k in ["cwd:", "git_root:", "workspace:"] { if l.starts_with(k) { cwd = l[k.len()..].trim().to_string(); } } }
        }
    }
    let (mut final_, mut current) = (None, String::new());
    for e in tail.iter().rev() {
        let ty = e.get("type").text().to_lowercase();
        if ty.is_empty() { continue; }
        if ["turn_end", "session.idle", "session.shutdown", "task_complete", "abort"].iter().any(|k| ty.contains(k)) { final_ = Some(true); break; }
        if ["turn_start", "user.", "tool", "assistant."].iter().any(|k| ty.contains(k)) {
            final_ = Some(false);
            let d = if e.get("data").is_null() { e } else { e.get("data") };
            let tn = [d.get("toolName").text(), d.get("tool").text(), d.get("name").text()].into_iter().find(|s| !s.is_empty()).unwrap_or("");
            current = if ty.contains("tool") && !tn.is_empty() { clean(tn, 40) } else { "Bekerja…".into() };
            break;
        }
    }
    Parsed { final_, title: clean(if title.is_empty() { &first } else { &title }, 80), cwd, current, action: false }
}

fn parse_agy(head: &[Value], tail: &[Value]) -> Parsed {
    let first = head.iter().find(|e| e.get("type").text() == "USER_INPUT" && !e.get("content").text().trim().is_empty())
        .map(|e| strip_tags(e.get("content").text())).unwrap_or_default();
    let (mut final_, mut current, mut action) = (None, String::new(), false);
    for e in tail.iter().rev() {
        let ty = e.get("type").text();
        if ty.is_empty() { continue; }
        let st = e.get("status").text();
        let up = st.to_uppercase();
        if ["ERROR", "FAIL", "CANCEL", "ABORT", "INTERRUPT"].iter().any(|k| up.contains(k)) {
            final_ = Some(true);
            break;
        }
        if !st.is_empty() && !matches!(up.as_str(), "DONE" | "COMPLETE" | "COMPLETED" | "SUCCESS") {
            final_ = Some(false);
            action = ["WAIT", "BLOCK", "APPROV", "PERMISSION", "CONFIRM", "PENDING_USER", "NEED"].iter().any(|k| up.contains(k));
            current = if action { "Menunggu persetujuanmu".into() } else { "Bekerja…".into() };
            break;
        }
        let calls = e.get("tool_calls").arr();
        if ty == "PLANNER_RESPONSE" && calls.is_empty() { final_ = Some(true); break; }
        final_ = Some(false);
        current = if let Some(c) = calls.first() { clean(&c.get("name").text().replace('_', " "), 40) } else if ty == "USER_INPUT" { "Berpikir…".into() } else { "Bekerja…".into() };
        break;
    }
    Parsed { final_, title: clean(&first, 80), cwd: String::new(), current, action }
}
fn strip_tags(s: &str) -> String {
    let (mut out, mut inside) = (String::new(), false);
    for c in s.chars() { match c { '<' => inside = true, '>' => { inside = false; out.push(' '); } _ if !inside => out.push(c), _ => {} } }
    out
}

fn parse_desk(file: &Path, size: u64, dir: &Path) -> Parsed {
    let base = dir.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let cands = [dir.parent().map(|p| p.join(format!("{base}.json"))), Some(dir.join("session.json"))];
    let mut title = String::new();
    for f in cands.into_iter().flatten() {
        if let Ok(t) = fs::read_to_string(&f) {
            if let Some(j) = json::parse(&t) {
                for k in ["title", "name", "summary", "initialMessage", "prompt"] { if title.is_empty() { title = j.get(k).text().to_string(); } }
                if !title.is_empty() { break; }
            }
        }
    }
    let (_, tail) = head_tail(file, size);
    let mut final_ = None;
    for e in tail.iter().rev() {
        let ty = e.get("type").text();
        let sr = e.get("stop_reason").text();
        let msr = e.get("message").get("stop_reason").text();
        if matches!(ty, "result" | "turn_end" | "error")
            || matches!(sr, "end_turn" | "stop_sequence" | "max_tokens")
            || matches!(msr, "end_turn" | "stop_sequence" | "max_tokens")
        {
            final_ = Some(true);
            break;
        }
        if !ty.is_empty() { break; }
    }
    Parsed { final_, title: clean(&title, 80), cwd: String::new(), current: "Bekerja…".into(), action: false }
}

/* ---------------------------------------------------------------- pemindai */

pub struct Scanner { provs: Vec<Provider> }

impl Scanner {
    /// `home` = folder pengguna; `appdata`/`localappdata` = %APPDATA% dan %LOCALAPPDATA% (boleh None).
    pub fn new(home: &Path, appdata: Option<&Path>, localappdata: Option<&Path>, claude_dir: Option<&Path>, copilot_dir: Option<&Path>) -> Self {
        let roam = appdata.map(Path::to_path_buf).unwrap_or_else(|| home.join("AppData").join("Roaming"));
        let local = localappdata.map(Path::to_path_buf).unwrap_or_else(|| home.join("AppData").join("Local"));
        let mut desk = vec![
            roam.join("Claude").join("local-agent-mode-sessions"),
            local.join("Claude").join("local-agent-mode-sessions"),
            local.join("Claude-3p").join("local-agent-mode-sessions"),
        ];
        if let Ok(rd) = fs::read_dir(local.join("Packages")) {
            for e in rd.flatten() {
                let n = e.file_name().to_string_lossy().to_lowercase();
                if n.starts_with("claude") || n.starts_with("anthropic") {
                    desk.push(e.path().join("LocalCache").join("Roaming").join("Claude").join("local-agent-mode-sessions"));
                }
            }
        }
        let claude = claude_dir.map(Path::to_path_buf).unwrap_or_else(|| home.join(".claude"));
        let copilot = copilot_dir.map(Path::to_path_buf).unwrap_or_else(|| home.join(".copilot"));
        let agy = home.join(".gemini").join("antigravity-cli");
        let agy_ide = home.join(".gemini").join("antigravity");
        Scanner { provs: vec![
            Provider { id: "claude", kind: Kind::Claude, roots: vec![claude.join("projects")], depth: 1 },
            Provider { id: "copilot", kind: Kind::Copilot, roots: vec![copilot.join("session-state")], depth: 2 },
            Provider { id: "agy", kind: Kind::AgyTranscript, roots: vec![agy.join("brain"), agy_ide.join("brain")], depth: 4 },
            Provider { id: "agy", kind: Kind::AgyLight, roots: vec![agy.join("conversations"), agy_ide.join("conversations")], depth: 1 },
            Provider { id: "claudedesk", kind: Kind::Desk, roots: desk, depth: 5 },
        ] }
    }

    fn matcher(kind: Kind) -> Box<dyn Fn(&str, &Path) -> bool> {
        match kind {
            Kind::Claude | Kind::Copilot => Box::new(|n, _| n.ends_with(".jsonl")),
            Kind::AgyTranscript => Box::new(|n, p| n == "transcript.jsonl" && p.parent().and_then(|x| x.file_name()).map(|x| x == "logs").unwrap_or(false)),
            Kind::AgyLight => Box::new(|n, _| (n.ends_with(".db") || n.ends_with(".db-wal") || n.ends_with(".pb")) && !n.starts_with("implicit")),
            Kind::Desk => Box::new(|n, _| n == "audit.jsonl"),
        }
    }
    fn id_of(kind: Kind, f: &Path) -> String {
        let stem = |p: &Path| p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        match kind {
            Kind::Claude => f.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default(),
            Kind::Copilot => if stem(f) == "events.jsonl" { f.parent().map(stem).unwrap_or_default() } else { f.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default() },
            // .../brain/<id>/.system_generated/logs/transcript.jsonl
            Kind::AgyTranscript => f.parent().and_then(|p| p.parent()).and_then(|p| p.parent()).map(stem).unwrap_or_default(),
            Kind::AgyLight => { let n = stem(f); for suf in [".db-wal", ".db", ".pb"] { if let Some(x) = n.strip_suffix(suf) { return x.to_string(); } } n }
            Kind::Desk => f.parent().map(stem).unwrap_or_default(),
        }
    }

    /// Daftar sesi (berjalan dulu, lalu terbaru). `enabled(provider_id)` boleh mematikan sumber.
    pub fn scan(&self, now: i64, enabled: &dyn Fn(&str) -> bool) -> Vec<Session> {
        let mut out: Vec<Session> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        for p in &self.provs {
            if !enabled(p.id) { continue; }
            let m = Self::matcher(p.kind);
            let mut files = Vec::new();
            for r in &p.roots { if r.exists() { walk(r, p.depth, &*m, now - RECENT_MS, &mut files, 0); } }
            files.sort_by(|a, b| b.1.cmp(&a.1));
            for (f, mt, size) in files.into_iter().take(MAX_PER_PROVIDER) {
                let id = format!("{}:{}", p.id, Self::id_of(p.kind, &f));
                if !seen.insert(id.clone()) { continue; }
                let dir = f.parent().unwrap_or(Path::new("."));
                let parsed = match p.kind {
                    Kind::Claude => { let (h, t) = head_tail(&f, size); parse_claude(&h, &t) }
                    Kind::Copilot => { let (h, t) = head_tail(&f, size); parse_copilot(&h, &t, dir) }
                    Kind::AgyTranscript => { let (h, t) = head_tail(&f, size); parse_agy(&h, &t) }
                    Kind::AgyLight => Parsed { final_: Some(true), ..Default::default() },
                    Kind::Desk => parse_desk(&f, size, dir),
                };
                let age = now - mt;
                let mut state = State::Idle;
                match parsed.final_ {
                    Some(false) => if age < MID_TURN_MS { state = if parsed.action { State::NeedsAction } else { State::Running }; },
                    Some(true) => {}
                    None => if age < QUIET_RUN_MS { state = State::Running; },
                }
                if state == State::Idle && age > KEEP_IDLE_MS { continue; }
                let mut current = if state == State::Running { parsed.current.clone() } else { String::new() };
                if state == State::Running && parsed.final_ == Some(false) && age > 15_000 && !current.contains("Menulis") && !current.contains("Berpikir") { current.push_str(" · menunggu…"); }
                let fallback = match p.kind { Kind::AgyLight | Kind::AgyTranscript => "Sesi Antigravity CLI", Kind::Desk => "Sesi Claude Desktop", _ => "Sesi tanpa judul" };
                out.push(Session { id, provider: p.id, title: if parsed.title.is_empty() { fallback.into() } else { parsed.title }, state, updated_ms: mt, cwd: parsed.cwd, current });
            }
        }
        out.sort_by(|a, b| (a.state == State::Idle).cmp(&(b.state == State::Idle)).then(b.updated_ms.cmp(&a.updated_ms)));
        out.truncate(60);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);

    fn tmp() -> PathBuf {
        let p = std::env::temp_dir().join(format!("isl-test-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }
    fn write(home: &Path, rel: &str, body: &str, age_ms: u64) {
        let f = home.join(rel);
        fs::create_dir_all(f.parent().unwrap()).unwrap();
        fs::write(&f, body).unwrap();
        let t = SystemTime::now() - std::time::Duration::from_millis(age_ms);
        fs::OpenOptions::new().write(true).open(&f).unwrap().set_modified(t).unwrap();
    }
    fn scan(home: &Path, off: &[&str]) -> Vec<Session> {
        let sc = Scanner::new(home, Some(&home.join("AppData/Roaming")), Some(&home.join("AppData/Local")), None, None);
        let off: Vec<String> = off.iter().map(|s| s.to_string()).collect();
        sc.scan(now_ms(), &|id| !off.iter().any(|o| o == id))
    }
    fn find<'a>(v: &'a [Session], id: &str) -> &'a Session { v.iter().find(|s| s.id == id).unwrap_or_else(|| panic!("tidak ada {id}: {v:?}")) }

    #[test]
    fn claude_code() {
        let h = tmp();
        write(&h, ".claude/projects/p1/run.jsonl", concat!(
            r#"{"type":"user","cwd":"/x/app","message":{"role":"user","content":"Tolong perbaiki bug login"}}"#, "\n",
            r#"{"type":"assistant","message":{"stop_reason":"tool_use","content":[{"type":"tool_use","name":"Edit","input":{"file_path":"/x/app/src/auth.ts"}}]}}"#, "\n"), 2000);
        write(&h, ".claude/projects/p1/done.jsonl", concat!(
            r#"{"type":"summary","summary":"Refactor modul"}"#, "\n",
            r#"{"type":"user","message":{"content":"halo"}}"#, "\n",
            r#"{"type":"assistant","message":{"stop_reason":"end_turn","content":[{"type":"text","text":"ok"}]}}"#, "\n"), 1000);
        write(&h, ".claude/projects/p1/stale.jsonl", "{\"type\":\"user\",\"message\":{\"content\":\"macet\"}}\n", 60_000);
        write(&h, ".claude/projects/p1/interrupted.jsonl", concat!(
            r#"{"type":"user","message":{"role":"user","content":"[Request interrupted by user]"}}"#, "\n"), 1000);
        let r = scan(&h, &[]);
        let run = find(&r, "claude:run");
        assert_eq!(run.state, State::Running);
        assert_eq!(run.title, "Tolong perbaiki bug login");
        assert_eq!(run.current, "Edit src/auth.ts");
        assert_eq!(run.cwd, "/x/app");
        assert_eq!(find(&r, "claude:done").state, State::Idle);
        assert_eq!(find(&r, "claude:done").title, "Refactor modul");
        assert_eq!(find(&r, "claude:stale").state, State::Idle);
        assert_eq!(find(&r, "claude:interrupted").state, State::Idle);
        assert_eq!(r[0].id, "claude:run", "yang berjalan harus di atas");
    }

    #[test]
    fn copilot_cli() {
        let h = tmp();
        write(&h, ".copilot/session-state/abc/events.jsonl", concat!(r#"{"type":"user.message","data":{"content":"tambah tes"}}"#, "\n", r#"{"type":"tool.execution_start","data":{"toolName":"bash"}}"#, "\n"), 1500);
        write(&h, ".copilot/session-state/abc/workspace.yaml", "id: abc\nsummary: Menambah tes unit\n", 1500);
        write(&h, ".copilot/session-state/def/events.jsonl", concat!(r#"{"type":"user.message","data":{"content":"jelaskan kode"}}"#, "\n", r#"{"type":"assistant.turn_end","data":{}}"#, "\n"), 1500);
        let r = scan(&h, &[]);
        assert_eq!(find(&r, "copilot:abc").state, State::Running);
        assert_eq!(find(&r, "copilot:abc").title, "Menambah tes unit");
        assert_eq!(find(&r, "copilot:def").state, State::Idle);
        assert_eq!(find(&r, "copilot:def").title, "jelaskan kode");
    }

    #[test]
    fn antigravity_cli_dan_cadangan() {
        let h = tmp();
        let t = ".gemini/antigravity-cli/brain/3b4a/.system_generated/logs/transcript.jsonl";
        write(&h, t, concat!(r#"{"step_index":0,"type":"USER_INPUT","status":"DONE","content":"Riset algoritma"}"#, "\n", r#"{"step_index":1,"type":"PLANNER_RESPONSE","status":"DONE","tool_calls":[{"name":"search_web"}]}"#, "\n"), 1000);
        write(&h, ".gemini/antigravity/brain/5c5c/.system_generated/logs/transcript.jsonl", concat!(r#"{"step_index":0,"type":"USER_INPUT","status":"DONE","content":"Buka IDE"}"#, "\n", r#"{"step_index":1,"type":"PLANNER_RESPONSE","status":"ERROR","content":"batal"}"#, "\n"), 1000);
        write(&h, ".gemini/antigravity-cli/brain/9f9f/.system_generated/logs/transcript.jsonl", concat!(r#"{"type":"USER_INPUT","status":"DONE","content":"Selesai"}"#, "\n", r#"{"type":"PLANNER_RESPONSE","status":"DONE","content":"jawaban"}"#, "\n"), 1000);
        write(&h, ".gemini/antigravity-cli/conversations/aaaa-1111.db-wal", "x", 2000);
        let r = scan(&h, &[]);
        assert_eq!(find(&r, "agy:3b4a").state, State::Running);
        assert_eq!(find(&r, "agy:3b4a").title, "Riset algoritma");
        assert_eq!(find(&r, "agy:3b4a").current, "search web");
        assert_eq!(find(&r, "agy:5c5c").state, State::Idle);
        assert_eq!(find(&r, "agy:9f9f").state, State::Idle);
        assert_eq!(find(&r, "agy:aaaa-1111").state, State::Idle);
    }

    #[test]
    fn claude_desktop_dan_matikan_sumber() {
        let h = tmp();
        write(&h, "AppData/Roaming/Claude/local-agent-mode-sessions/org/dev/local_1/audit.jsonl", "{\"e\":1}\n", 2000);
        write(&h, "AppData/Roaming/Claude/local-agent-mode-sessions/org/dev/local_1.json", r#"{"title":"Rapikan laporan"}"#, 2000);
        write(&h, "AppData/Roaming/Claude/local-agent-mode-sessions/org/dev/local_2/audit.jsonl", "{\"type\":\"result\",\"stop_reason\":\"end_turn\"}\n", 1000);
        let r = scan(&h, &[]);
        assert_eq!(r.len(), 2);
        assert_eq!(find(&r, "claudedesk:local_1").provider, "claudedesk");
        assert_eq!(find(&r, "claudedesk:local_1").state, State::Running);
        assert_eq!(find(&r, "claudedesk:local_1").title, "Rapikan laporan");
        assert_eq!(find(&r, "claudedesk:local_2").state, State::Idle);
        assert!(scan(&h, &["claudedesk"]).is_empty());
    }

    #[test]
    fn berkas_rusak_tidak_membuat_crash() {
        let h = tmp();
        write(&h, ".claude/projects/p/bad.jsonl", "bukan json\n{potong", 1000);
        write(&h, ".claude/projects/p/empty.jsonl", "", 1000);
        let r = scan(&h, &[]);
        assert_eq!(r.len(), 2);
    }
}
