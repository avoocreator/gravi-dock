//! Uji ujung-ke-ujung: server TCP palsu berperan sebagai island, biner gravi-hook yang asli dijalankan.
use island_core::hookproto::{Decision, Endpoint, Request};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::process::{Command, Stdio};

fn run_hook(src: &str, stdin: &str, data: &std::path::Path) -> String {
    let mut c = Command::new(env!("CARGO_BIN_EXE_gravi-hook"))
        .arg(src).env("GRAVI_DATA_DIR", data).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    c.stdin.take().unwrap().write_all(stdin.as_bytes()).unwrap();
    String::from_utf8(c.wait_with_output().unwrap().stdout).unwrap()
}

fn fake_island(reply: Decision) -> (std::path::PathBuf, std::thread::JoinHandle<Request>) {
    let dir = std::env::temp_dir().join(format!("gravi-e2e-{}-{:?}", std::process::id(), std::thread::current().id()));
    std::fs::create_dir_all(&dir).unwrap();
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let ep = Endpoint { port: l.local_addr().unwrap().port(), token: "rahasia".into() };
    std::fs::write(dir.join("hook.json"), ep.to_json()).unwrap();
    let h = std::thread::spawn(move || {
        let (mut s, _) = l.accept().unwrap();
        let mut line = String::new();
        BufReader::new(s.try_clone().unwrap()).read_line(&mut line).unwrap();
        writeln!(s, "{}", reply.to_line()).unwrap();
        Request::parse_line(&line).unwrap()
    });
    (dir, h)
}

#[test]
fn claude_allow_roundtrip() {
    let (dir, srv) = fake_island(Decision::Allow);
    let out = run_hook("claude", r#"{"session_id":"s1","cwd":"/p","tool_name":"Bash","tool_input":{"command":"ls -la"}}"#, &dir);
    let req = srv.join().unwrap();
    assert_eq!((req.token.as_str(), req.src.as_str(), req.tool.as_str(), req.summary.as_str()), ("rahasia", "claude", "Bash", "ls -la"));
    assert!(out.contains(r#""behavior":"allow""#) && out.contains("PermissionRequest"), "{out}");
}

#[test]
fn copilot_deny_roundtrip() {
    let (dir, srv) = fake_island(Decision::Deny("jangan".into()));
    let out = run_hook("copilot", r#"{"sessionId":"c","toolName":"powershell","toolArgs":{"command":"git push"}}"#, &dir);
    assert_eq!(srv.join().unwrap().summary, "git push");
    assert!(out.contains(r#""behavior":"deny""#) && out.contains("jangan") && !out.contains("hookSpecificOutput"), "{out}");
}

#[test]
fn ask_and_missing_island_print_nothing() {
    let (dir, srv) = fake_island(Decision::Ask);
    assert_eq!(run_hook("claude", r#"{"tool_name":"Bash","tool_input":{"command":"x"}}"#, &dir).trim(), "");
    srv.join().unwrap();
    let empty = std::env::temp_dir().join("gravi-e2e-kosong");
    let _ = std::fs::remove_dir_all(&empty);
    assert_eq!(run_hook("copilot", "{}", &empty), "", "island tidak berjalan -> diam");
}
