use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::{ProcessesToUpdate, System};
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

const MID_TURN_MS: i64 = 60_000;
const ACTIVE_TURN_MS: i64 = 25_000;
const CONCURRENT_TURN_MS: i64 = 10_000;
const QUIET_RUN_MS: i64 = 10_000;
const TAIL_BYTES: u64 = 64 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSession {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub provider: String,
    pub status: String,
    pub pid: u32,
    pub cpu_usage: f32,
    pub memory_mb: u64,
    pub detail: String,
    pub started_secs: u64,
}

struct TargetRule {
    id: &'static str,
    name: &'static str,
    kind: &'static str,
    provider: &'static str,
    patterns: &'static [&'static str],
}

const RULES: &[TargetRule] = &[
    TargetRule {
        id: "claude-desktop",
        name: "Claude",
        kind: "desktop",
        provider: "claude",
        patterns: &["claude.exe", "claude-desktop", "claude"],
    },
    TargetRule {
        id: "gemini-desktop",
        name: "Gemini",
        kind: "desktop",
        provider: "gemini",
        patterns: &["gemini-desktop", "gemini.exe"],
    },
    TargetRule {
        id: "chatgpt-desktop",
        name: "ChatGPT",
        kind: "desktop",
        provider: "chatgpt",
        patterns: &["chatgpt.exe", "chatgpt"],
    },
    TargetRule {
        id: "antigravity-desktop",
        name: "Antigravity",
        kind: "desktop",
        provider: "antigravity",
        patterns: &["antigravity.exe", "antigravity-desktop", "antigravity"],
    },
    TargetRule {
        id: "hermes-desktop",
        name: "Hermes",
        kind: "desktop",
        provider: "hermes",
        patterns: &["hermes.exe", "hermes-desktop", "hermes"],
    },
    TargetRule {
        id: "openbot-desktop",
        name: "OpenBot",
        kind: "desktop",
        provider: "openbot",
        patterns: &["openbot.exe", "openbot"],
    },
    TargetRule {
        id: "antigravity-cli",
        name: "Antigravity CLI",
        kind: "cli",
        provider: "antigravity",
        patterns: &["agy", "agy.exe", "antigravity-cli", "antigravity-cli.exe"],
    },
    TargetRule {
        id: "copilot-cli",
        name: "GitHub Copilot CLI",
        kind: "cli",
        provider: "copilot",
        patterns: &[
            "github-copilot-cli",
            "github-copilot-cli.exe",
            "copilot-cli",
            "copilot-cli.exe",
            "gh-copilot",
            "gh-copilot.exe",
        ],
    },
];

fn has_token(cmdline: &str, token: &str) -> bool {
    cmdline.split_whitespace().any(|w| w == token)
}

fn is_noise_subprocess(cmdline: &str, exe_path: &str) -> bool {
    cmdline.contains("crashpad")
        || cmdline.contains("crash_reporter")
        || cmdline.contains("crashreporter")
        || cmdline.contains("sidecars/")
        || cmdline.contains("sidecars\\")
        || cmdline.contains("sidecar_sdk")
        || cmdline.contains("--import=")
        || cmdline.contains("--type=utility")
        || cmdline.contains("--type=zygote")
        || cmdline.contains("--type=crashpad-handler")
        || cmdline.contains("--type=broker")
        || cmdline.contains("--headless")
        || exe_path.contains("crashpad")
        || exe_path.contains("launcher")
        || exe_path.contains("updater")
        || exe_path.contains("uninstall")
}

fn is_electron_child(cmdline: &str) -> bool {
    cmdline.contains("--type=")
}

fn is_ui_process(cmdline: &str) -> bool {
    cmdline.contains("--type=gpu-process") || cmdline.contains("--type=renderer")
}

fn is_cli_binary_path(exe_path: &str) -> bool {
    exe_path.contains("\\.local\\bin\\")
        || exe_path.contains("/.local/bin/")
        || exe_path.contains("\\.claude\\")
        || exe_path.contains("/.claude/")
        || exe_path.contains("\\hermes\\bin\\")
        || exe_path.contains("/hermes/bin/")
        || exe_path.contains("\\npm\\")
        || exe_path.contains("/npm/")
}

fn is_utility_cli_invocation(cmdline: &str) -> bool {
    let mut args = cmdline.split_whitespace().skip(1);
    let Some(first) = args.next() else {
        return false;
    };
    matches!(
        first,
        "--version"
            | "-v"
            | "--help"
            | "-h"
            | "help"
            | "version"
            | "status"
            | "auth"
            | "login"
            | "logout"
            | "mcp"
            | "serve"
            | "update"
            | "upgrade"
            | "doctor"
            | "config"
            | "completion"
    )
}

fn is_antigravity_ide_app_data(cmdline: &str) -> bool {
    has_token(cmdline, "--app_data_dir=antigravity")
        || cmdline.contains("--app_data_dir antigravity ")
        || cmdline.ends_with("--app_data_dir antigravity")
}

fn matches_rule(rule: &TargetRule, pname: &str, cmdline: &str, exe_path: &str) -> bool {
    if is_noise_subprocess(cmdline, exe_path) {
        return false;
    }

    if rule.id == "antigravity-cli" {
        if !rule.patterns.iter().any(|pat| pname == *pat) {
            return false;
        }
        if cmdline.trim().is_empty()
            || cmdline.contains("--hub")
            || cmdline.contains("--hub-port")
            || is_antigravity_ide_app_data(cmdline)
            || is_utility_cli_invocation(cmdline)
        {
            return false;
        }
        let from_ide_bin =
            exe_path.contains("\\.gemini\\bin\\") || exe_path.contains("/.gemini/bin/");
        if from_ide_bin && !cmdline.contains("antigravity-cli") && !has_token(cmdline, "chat") {
            return false;
        }
        return true;
    }

    if rule.id == "copilot-cli" {
        if is_utility_cli_invocation(cmdline) {
            return false;
        }
        if rule.patterns.iter().any(|pat| pname == *pat) {
            return true;
        }
        if (pname == "gh" || pname == "gh.exe") && has_token(cmdline, "copilot") {
            return true;
        }
        if (pname == "node" || pname == "node.exe")
            && (cmdline.contains("github-copilot-cli") || cmdline.contains("@github/copilot-cli"))
        {
            return true;
        }
        return false;
    }

    if rule.id == "antigravity-desktop" && (cmdline.contains("--hub") || cmdline.contains("antigravity-cli")) {
        return false;
    }

    if (rule.id == "claude-desktop" || rule.id == "hermes-desktop") && is_cli_binary_path(exe_path)
    {
        return false;
    }

    rule.patterns.iter().any(|pat| pname == *pat)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TranscriptState {
    Idle,
    Active(String),
    Waiting(String),
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn mtime_ms(md: &fs::Metadata) -> i64 {
    md.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn collect_recent_files(
    root: &Path,
    max_depth: usize,
    matcher: &dyn Fn(&str, &Path) -> bool,
    cutoff_ms: i64,
    out: &mut Vec<(PathBuf, i64, u64)>,
    depth: usize,
) {
    let Ok(rd) = fs::read_dir(root) else { return };
    for entry in rd.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let Ok(ft) = entry.file_type() else { continue };
        if ft.is_dir() {
            if depth < max_depth && name != "node_modules" && name != "subagents" {
                collect_recent_files(&path, max_depth, matcher, cutoff_ms, out, depth + 1);
            }
        } else if ft.is_file() && matcher(&name, &path) {
            if let Ok(md) = entry.metadata() {
                let mt = mtime_ms(&md);
                if mt >= cutoff_ms {
                    out.push((path, mt, md.len()));
                }
            }
        }
    }
}

fn read_tail_json_lines(path: &Path, size: u64) -> Vec<serde_json::Value> {
    let Ok(mut f) = fs::File::open(path) else {
        return Vec::new();
    };
    let start = size.saturating_sub(TAIL_BYTES);
    if f.seek(SeekFrom::Start(start)).is_err() {
        return Vec::new();
    }
    let mut buf = Vec::new();
    let _ = f.take(TAIL_BYTES).read_to_end(&mut buf);
    let text = String::from_utf8_lossy(&buf);
    let mut lines = text.split('\n');
    if start > 0 {
        lines.next();
    }
    lines
        .filter_map(|l| {
            let trimmed = l.trim();
            if trimmed.is_empty() {
                None
            } else {
                serde_json::from_str::<serde_json::Value>(trimmed).ok()
            }
        })
        .collect()
}

fn eval_agy_tail(tail: &[serde_json::Value]) -> TranscriptState {
    for ev in tail.iter().rev() {
        let ty = ev["type"].as_str().unwrap_or("");
        if ty.is_empty() {
            continue;
        }
        let status = ev["status"].as_str().unwrap_or("");
        let up = status.to_uppercase();
        let ty_up = ty.to_uppercase();
        if matches!(up.as_str(), "CANCELLED" | "CANCELED" | "FAILED" | "ERROR" | "ABORTED")
            || ty_up.contains("CANCEL")
            || ty_up.contains("ABORT")
        {
            return TranscriptState::Idle;
        }
        if !status.is_empty() && !matches!(up.as_str(), "DONE" | "COMPLETE" | "COMPLETED" | "SUCCESS")
        {
            let waiting = ["WAIT", "BLOCK", "APPROV", "PERMISSION", "CONFIRM", "PENDING_USER", "NEED"]
                .iter()
                .any(|k| up.contains(k));
            return if waiting {
                TranscriptState::Waiting("Menunggu persetujuanmu".into())
            } else {
                TranscriptState::Active("Bekerja…".into())
            };
        }
        if ty == "PLANNER_RESPONSE" {
            let calls = ev["tool_calls"].as_array();
            if let Some(first_call) = calls.and_then(|c| c.first()) {
                let tool = first_call["name"]
                    .as_str()
                    .unwrap_or("tool")
                    .replace('_', " ");
                return TranscriptState::Active(tool);
            }
            return TranscriptState::Idle;
        }
        if ty == "USER_INPUT" && ev["source"].as_str().unwrap_or("") != "SYSTEM" {
            return TranscriptState::Active("Berpikir…".into());
        }
    }
    TranscriptState::Idle
}

fn eval_agy_roots(roots: &[PathBuf], now: i64) -> TranscriptState {
    let mut files = Vec::new();
    let matcher = |n: &str, p: &Path| {
        n == "transcript.jsonl"
            && p.parent()
                .and_then(|x| x.file_name())
                .map(|x| x == "logs")
                .unwrap_or(false)
    };
    for r in roots {
        if r.exists() {
            collect_recent_files(r, 4, &matcher, now - MID_TURN_MS, &mut files, 0);
        }
    }
    files.sort_by(|a, b| b.1.cmp(&a.1));
    for (idx, (path, mt, size)) in files.into_iter().take(3).enumerate() {
        let age_ms = now.saturating_sub(mt);
        if idx > 0 && age_ms > CONCURRENT_TURN_MS {
            break;
        }
        let tail = read_tail_json_lines(&path, size);
        match eval_agy_tail(&tail) {
            TranscriptState::Idle => continue,
            TranscriptState::Waiting(msg) => return TranscriptState::Waiting(msg),
            TranscriptState::Active(msg) => {
                if age_ms <= ACTIVE_TURN_MS {
                    return TranscriptState::Active(msg);
                }
            }
        }
    }
    TranscriptState::Idle
}

fn eval_copilot_roots(root: &Path, now: i64) -> TranscriptState {
    if !root.exists() {
        return TranscriptState::Idle;
    }
    let mut files = Vec::new();
    let matcher = |n: &str, _: &Path| {
        n == "events.jsonl" || (n.starts_with("inuse.") && (n.ends_with(".lock") || n.ends_with(".hold")))
    };
    collect_recent_files(root, 2, &matcher, now - MID_TURN_MS, &mut files, 0);
    files.sort_by(|a, b| b.1.cmp(&a.1));

    for (path, _, size) in files.into_iter().take(4) {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.starts_with("inuse.") {
            return TranscriptState::Active("Bekerja…".into());
        }
        let tail = read_tail_json_lines(&path, size);
        for ev in tail.iter().rev() {
            let ty = ev["type"].as_str().unwrap_or("").to_lowercase();
            if ty.is_empty() {
                continue;
            }
            if ["turn_end", "session.idle", "session.shutdown", "task_complete", "abort"]
                .iter()
                .any(|k| ty.contains(k))
            {
                break;
            }
            if ["turn_start", "user.", "tool", "assistant."]
                .iter()
                .any(|k| ty.contains(k))
            {
                return TranscriptState::Active("Bekerja…".into());
            }
        }
    }
    TranscriptState::Idle
}

fn has_recent_activity(roots: &[PathBuf], max_depth: usize, matcher: &dyn Fn(&str, &Path) -> bool, window_ms: i64, now: i64) -> bool {
    let mut files = Vec::new();
    for r in roots {
        if r.exists() {
            collect_recent_files(r, max_depth, matcher, now - window_ms, &mut files, 0);
        }
    }
    !files.is_empty()
}

fn detect_transcript_states(now: i64) -> HashMap<&'static str, TranscriptState> {
    let mut map = HashMap::new();
    let Some(home) = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
    else {
        return map;
    };
    let roam = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join("AppData").join("Roaming"));
    let local = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join("AppData").join("Local"));

    let agy_desk_roots = vec![
        home.join(".gemini").join("antigravity").join("brain"),
        home.join(".gemini").join("antigravity-ide").join("brain"),
    ];
    map.insert("antigravity-desktop", eval_agy_roots(&agy_desk_roots, now));

    let agy_cli_roots = vec![home.join(".gemini").join("antigravity-cli").join("brain")];
    map.insert("antigravity-cli", eval_agy_roots(&agy_cli_roots, now));

    let copilot_root = home.join(".copilot").join("session-state");
    map.insert("copilot-cli", eval_copilot_roots(&copilot_root, now));

    let claude_roots = vec![
        roam.join("Claude").join("local-agent-mode-sessions"),
        local.join("Claude").join("local-agent-mode-sessions"),
    ];
    let claude_active = has_recent_activity(
        &claude_roots,
        5,
        &|n, _| n == "audit.jsonl" || n.ends_with(".jsonl"),
        QUIET_RUN_MS,
        now,
    );
    map.insert(
        "claude-desktop",
        if claude_active {
            TranscriptState::Active("Bekerja…".into())
        } else {
            TranscriptState::Idle
        },
    );

    let codex_roots = vec![home.join(".codex").join("thread-writer-locks")];
    let chatgpt_active = has_recent_activity(
        &codex_roots,
        2,
        &|n, _| n.ends_with(".lock") && n != ".coordination.lock",
        MID_TURN_MS,
        now,
    );
    map.insert(
        "chatgpt-desktop",
        if chatgpt_active {
            TranscriptState::Active("Menulis…".into())
        } else {
            TranscriptState::Idle
        },
    );

    let hermes_roots = vec![local.join("hermes").join("sessions")];
    let hermes_active = has_recent_activity(
        &hermes_roots,
        1,
        &|n, _| n.starts_with("request_dump_") && n.ends_with(".json"),
        QUIET_RUN_MS,
        now,
    );
    map.insert(
        "hermes-desktop",
        if hermes_active {
            TranscriptState::Active("Bekerja…".into())
        } else {
            TranscriptState::Idle
        },
    );

    map
}

#[cfg(target_os = "windows")]
fn collect_visible_window_pids() -> HashSet<u32> {
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextLengthW, GetWindowThreadProcessId, IsWindowVisible,
    };

    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        if IsWindowVisible(hwnd).as_bool() && GetWindowTextLengthW(hwnd) > 0 {
            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid != 0 {
                let set = &mut *(lparam.0 as *mut HashSet<u32>);
                set.insert(pid);
            }
        }
        BOOL(1)
    }

    let mut pids = HashSet::new();
    unsafe {
        let _ = EnumWindows(Some(enum_proc), LPARAM(&mut pids as *mut _ as isize));
    }
    pids
}

#[cfg(not(target_os = "windows"))]
fn collect_visible_window_pids() -> HashSet<u32> {
    HashSet::new()
}

struct MatchedAccum {
    session: AiSession,
    has_main_process: bool,
    has_visible_window: bool,
}

fn scan_processes_detailed(
    sys: &mut System,
    transcripts: &HashMap<&'static str, TranscriptState>,
) -> (Vec<AiSession>, HashSet<String>) {
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let cores = sys.cpus().len().max(1) as f32;
    let visible_pids = collect_visible_window_pids();
    let check_windows = cfg!(target_os = "windows") && !visible_pids.is_empty();
    let mut matched: HashMap<&'static str, MatchedAccum> = HashMap::new();

    for (pid, proc_) in sys.processes() {
        let pname = proc_.name().to_string_lossy().to_lowercase();
        let cmdline = proc_
            .cmd()
            .iter()
            .map(|s| s.to_string_lossy().to_lowercase())
            .collect::<Vec<_>>()
            .join(" ");
        let exe_path = proc_
            .exe()
            .map(|p| p.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        for rule in RULES {
            if matches_rule(rule, &pname, &cmdline, &exe_path) {
                let is_child = is_electron_child(&cmdline);
                let cpu = if is_ui_process(&cmdline) {
                    0.0
                } else {
                    proc_.cpu_usage() / cores
                };
                let mem_mb = proc_.memory() / (1024 * 1024);
                let run_time = proc_.run_time();
                let pid_u32 = pid.as_u32();
                let win_visible = visible_pids.contains(&pid_u32);

                let entry = matched.entry(rule.id).or_insert_with(|| MatchedAccum {
                    session: AiSession {
                        id: rule.id.to_string(),
                        name: rule.name.to_string(),
                        kind: rule.kind.to_string(),
                        provider: rule.provider.to_string(),
                        status: "idle".into(),
                        pid: pid_u32,
                        cpu_usage: 0.0,
                        memory_mb: 0,
                        detail: if rule.kind == "cli" {
                            "Sesi CLI terbuka".into()
                        } else {
                            "Aplikasi desktop terbuka".into()
                        },
                        started_secs: run_time,
                    },
                    has_main_process: false,
                    has_visible_window: false,
                });

                if !is_child {
                    entry.has_main_process = true;
                    entry.session.pid = pid_u32;
                }
                if win_visible {
                    entry.has_visible_window = true;
                }
                entry.session.cpu_usage += cpu;
                entry.session.memory_mb += mem_mb;
                break;
            }
        }
    }

    let mut visible_sessions = HashSet::new();
    let mut sessions: Vec<AiSession> = matched
        .into_iter()
        .filter_map(|(id, acc)| {
            if acc.session.kind == "desktop" {
                if !acc.has_main_process {
                    return None;
                }
                let has_live_transcript = matches!(
                    transcripts.get(id),
                    Some(TranscriptState::Active(_) | TranscriptState::Waiting(_))
                );
                if check_windows && !acc.has_visible_window && !has_live_transcript {
                    return None;
                }
            }
            if acc.has_visible_window {
                visible_sessions.insert(id.to_string());
            }
            Some(acc.session)
        })
        .collect();
    sessions.sort_by(|a, b| a.kind.cmp(&b.kind).then(a.name.cmp(&b.name)));
    (sessions, visible_sessions)
}

#[derive(Default)]
struct ActivityTracker {
    active_streak: HashMap<String, u8>,
    idle_streak: HashMap<String, u8>,
    is_active: HashMap<String, bool>,
}

impl ActivityTracker {
    fn apply(
        &mut self,
        sessions: &mut [AiSession],
        transcripts: &HashMap<&'static str, TranscriptState>,
        visible_sessions: &HashSet<String>,
    ) {
        let mut seen = HashSet::with_capacity(sessions.len());
        for s in sessions.iter_mut() {
            seen.insert(s.id.clone());

            if let Some(tstate) = transcripts.get(s.id.as_str()) {
                match tstate {
                    TranscriptState::Waiting(msg) => {
                        s.status = "waiting".into();
                        s.detail = msg.clone();
                    }
                    TranscriptState::Active(msg) => {
                        s.status = "active".into();
                        s.detail = msg.clone();
                    }
                    TranscriptState::Idle => {
                        s.status = "idle".into();
                        s.detail = if s.kind == "cli" {
                            "Sesi CLI terbuka".into()
                        } else {
                            "Aplikasi desktop terbuka".into()
                        };
                    }
                }
                continue;
            }

            if cfg!(target_os = "windows")
                && s.kind == "desktop"
                && !visible_sessions.is_empty()
                && !visible_sessions.contains(&s.id)
            {
                self.is_active.insert(s.id.clone(), false);
                self.active_streak.insert(s.id.clone(), 0);
                s.status = "idle".into();
                s.detail = "Aplikasi desktop terbuka".into();
                continue;
            }

            let (enter_thresh, exit_thresh, required_ticks) = if s.kind == "cli" {
                (2.0_f32, 0.5_f32, 2_u8)
            } else {
                (12.0_f32, 2.5_f32, 3_u8)
            };

            let currently_active = *self.is_active.get(&s.id).unwrap_or(&false);
            if currently_active {
                if s.cpu_usage < exit_thresh {
                    let count = self.idle_streak.entry(s.id.clone()).or_insert(0);
                    *count = count.saturating_add(1);
                    if *count >= 2 {
                        self.is_active.insert(s.id.clone(), false);
                        *count = 0;
                    }
                } else {
                    self.idle_streak.insert(s.id.clone(), 0);
                }
            } else if s.cpu_usage >= enter_thresh {
                let count = self.active_streak.entry(s.id.clone()).or_insert(0);
                *count = count.saturating_add(1);
                if *count >= required_ticks {
                    self.is_active.insert(s.id.clone(), true);
                    *count = 0;
                }
            } else {
                self.active_streak.insert(s.id.clone(), 0);
            }

            let active = *self.is_active.get(&s.id).unwrap_or(&false);
            s.status = if active { "active".into() } else { "idle".into() };
            if active {
                s.detail = if s.kind == "cli" {
                    "Sesi CLI aktif memproses".into()
                } else {
                    "Aplikasi desktop aktif".into()
                };
            }
        }

        self.is_active.retain(|k, _| seen.contains(k));
        self.active_streak.retain(|k, _| seen.contains(k));
        self.idle_streak.retain(|k, _| seen.contains(k));
    }
}

pub async fn start_watcher(app: AppHandle, cache: Arc<Mutex<Vec<AiSession>>>) {
    let mut sys = System::new_all();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    sleep(Duration::from_millis(500)).await;

    let mut tracker = ActivityTracker::default();
    let mut prev_statuses: HashMap<String, String> = HashMap::new();
    loop {
        let transcripts = detect_transcript_states(now_ms());
        let (mut sessions, visible_sessions) = scan_processes_detailed(&mut sys, &transcripts);
        tracker.apply(&mut sessions, &transcripts, &visible_sessions);

        let cfg = crate::settings::get_settings(app.clone());
        if cfg.hud_enabled && cfg.hud_ai_done {
            for s in &sessions {
                if !cfg.monitored_ais.contains(&s.id) {
                    continue;
                }
                if let Some(prev_st) = prev_statuses.get(&s.id) {
                    if (prev_st == "active" || prev_st == "waiting") && s.status == "idle" {
                        let _ = app.emit(
                            "nobi:hud-event",
                            crate::system_hud::HudEventPayload {
                                kind: "ai_done".into(),
                                title: format!("{} Selesai", s.name),
                                subtitle: "Sesi AI selesai — klik untuk fokus".into(),
                                mood: "celebrate".into(),
                                accent: "emerald".into(),
                                badge: Some("Selesai".into()),
                                provider: Some(s.provider.clone()),
                                app_name: Some(s.name.clone()),
                            },
                        );
                    }
                }
            }
        }
        prev_statuses = sessions
            .iter()
            .map(|s| (s.id.clone(), s.status.clone()))
            .collect();

        {
            let mut guard = cache.lock().await;
            *guard = sessions.clone();
        }
        let _ = app.emit("nobi:ai-sessions", &sessions);
        let interval = crate::settings::get_settings(app.clone()).poll_interval_secs.clamp(1, 30) as u64;
        sleep(Duration::from_secs(interval)).await;
    }
}

#[tauri::command]
pub async fn get_ai_sessions(
    state: tauri::State<'_, crate::AppState>,
) -> Result<Vec<AiSession>, String> {
    let guard = state.monitor_cache.lock().await;
    Ok(guard.clone())
}

#[tauri::command]
pub fn focus_ai_window(app_name: String) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let script = format!(
            "$wshell = New-Object -ComObject wscript.shell; $wshell.AppActivate('{}')",
            app_name.replace('\'', "")
        );
        std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(format!("Focusing {}", app_name));
    }

    #[cfg(target_os = "linux")]
    {
        let status = std::process::Command::new("wmctrl")
            .args(["-a", &app_name])
            .status();
        if status.is_ok() {
            return Ok(format!("Focusing {}", app_name));
        }
        let _ = std::process::Command::new("xdotool")
            .args(["search", "--name", &app_name, "windowactivate"])
            .spawn();
        return Ok(format!("Focusing {}", app_name));
    }

    #[allow(unreachable_code)]
    Ok(format!("Requested focus for {}", app_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule_by_id(id: &str) -> &'static TargetRule {
        RULES.iter().find(|r| r.id == id).unwrap()
    }

    fn dummy_session(id: &str, kind: &str, cpu_usage: f32) -> AiSession {
        AiSession {
            id: id.into(),
            name: id.into(),
            kind: kind.into(),
            provider: "antigravity".into(),
            status: "idle".into(),
            pid: 1,
            cpu_usage,
            memory_mb: 100,
            detail: String::new(),
            started_secs: 10,
        }
    }

    #[test]
    fn ignores_vscode_copilot_runtime_and_agy_hub() {
        let copilot_cli = rule_by_id("copilot-cli");
        assert!(!matches_rule(
            copilot_cli,
            "copilot-runtime.exe",
            r#""c:\program files\microsoft vs code\copilot-runtime.exe""#,
            r"c:\program files\microsoft vs code\copilot-runtime.exe"
        ));
        assert!(matches_rule(
            copilot_cli,
            "gh.exe",
            "gh.exe copilot suggest",
            r"c:\program files\github cli\gh.exe"
        ));

        let agy_cli = rule_by_id("antigravity-cli");
        assert!(!matches_rule(
            agy_cli,
            "agy.exe",
            r"c:\users\kira\.gemini\bin\agy.exe --hub --hub-port=56544",
            r"c:\users\kira\.gemini\bin\agy.exe"
        ));
        assert!(!matches_rule(
            agy_cli,
            "agy.exe",
            "",
            r"c:\users\kira\.gemini\bin\agy.exe"
        ));
        assert!(!matches_rule(
            agy_cli,
            "agy.exe",
            r"c:\users\kira\.gemini\bin\agy.exe --app_data_dir=antigravity",
            r"c:\users\kira\.gemini\bin\agy.exe"
        ));
        assert!(!matches_rule(
            agy_cli,
            "agy.exe",
            r"c:\users\kira\appdata\local\agy\bin\agy.exe --version",
            r"c:\users\kira\appdata\local\agy\bin\agy.exe"
        ));
        assert!(matches_rule(
            agy_cli,
            "agy.exe",
            r"c:\users\kira\.gemini\bin\agy.exe chat",
            r"c:\users\kira\.gemini\bin\agy.exe"
        ));
        assert!(matches_rule(
            agy_cli,
            "agy.exe",
            r"c:\users\kira\appdata\local\agy\bin\agy.exe",
            r"c:\users\kira\appdata\local\agy\bin\agy.exe"
        ));

        let agy_desktop = rule_by_id("antigravity-desktop");
        assert!(!matches_rule(
            agy_desktop,
            "antigravity.exe",
            "antigravity.exe sidecars/biomolecular_structure_viewer/dist/mcp_entry.cjs",
            r"c:\users\kira\appdata\local\programs\antigravity\antigravity.exe"
        ));
        assert!(!matches_rule(
            agy_desktop,
            "antigravity.exe",
            "antigravity.exe --import=file:///c:/sidecar_sdk/register_loader.js dist/server.cjs",
            r"c:\users\kira\appdata\local\programs\antigravity\antigravity.exe"
        ));

        let claude_desktop = rule_by_id("claude-desktop");
        assert!(!matches_rule(
            claude_desktop,
            "claude.exe",
            "claude.exe",
            r"c:\users\kira\.local\bin\claude.exe"
        ));
        assert!(matches_rule(
            claude_desktop,
            "claude.exe",
            "claude.exe",
            r"c:\users\kira\appdata\local\anthropicclaude\claude.exe"
        ));

        let hermes_desktop = rule_by_id("hermes-desktop");
        assert!(!matches_rule(
            hermes_desktop,
            "hermes.exe",
            "hermes.exe",
            r"c:\users\kira\appdata\local\hermes\bin\hermes.exe"
        ));
    }

    #[test]
    fn transcript_overrides_cpu_spikes_and_hysteresis_handles_fallback() {
        let mut tracker = ActivityTracker::default();
        let mut transcripts = HashMap::new();
        transcripts.insert("antigravity-desktop", TranscriptState::Idle);
        let mut visible = HashSet::new();
        visible.insert("antigravity-desktop".to_string());
        visible.insert("openbot-desktop".to_string());

        for _ in 0..2 {
            let mut tick = vec![
                dummy_session("antigravity-desktop", "desktop", 18.0),
                dummy_session("openbot-desktop", "desktop", 14.0),
                dummy_session("gemini-desktop", "desktop", 18.0),
            ];
            tracker.apply(&mut tick, &transcripts, &visible);
            assert_eq!(tick[0].status, "idle");
            assert_eq!(tick[1].status, "idle");
            assert_eq!(tick[2].status, "idle");
        }

        let mut tick3 = vec![
            dummy_session("antigravity-desktop", "desktop", 18.0),
            dummy_session("openbot-desktop", "desktop", 13.5),
            dummy_session("gemini-desktop", "desktop", 18.0),
        ];
        tracker.apply(&mut tick3, &transcripts, &visible);
        assert_eq!(tick3[0].status, "idle");
        assert_eq!(tick3[1].status, "active");
        if cfg!(target_os = "windows") {
            assert_eq!(tick3[2].status, "idle");
        }

        let done_tail = vec![
            serde_json::json!({"type":"USER_INPUT","status":"DONE","content":"hi"}),
            serde_json::json!({"type":"PLANNER_RESPONSE","status":"DONE","tool_calls":[]}),
            serde_json::json!({"type":"GENERIC","status":"DONE"}),
        ];
        assert_eq!(eval_agy_tail(&done_tail), TranscriptState::Idle);

        let cancelled_tail = vec![
            serde_json::json!({"type":"USER_INPUT","status":"CANCELLED","content":"hi"}),
        ];
        assert_eq!(eval_agy_tail(&cancelled_tail), TranscriptState::Idle);

        let run_tail = vec![
            serde_json::json!({"type":"PLANNER_RESPONSE","status":"DONE","tool_calls":[{"name":"run_command"}]}),
        ];
        assert_eq!(
            eval_agy_tail(&run_tail),
            TranscriptState::Active("run command".into())
        );
    }
}
