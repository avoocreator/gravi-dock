//! Protokol persetujuan permission dari island untuk CLI AI (Claude Code, Copilot CLI).
//!
//! Alur: CLI menjalankan `gravi-hook.exe <claude|copilot>` pada event permission -> hook membaca JSON dari stdin,
//! mengirim satu baris JSON ke island lewat TCP loopback, menunggu keputusan (Setujui / Tolak / Serahkan ke terminal),
//! lalu mencetak JSON keluaran yang sesuai format masing-masing CLI. Bila island tidak berjalan, hook diam saja (exit 0)
//! sehingga CLI memakai alur izin normalnya.
use crate::json::{self, Value};

pub const DEFAULT_PORT: u16 = 47831;
/// Hook menunggu keputusan selama ini; setelahnya diserahkan kembali ke terminal.
pub const WAIT_SECS: u64 = 110;
/// Batas waktu yang ditulis ke konfigurasi CLI (harus lebih besar dari WAIT_SECS).
pub const CLI_TIMEOUT_SECS: u64 = 120;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Src { Claude, Copilot }

impl Src {
    pub fn parse(s: &str) -> Option<Src> { match s { "claude" => Some(Src::Claude), "copilot" => Some(Src::Copilot), _ => None } }
    pub fn as_str(self) -> &'static str { match self { Src::Claude => "claude", Src::Copilot => "copilot" } }
}

/// Folder data Gravi: `GRAVI_DATA_DIR`, atau `%LOCALAPPDATA%\\GraviIsland`, atau `~/.gravi-island`.
pub fn data_dir() -> std::path::PathBuf {
    use std::path::PathBuf;
    if let Some(d) = std::env::var_os("GRAVI_DATA_DIR") { return PathBuf::from(d); }
    if let Some(l) = std::env::var_os("LOCALAPPDATA") { return PathBuf::from(l).join("GraviIsland"); }
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".gravi-island")).unwrap_or_else(|| PathBuf::from(".gravi-island"))
}

/// Alamat + rahasia island (ditulis island ke `hook.json` di folder data; dibaca hook).
#[derive(Clone, Debug, PartialEq)]
pub struct Endpoint { pub port: u16, pub token: String }

impl Endpoint {
    pub fn to_json(&self) -> String { Value::obj(vec![("port", Value::n(self.port as f64)), ("token", Value::s(&self.token))]).to_json() }
    pub fn parse(src: &str) -> Option<Endpoint> {
        let v = json::parse(src)?;
        Some(Endpoint { port: v.get("port").num()? as u16, token: v.get("token").str()?.to_string() })
    }
}

/// Permintaan yang dikirim hook ke island (satu baris JSON).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Request { pub token: String, pub src: String, pub id: String, pub session: String, pub tool: String, pub summary: String, pub cwd: String }

impl Request {
    pub fn to_line(&self) -> String {
        Value::obj(vec![("v", Value::n(1.0)), ("token", Value::s(&self.token)), ("src", Value::s(&self.src)), ("id", Value::s(&self.id)),
            ("session", Value::s(&self.session)), ("tool", Value::s(&self.tool)), ("summary", Value::s(&self.summary)), ("cwd", Value::s(&self.cwd))]).to_json()
    }
    pub fn parse_line(line: &str) -> Option<Request> {
        let v = json::parse(line.trim())?;
        if v.get("v").num() != Some(1.0) { return None; }
        let t = |k: &str| v.get(k).text().to_string();
        Some(Request { token: t("token"), src: t("src"), id: t("id"), session: t("session"), tool: t("tool"), summary: t("summary"), cwd: t("cwd") })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Decision { Allow, Deny(String), Ask }

impl Decision {
    pub fn to_line(&self) -> String {
        let (d, m) = match self { Decision::Allow => ("allow", ""), Decision::Deny(m) => ("deny", m.as_str()), Decision::Ask => ("ask", "") };
        Value::obj(vec![("decision", Value::s(d)), ("message", Value::s(m))]).to_json()
    }
    pub fn parse_line(line: &str) -> Decision {
        let Some(v) = json::parse(line.trim()) else { return Decision::Ask };
        match v.get("decision").text() { "allow" => Decision::Allow, "deny" => Decision::Deny(v.get("message").text().to_string()), _ => Decision::Ask }
    }
}

/// Teks stdout yang diharapkan CLI. Kosong = tidak ada keputusan (CLI memakai alur izin normal).
pub fn render_output(src: Src, d: &Decision) -> String {
    match (src, d) {
        (_, Decision::Ask) => String::new(),
        (Src::Claude, Decision::Allow) => r#"{"hookSpecificOutput":{"hookEventName":"PermissionRequest","decision":{"behavior":"allow"}}}"#.to_string(),
        (Src::Claude, Decision::Deny(m)) => Value::obj(vec![("hookSpecificOutput", Value::obj(vec![("hookEventName", Value::s("PermissionRequest")),
            ("decision", Value::obj(vec![("behavior", Value::s("deny")), ("message", Value::s(&deny_msg(m)))]))]))]).to_json(),
        (Src::Copilot, Decision::Allow) => r#"{"behavior":"allow"}"#.to_string(),
        (Src::Copilot, Decision::Deny(m)) => Value::obj(vec![("behavior", Value::s("deny")), ("message", Value::s(&deny_msg(m))), ("interrupt", Value::b(false))]).to_json(),
    }
}

fn deny_msg(m: &str) -> String { if m.is_empty() { "Ditolak dari Gravi Island".to_string() } else { m.to_string() } }

/// Isi permintaan hasil parse stdin CLI.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct HookInput { pub session: String, pub tool: String, pub summary: String, pub cwd: String }

/// Parser toleran: mendukung nama field camelCase (`toolName`, `toolInput`/`toolArgs`) dan snake_case (`tool_name`, `tool_input`).
pub fn parse_input(stdin: &str) -> HookInput {
    let Some(v) = json::parse(stdin) else { return HookInput::default() };
    let pick = |a: &str, b: &str| { let x = v.get(a).text(); if x.is_empty() { v.get(b).text() } else { x }.to_string() };
    let tool = pick("tool_name", "toolName");
    let raw = [v.get("tool_input"), v.get("toolInput"), v.get("toolArgs")].into_iter().find(|x| !x.is_null()).cloned().unwrap_or(Value::Null);
    // Copilot kadang memberi argumen sebagai string JSON.
    let input = match &raw { Value::Str(s) => json::parse(s).unwrap_or(raw.clone()), _ => raw };
    HookInput { session: pick("session_id", "sessionId"), summary: summarize(&tool, &input), tool, cwd: v.get("cwd").text().to_string() }
}

/// Ringkasan satu baris untuk ditampilkan di kartu persetujuan.
pub fn summarize(tool: &str, input: &Value) -> String {
    let first = |keys: &[&str]| keys.iter().map(|k| input.get(k).text()).find(|s| !s.is_empty()).map(String::from);
    let s = match tool.to_ascii_lowercase().as_str() {
        "bash" | "shell" | "powershell" | "run_command" | "terminal" => first(&["command", "cmd", "script"]),
        "edit" | "write" | "multiedit" | "read" | "create" | "view" | "str_replace_editor" | "notebookedit" => first(&["file_path", "path", "notebook_path"]),
        "webfetch" | "websearch" | "fetch" => first(&["url", "query"]),
        "task" | "agent" => first(&["description", "prompt"]),
        _ => None,
    }.or_else(|| match input { Value::Obj(kv) => kv.iter().find_map(|(_, v)| v.str().filter(|s| !s.is_empty()).map(String::from)), Value::Str(s) => Some(s.clone()), _ => None });
    let line = s.unwrap_or_default().split_whitespace().collect::<Vec<_>>().join(" ");
    if line.chars().count() > 180 { format!("{}…", line.chars().take(180).collect::<String>()) } else { line }
}

/* ---------------- pemasangan hook ke konfigurasi CLI ---------------- */

const MARK: &str = "gravi-hook";

/// Perintah yang ditulis ke ~/.claude/settings.json.
pub fn claude_command(exe: &str) -> String { format!("\"{exe}\" claude") }

fn is_ours(entry: &Value) -> bool { entry.get("hooks").arr().iter().any(|h| h.get("command").text().contains(MARK)) }

/// Gabungkan hook PermissionRequest ke isi settings.json Claude Code, tanpa menyentuh pengaturan lain.
pub fn merge_claude_settings(existing: &str, exe: &str) -> Result<String, String> {
    let mut root = if existing.trim().is_empty() { Value::Obj(vec![]) } else { json::parse(existing).ok_or("settings.json Claude Code bukan JSON valid; tidak diubah")? };
    if !matches!(root, Value::Obj(_)) { return Err("settings.json Claude Code bukan objek".into()); }
    if root.get("hooks").is_null() { root.set("hooks", Value::Obj(vec![])); }
    let hooks = root.get_mut("hooks").ok_or("hooks tidak valid")?;
    if hooks.get("PermissionRequest").is_null() { hooks.set("PermissionRequest", Value::Arr(vec![])); }
    let Some(Value::Arr(list)) = hooks.get_mut("PermissionRequest") else { return Err("hooks.PermissionRequest bukan larik".into()) };
    list.retain(|e| !is_ours(e));
    list.push(Value::obj(vec![("hooks", Value::Arr(vec![Value::obj(vec![
        ("type", Value::s("command")), ("command", Value::s(&claude_command(exe))), ("timeout", Value::n(CLI_TIMEOUT_SECS as f64))])]))]));
    Ok(root.pretty())
}

/// Hapus hook Gravi dari settings.json Claude Code.
pub fn remove_claude_settings(existing: &str) -> Result<String, String> {
    let mut root = json::parse(existing).ok_or("settings.json Claude Code bukan JSON valid; tidak diubah")?;
    if let Some(hooks) = root.get_mut("hooks") {
        if let Some(Value::Arr(list)) = hooks.get_mut("PermissionRequest") { list.retain(|e| !is_ours(e)); }
    }
    Ok(root.pretty())
}

pub fn claude_installed(existing: &str) -> bool {
    json::parse(existing).map_or(false, |r| r.get("hooks").get("PermissionRequest").arr().iter().any(is_ours))
}

/// Isi berkas ~/.copilot/hooks/gravi-island.json (berkas milik Gravi sendiri, aman ditimpa/dihapus).
pub fn copilot_hook_file(exe: &str) -> String {
    Value::obj(vec![("version", Value::n(1.0)), ("hooks", Value::obj(vec![("permissionRequest", Value::Arr(vec![Value::obj(vec![
        ("type", Value::s("command")), ("exec", Value::s(exe)), ("args", Value::Arr(vec![Value::s("copilot")])), ("timeoutSec", Value::n(CLI_TIMEOUT_SECS as f64))])]))]))]).pretty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_stdin_is_summarized() {
        let i = parse_input(r#"{"session_id":"s1","cwd":"C:\\p","hook_event_name":"PermissionRequest","tool_name":"Bash","tool_input":{"command":"rm  -rf\n build","description":"x"}}"#);
        assert_eq!((i.tool.as_str(), i.session.as_str(), i.summary.as_str(), i.cwd.as_str()), ("Bash", "s1", "rm -rf build", "C:\\p"));
        let e = parse_input(r#"{"tool_name":"Edit","tool_input":{"file_path":"src/a.rs","old_string":"x"}}"#);
        assert_eq!(e.summary, "src/a.rs");
    }
    #[test]
    fn copilot_stdin_camel_case_and_string_args() {
        let i = parse_input(r#"{"sessionId":"c1","cwd":"D:\\r","toolName":"powershell","toolArgs":"{\"command\":\"git push\"}"}"#);
        assert_eq!((i.tool.as_str(), i.summary.as_str(), i.session.as_str()), ("powershell", "git push", "c1"));
        let j = parse_input(r#"{"toolName":"create","toolInput":{"path":"a.txt"}}"#);
        assert_eq!(j.summary, "a.txt");
        assert_eq!(parse_input("bukan json"), HookInput::default());
    }
    #[test]
    fn summary_is_truncated_and_has_fallback() {
        let long = "x".repeat(500);
        let s = summarize("Bash", &json::parse(&format!(r#"{{"command":"{long}"}}"#)).unwrap());
        assert_eq!(s.chars().count(), 181);
        assert_eq!(summarize("Mystery", &json::parse(r#"{"n":1,"foo":"bar"}"#).unwrap()), "bar");
    }
    #[test]
    fn wire_roundtrip() {
        let r = Request { token: "t".into(), src: "claude".into(), id: "7".into(), session: "s".into(), tool: "Bash".into(), summary: "ls \"x\"".into(), cwd: "C:\\".into() };
        assert_eq!(Request::parse_line(&r.to_line()), Some(r));
        assert_eq!(Request::parse_line("{\"v\":2}"), None);
        for d in [Decision::Allow, Decision::Deny("tidak".into()), Decision::Ask] { assert_eq!(Decision::parse_line(&d.to_line()), d); }
        assert_eq!(Decision::parse_line("sampah"), Decision::Ask);
        let ep = Endpoint { port: 47831, token: "abc".into() };
        assert_eq!(Endpoint::parse(&ep.to_json()), Some(ep));
    }
    #[test]
    fn outputs_match_each_cli_format() {
        let c = render_output(Src::Claude, &Decision::Allow);
        let v = json::parse(&c).unwrap();
        assert_eq!(v.get("hookSpecificOutput").get("hookEventName").text(), "PermissionRequest");
        assert_eq!(v.get("hookSpecificOutput").get("decision").get("behavior").text(), "allow");
        let d = json::parse(&render_output(Src::Claude, &Decision::Deny("jangan".into()))).unwrap();
        assert_eq!(d.get("hookSpecificOutput").get("decision").get("message").text(), "jangan");
        let p = json::parse(&render_output(Src::Copilot, &Decision::Deny(String::new()))).unwrap();
        assert_eq!((p.get("behavior").text(), p.get("message").text().is_empty()), ("deny", false));
        assert_eq!(p.get("interrupt").truthy(), false);
        assert_eq!(render_output(Src::Copilot, &Decision::Allow), r#"{"behavior":"allow"}"#);
        assert_eq!(render_output(Src::Claude, &Decision::Ask), "");
    }
    #[test]
    fn claude_settings_merge_is_idempotent_and_preserves_others() {
        let existing = r#"{"model":"opus","hooks":{"PreToolUse":[{"matcher":"Bash","hooks":[{"type":"command","command":"x"}]}],"PermissionRequest":[{"hooks":[{"type":"command","command":"other.exe"}]}]}}"#;
        let once = merge_claude_settings(existing, "C:\\Gravi\\gravi-hook.exe").unwrap();
        let twice = merge_claude_settings(&once, "C:\\Gravi\\gravi-hook.exe").unwrap();
        assert_eq!(once, twice);
        let v = json::parse(&once).unwrap();
        assert_eq!(v.get("model").text(), "opus");
        assert_eq!(v.get("hooks").get("PreToolUse").arr().len(), 1);
        assert_eq!(v.get("hooks").get("PermissionRequest").arr().len(), 2);
        assert!(claude_installed(&once));
        let removed = remove_claude_settings(&once).unwrap();
        assert!(!claude_installed(&removed));
        assert_eq!(json::parse(&removed).unwrap().get("hooks").get("PermissionRequest").arr().len(), 1);
        assert!(merge_claude_settings("{rusak", "x").is_err(), "file rusak tidak boleh ditimpa");
        assert!(merge_claude_settings("", "x").is_ok());
    }
    #[test]
    fn copilot_file_shape() {
        let v = json::parse(&copilot_hook_file("C:\\Program Files\\Gravi\\gravi-hook.exe")).unwrap();
        let h = &v.get("hooks").get("permissionRequest").arr()[0];
        assert_eq!(v.get("version").num(), Some(1.0));
        assert_eq!((h.get("type").text(), h.get("args").arr()[0].text()), ("command", "copilot"));
        assert!(h.get("exec").text().contains("gravi-hook"));
        assert!(h.get("timeoutSec").num().unwrap() as u64 > WAIT_SECS);
    }
}
