use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use sysinfo::{ProcessesToUpdate, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedAiProvider {
    pub id: String,
    pub name: String,
    pub installed: bool,
    pub running: bool,
    pub has_account: bool,
    pub source: String,
    pub status_label: String,
    pub models: Vec<String>,
    pub default_model: String,
}

fn command_exists(bin: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        Command::new("where")
            .arg(bin)
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("which")
            .arg(bin)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

fn path_exists_any(paths: &[PathBuf]) -> bool {
    paths.iter().any(|p| p.exists())
}

fn app_installed_paths(app: &str) -> bool {
    let Some(home) = home_dir() else {
        return false;
    };
    let local = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join("AppData").join("Local"));
    let roam = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join("AppData").join("Roaming"));
    let prog = std::env::var_os("ProgramFiles")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("C:\\Program Files"));

    match app {
        "claude" => path_exists_any(&[
            local.join("AnthropicClaude"),
            local.join("Programs").join("claude-desktop"),
            roam.join("Claude"),
            home.join(".claude"),
            PathBuf::from("/Applications/Claude.app"),
        ]),
        "chatgpt" => path_exists_any(&[
            local.join("Packages"),
            roam.join("OpenAI"),
            home.join(".codex"),
            PathBuf::from("/Applications/ChatGPT.app"),
        ]) && (roam.join("OpenAI").exists()
            || home.join(".codex").exists()
            || PathBuf::from("/Applications/ChatGPT.app").exists()),
        "gemini" => path_exists_any(&[
            home.join(".gemini"),
            roam.join("Gemini"),
            local.join("Programs").join("Antigravity"),
            PathBuf::from("/Applications/Antigravity.app"),
        ]),
        "ollama" => path_exists_any(&[
            local.join("Programs").join("Ollama").join("ollama.exe"),
            home.join(".ollama"),
            PathBuf::from("/Applications/Ollama.app"),
            PathBuf::from("/usr/local/bin/ollama"),
            PathBuf::from("/usr/bin/ollama"),
        ]),
        "lmstudio" => path_exists_any(&[
            local.join("LM-Studio"),
            local.join("Programs").join("LM Studio"),
            roam.join("LM Studio"),
            prog.join("LM Studio"),
            home.join(".cache").join("lm-studio"),
            PathBuf::from("/Applications/LM Studio.app"),
        ]),
        _ => false,
    }
}

fn is_process_running(sys: &System, names: &[&str]) -> bool {
    sys.processes().values().any(|p| {
        let pname = p.name().to_string_lossy().to_lowercase();
        names.iter().any(|n| pname == *n || pname.starts_with(n))
    })
}

async fn fetch_ollama_models(client: &reqwest::Client, endpoint: &str) -> Result<Vec<String>, String> {
    let base = endpoint.trim().trim_end_matches('/');
    let url = format!("{}/api/tags", base);
    let resp = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let val: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let mut models = Vec::new();
    if let Some(arr) = val["models"].as_array() {
        for m in arr {
            if let Some(name) = m["name"].as_str() {
                models.push(name.to_string());
            }
        }
    }
    Ok(models)
}

async fn fetch_lmstudio_models(client: &reqwest::Client) -> Result<Vec<String>, String> {
    let resp = client
        .get("http://127.0.0.1:1234/v1/models")
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let val: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let mut models = Vec::new();
    if let Some(arr) = val["data"].as_array() {
        for m in arr {
            if let Some(id) = m["id"].as_str() {
                models.push(id.to_string());
            }
        }
    }
    Ok(models)
}

async fn fetch_openai_models(client: &reqwest::Client, key: &str) -> Result<Vec<String>, String> {
    let resp = client
        .get("https://api.openai.com/v1/models")
        .bearer_auth(key)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let val: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let mut models: Vec<String> = val["data"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
                .filter(|id| {
                    (id.starts_with("gpt-4")
                        || id.starts_with("gpt-5")
                        || id.starts_with("o1")
                        || id.starts_with("o3")
                        || id.starts_with("o4"))
                        && !id.contains("audio")
                        && !id.contains("realtime")
                        && !id.contains("transcribe")
                        && !id.contains("tts")
                        && !id.contains("image")
                })
                .collect()
        })
        .unwrap_or_default();
    models.sort();
    models.dedup();
    models.reverse();
    models.truncate(20);
    Ok(models)
}

async fn fetch_anthropic_models(client: &reqwest::Client, key: &str) -> Result<Vec<String>, String> {
    let resp = client
        .get("https://api.anthropic.com/v1/models")
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let val: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let mut models: Vec<String> = val["data"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    models.truncate(20);
    Ok(models)
}

async fn fetch_gemini_models(client: &reqwest::Client, key: &str) -> Result<Vec<String>, String> {
    let resp = client
        .get("https://generativelanguage.googleapis.com/v1beta/models")
        .header("x-goog-api-key", key)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let val: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let mut models: Vec<String> = val["models"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter(|m| {
                    m["supportedGenerationMethods"]
                        .as_array()
                        .map(|methods| methods.iter().any(|x| x.as_str() == Some("generateContent")))
                        .unwrap_or(false)
                })
                .filter_map(|m| {
                    m["name"]
                        .as_str()
                        .map(|n| n.trim_start_matches("models/").to_string())
                })
                .filter(|n| n.starts_with("gemini-"))
                .collect()
        })
        .unwrap_or_default();
    models.reverse();
    models.truncate(20);
    Ok(models)
}

fn fallback_models(provider: &str) -> Vec<String> {
    match provider {
        "ollama" => vec!["llama3.2".into(), "qwen2.5".into(), "gemma2".into()],
        "lmstudio" => vec!["local-model".into()],
        "gemini" => vec![
            "gemini-2.5-flash".into(),
            "gemini-2.5-pro".into(),
            "gemini-2.0-flash".into(),
        ],
        "anthropic" => vec![
            "claude-sonnet-4-20250514".into(),
            "claude-3-5-sonnet-latest".into(),
            "claude-3-5-haiku-latest".into(),
        ],
        "openai" => vec![
            "gpt-4o-mini".into(),
            "gpt-4o".into(),
            "o4-mini".into(),
        ],
        "github-models" => vec![
            "gpt-4o-mini".into(),
            "gpt-4o".into(),
            "Meta-Llama-3.1-8B-Instruct".into(),
        ],
        "claude-cli" => vec!["sonnet".into(), "opus".into(), "haiku".into()],
        "gemini-cli" => vec!["gemini-2.5-flash".into(), "gemini-2.5-pro".into()],
        _ => vec![],
    }
}

pub fn read_api_key_or_env(secret_key: &str, env_var: &str) -> Option<String> {
    crate::secrets::read_secret_internal(secret_key)
        .or_else(|| std::env::var(env_var).ok().filter(|v| !v.trim().is_empty()))
}

#[tauri::command]
pub async fn detect_ai_providers(
    ollama_endpoint: Option<String>,
) -> Result<Vec<DetectedAiProvider>, String> {
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let client = reqwest::Client::new();
    let ollama_url = ollama_endpoint
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("http://127.0.0.1:11434")
        .to_string();

    let mut providers = Vec::new();

    // Ollama
    let ollama_running = is_process_running(&sys, &["ollama"]);
    let ollama_bin = command_exists("ollama");
    let ollama_models_res = fetch_ollama_models(&client, &ollama_url).await;
    let ollama_reachable = ollama_models_res.is_ok();
    let ollama_installed = ollama_running || ollama_bin || ollama_reachable || app_installed_paths("ollama");
    let ollama_models = match ollama_models_res {
        Ok(m) if !m.is_empty() => m,
        _ => fallback_models("ollama"),
    };
    let ollama_default = ollama_models.first().cloned().unwrap_or_else(|| "llama3.2".into());
    providers.push(DetectedAiProvider {
        id: "ollama".into(),
        name: "Ollama (Lokal)".into(),
        installed: ollama_installed,
        running: ollama_running || ollama_reachable,
        has_account: ollama_reachable,
        source: "local-server".into(),
        status_label: if ollama_reachable {
            format!("Aktif · {} model terpasang", ollama_models.len())
        } else if ollama_installed {
            "Terpasang · Server belum aktif".into()
        } else {
            "Tidak terdeteksi".into()
        },
        models: ollama_models,
        default_model: ollama_default,
    });

    // LM Studio
    let lms_running = is_process_running(&sys, &["lm studio", "lmstudio", "lms"]);
    let lms_models_res = fetch_lmstudio_models(&client).await;
    let lms_reachable = lms_models_res.is_ok();
    let lms_installed = lms_running || lms_reachable || app_installed_paths("lmstudio") || command_exists("lms");
    if lms_installed {
        let lms_models = match lms_models_res {
            Ok(m) if !m.is_empty() => m,
            _ => fallback_models("lmstudio"),
        };
        let lms_default = lms_models.first().cloned().unwrap_or_else(|| "local-model".into());
        providers.push(DetectedAiProvider {
            id: "lmstudio".into(),
            name: "LM Studio (Lokal)".into(),
            installed: true,
            running: lms_running || lms_reachable,
            has_account: lms_reachable,
            source: "local-server".into(),
            status_label: if lms_reachable {
                format!("Server aktif · {} model", lms_models.len())
            } else {
                "Terpasang · Aktifkan Local Server".into()
            },
            models: lms_models,
            default_model: lms_default,
        });
    }

    // Gemini
    let gemini_key = read_api_key_or_env("gemini_api_key", "GEMINI_API_KEY");
    let gemini_running = is_process_running(&sys, &["gemini", "antigravity"]);
    let gemini_installed = gemini_running || app_installed_paths("gemini") || command_exists("gemini");
    let gemini_models = if let Some(ref k) = gemini_key {
        fetch_gemini_models(&client, k)
            .await
            .ok()
            .filter(|m| !m.is_empty())
            .unwrap_or_else(|| fallback_models("gemini"))
    } else {
        fallback_models("gemini")
    };
    let gemini_default = gemini_models
        .iter()
        .find(|m| m.contains("2.5-flash"))
        .or_else(|| gemini_models.first())
        .cloned()
        .unwrap_or_else(|| "gemini-2.5-flash".into());
    providers.push(DetectedAiProvider {
        id: "gemini".into(),
        name: "Google Gemini".into(),
        installed: gemini_installed || gemini_key.is_some(),
        running: gemini_running,
        has_account: gemini_key.is_some(),
        source: if gemini_key.is_some() { "api-key".into() } else { "app".into() },
        status_label: if gemini_key.is_some() {
            format!("Akun terhubung · {} model", gemini_models.len())
        } else if gemini_installed {
            "Aplikasi terdeteksi · Tambah API Key".into()
        } else {
            "Tambah Gemini API Key".into()
        },
        models: gemini_models,
        default_model: gemini_default,
    });

    // Gemini CLI
    if command_exists("gemini") {
        let cli_models = fallback_models("gemini-cli");
        providers.push(DetectedAiProvider {
            id: "gemini-cli".into(),
            name: "Gemini CLI (Akun Lokal)".into(),
            installed: true,
            running: is_process_running(&sys, &["gemini"]),
            has_account: true,
            source: "cli".into(),
            status_label: "CLI terdeteksi · Pakai sesi akun lokal".into(),
            default_model: cli_models[0].clone(),
            models: cli_models,
        });
    }

    // Claude
    let claude_key = read_api_key_or_env("anthropic_api_key", "ANTHROPIC_API_KEY");
    let claude_running = is_process_running(&sys, &["claude"]);
    let claude_installed = claude_running || app_installed_paths("claude") || command_exists("claude");
    let claude_models = if let Some(ref k) = claude_key {
        fetch_anthropic_models(&client, k)
            .await
            .ok()
            .filter(|m| !m.is_empty())
            .unwrap_or_else(|| fallback_models("anthropic"))
    } else {
        fallback_models("anthropic")
    };
    let claude_default = claude_models
        .first()
        .cloned()
        .unwrap_or_else(|| "claude-3-5-sonnet-latest".into());
    providers.push(DetectedAiProvider {
        id: "anthropic".into(),
        name: "Anthropic Claude".into(),
        installed: claude_installed || claude_key.is_some(),
        running: claude_running,
        has_account: claude_key.is_some(),
        source: if claude_key.is_some() { "api-key".into() } else { "app".into() },
        status_label: if claude_key.is_some() {
            format!("Akun terhubung · {} model", claude_models.len())
        } else if claude_installed {
            "Claude terdeteksi · Tambah API Key".into()
        } else {
            "Tambah Anthropic API Key".into()
        },
        models: claude_models,
        default_model: claude_default,
    });

    // Claude CLI
    if command_exists("claude") {
        let cli_models = fallback_models("claude-cli");
        providers.push(DetectedAiProvider {
            id: "claude-cli".into(),
            name: "Claude Code CLI (Akun Lokal)".into(),
            installed: true,
            running: is_process_running(&sys, &["claude"]),
            has_account: true,
            source: "cli".into(),
            status_label: "CLI terdeteksi · Pakai login akun Claude".into(),
            default_model: cli_models[0].clone(),
            models: cli_models,
        });
    }

    // OpenAI
    let openai_key = read_api_key_or_env("openai_api_key", "OPENAI_API_KEY");
    let chatgpt_running = is_process_running(&sys, &["chatgpt", "codex"]);
    let chatgpt_installed = chatgpt_running || app_installed_paths("chatgpt") || command_exists("codex");
    let openai_models = if let Some(ref k) = openai_key {
        fetch_openai_models(&client, k)
            .await
            .ok()
            .filter(|m| !m.is_empty())
            .unwrap_or_else(|| fallback_models("openai"))
    } else {
        fallback_models("openai")
    };
    let openai_default = openai_models
        .iter()
        .find(|m| m.as_str() == "gpt-4o-mini")
        .or_else(|| openai_models.first())
        .cloned()
        .unwrap_or_else(|| "gpt-4o-mini".into());
    providers.push(DetectedAiProvider {
        id: "openai".into(),
        name: "OpenAI / ChatGPT".into(),
        installed: chatgpt_installed || openai_key.is_some(),
        running: chatgpt_running,
        has_account: openai_key.is_some(),
        source: if openai_key.is_some() { "api-key".into() } else { "app".into() },
        status_label: if openai_key.is_some() {
            format!("Akun terhubung · {} model", openai_models.len())
        } else if chatgpt_installed {
            "ChatGPT terdeteksi · Tambah API Key".into()
        } else {
            "Tambah OpenAI API Key".into()
        },
        models: openai_models,
        default_model: openai_default,
    });

    // GitHub Models
    let gh_token = crate::secrets::read_secret_internal("github_token");
    let gh_cli = command_exists("gh");
    if gh_token.is_some() || gh_cli {
        let gh_models = fallback_models("github-models");
        providers.push(DetectedAiProvider {
            id: "github-models".into(),
            name: "GitHub Models".into(),
            installed: true,
            running: false,
            has_account: gh_token.is_some(),
            source: "api-key".into(),
            status_label: if gh_token.is_some() {
                "Siap · Pakai GitHub PAT".into()
            } else {
                "GitHub CLI terdeteksi · Tambah PAT".into()
            },
            default_model: gh_models[0].clone(),
            models: gh_models,
        });
    }

    providers.sort_by(|a, b| {
        b.has_account
            .cmp(&a.has_account)
            .then(b.running.cmp(&a.running))
            .then(b.installed.cmp(&a.installed))
    });

    Ok(providers)
}

#[tauri::command]
pub async fn fetch_provider_models(
    provider: String,
    ollama_endpoint: Option<String>,
) -> Result<Vec<String>, String> {
    let client = reqwest::Client::new();
    let models = match provider.as_str() {
        "ollama" => {
            let base = ollama_endpoint
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or("http://127.0.0.1:11434");
            fetch_ollama_models(&client, base)
                .await
                .unwrap_or_else(|_| fallback_models("ollama"))
        }
        "lmstudio" => fetch_lmstudio_models(&client)
            .await
            .unwrap_or_else(|_| fallback_models("lmstudio")),
        "openai" => {
            if let Some(key) = read_api_key_or_env("openai_api_key", "OPENAI_API_KEY") {
                fetch_openai_models(&client, &key)
                    .await
                    .unwrap_or_else(|_| fallback_models("openai"))
            } else {
                fallback_models("openai")
            }
        }
        "anthropic" => {
            if let Some(key) = read_api_key_or_env("anthropic_api_key", "ANTHROPIC_API_KEY") {
                fetch_anthropic_models(&client, &key)
                    .await
                    .unwrap_or_else(|_| fallback_models("anthropic"))
            } else {
                fallback_models("anthropic")
            }
        }
        "gemini" => {
            if let Some(key) = read_api_key_or_env("gemini_api_key", "GEMINI_API_KEY") {
                fetch_gemini_models(&client, &key)
                    .await
                    .unwrap_or_else(|_| fallback_models("gemini"))
            } else {
                fallback_models("gemini")
            }
        }
        other => fallback_models(other),
    };
    Ok(models)
}
