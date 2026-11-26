use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use sysinfo::{ProcessesToUpdate, System};
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSession {
    pub id: String,
    pub name: String,
    pub kind: String,     // "desktop" | "cli"
    pub provider: String, // "claude" | "gemini" | "chatgpt" | "antigravity" | "copilot" | "hermes" | "openbot"
    pub status: String,   // "active" | "idle" | "waiting"
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
        patterns: &["agy", "agy.exe", "antigravity-cli"],
    },
    TargetRule {
        id: "copilot-cli",
        name: "GitHub Copilot CLI",
        kind: "cli",
        provider: "copilot",
        patterns: &["github-copilot-cli", "copilot-cli", "gh-copilot", "copilot"],
    },
];

pub fn scan_processes(sys: &mut System) -> Vec<AiSession> {
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let mut matched: HashMap<&'static str, AiSession> = HashMap::new();

    for (pid, proc_) in sys.processes() {
        let pname = proc_.name().to_string_lossy().to_lowercase();
        let cmdline = proc_
            .cmd()
            .iter()
            .map(|s| s.to_string_lossy().to_lowercase())
            .collect::<Vec<_>>()
            .join(" ");

        for rule in RULES {
            let is_match = rule.patterns.iter().any(|pat| {
                if rule.kind == "cli" {
                    pname == *pat
                        || pname.starts_with(pat)
                        || cmdline.contains(&format!(" {}", pat))
                        || cmdline.contains("gh copilot")
                } else {
                    pname == *pat || pname.starts_with(pat)
                }
            });

            if is_match {
                if rule.id == "antigravity-desktop" && cmdline.contains("cli") {
                    continue;
                }
                let cpu = proc_.cpu_usage();
                let mem_mb = proc_.memory() / (1024 * 1024);
                let run_time = proc_.run_time();
                let entry = matched.entry(rule.id).or_insert_with(|| AiSession {
                    id: rule.id.to_string(),
                    name: rule.name.to_string(),
                    kind: rule.kind.to_string(),
                    provider: rule.provider.to_string(),
                    status: if cpu > 1.5 { "active".into() } else { "idle".into() },
                    pid: pid.as_u32(),
                    cpu_usage: cpu,
                    memory_mb: mem_mb,
                    detail: if rule.kind == "cli" {
                        "Sesi CLI berjalan".into()
                    } else {
                        "Aplikasi desktop aktif".into()
                    },
                    started_secs: run_time,
                });

                if entry.pid != pid.as_u32() {
                    entry.cpu_usage += cpu;
                    entry.memory_mb += mem_mb;
                    if entry.cpu_usage > 1.5 {
                        entry.status = "active".into();
                    }
                }
                break;
            }
        }
    }

    let mut sessions: Vec<AiSession> = matched.into_values().collect();
    sessions.sort_by(|a, b| a.kind.cmp(&b.kind).then(a.name.cmp(&b.name)));
    sessions
}

pub async fn start_watcher(app: AppHandle, cache: Arc<Mutex<Vec<AiSession>>>) {
    let mut sys = System::new_all();
    loop {
        let sessions = scan_processes(&mut sys);
        {
            let mut guard = cache.lock().await;
            *guard = sessions.clone();
        }
        let _ = app.emit("nobi:ai-sessions", &sessions);
        sleep(Duration::from_secs(3)).await;
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
        let script = format!(
            "$wshell = New-Object -ComObject wscript.shell; $wshell.AppActivate('{}')",
            app_name.replace('\'', "")
        );
        std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
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
