use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter};

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .pool_max_idle_per_host(8)
            .tcp_keepalive(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(8))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatPayload {
    pub provider: String,
    pub model: String,
    pub prompt: String,
    pub attached_files: Vec<String>,
    pub ollama_endpoint: Option<String>,
    pub system_prompt: Option<String>,
    pub max_file_kb: Option<u32>,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct ChatChunkEvent {
    request_id: String,
    delta: String,
}

fn emit_chunk(app: &AppHandle, req_id: Option<&str>, delta: &str) {
    if delta.is_empty() {
        return;
    }
    if let Some(id) = req_id {
        let _ = app.emit(
            "nobi:chat-chunk",
            ChatChunkEvent {
                request_id: id.to_string(),
                delta: delta.to_string(),
            },
        );
    }
}

fn build_prompt_with_files(prompt: &str, files: &[String], max_file_kb: u32) -> String {
    if files.is_empty() {
        return prompt.to_string();
    }
    let max_bytes = (max_file_kb.max(16) as u64) * 1024;
    let mut ctx = String::from("File terlampir:\n");
    for path in files.iter().take(4) {
        if let Ok(meta) = std::fs::metadata(path) {
            if meta.len() > max_bytes {
                ctx.push_str(&format!(
                    "\n--- {} (melebihi batas {} KB, hanya 4000 karakter awal dibaca) ---\n",
                    path, max_file_kb
                ));
            }
        }
        if let Ok(content) = std::fs::read_to_string(path) {
            let snippet: String = content.chars().take(4000).collect();
            ctx.push_str(&format!("\n--- {} ---\n{}\n", path, snippet));
        } else {
            ctx.push_str(&format!("\n--- {} (file biner/tidak terbaca) ---\n", path));
        }
    }
    format!("{}\nPertanyaan pengguna:\n{}", ctx, prompt)
}

async fn check_http_error(resp: reqwest::Response, provider: &str) -> Result<reqwest::Response, String> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    let text = resp.text().await.unwrap_or_default();
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
        if let Some(msg) = val["error"]["message"]
            .as_str()
            .or_else(|| val["error"].as_str())
            .or_else(|| val["message"].as_str())
        {
            return Err(format!("{} (HTTP {}): {}", provider, status.as_u16(), msg));
        }
    }
    let short: String = text.chars().take(240).collect();
    Err(format!("{} error HTTP {}: {}", provider, status.as_u16(), short))
}

async fn stream_sse_response(
    app: &AppHandle,
    req_id: Option<&str>,
    mut resp: reqwest::Response,
    mode: &str,
) -> Result<String, String> {
    let mut full = String::new();
    let mut buf = String::new();

    while let Some(chunk) = resp.chunk().await.map_err(|e| e.to_string())? {
        buf.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(pos) = buf.find('\n') {
            let line = buf[..pos].trim().to_string();
            buf.drain(..=pos);
            if !line.starts_with("data:") {
                continue;
            }
            let data_str = line.trim_start_matches("data:").trim();
            if data_str.is_empty() || data_str == "[DONE]" {
                continue;
            }
            let Ok(val) = serde_json::from_str::<serde_json::Value>(data_str) else {
                continue;
            };
            let delta = match mode {
                "openai" => val["choices"][0]["delta"]["content"].as_str().unwrap_or(""),
                "anthropic" => {
                    if val["type"].as_str() == Some("content_block_delta") {
                        val["delta"]["text"].as_str().unwrap_or("")
                    } else {
                        ""
                    }
                }
                "gemini" => val["candidates"][0]["content"]["parts"][0]["text"]
                    .as_str()
                    .unwrap_or(""),
                _ => "",
            };
            if !delta.is_empty() {
                full.push_str(delta);
                emit_chunk(app, req_id, delta);
            }
        }
    }

    Ok(full)
}

async fn run_local_cli(bin: &str, args: Vec<String>) -> Result<String, String> {
    let bin_owned = bin.to_string();
    tokio::task::spawn_blocking(move || {
        let mut cmd = std::process::Command::new(&bin_owned);
        cmd.args(&args);
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        let out = cmd
            .output()
            .map_err(|e| format!("Gagal menjalankan {} CLI: {}", bin_owned, e))?;
        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        if !out.status.success() {
            let msg = if !stderr.is_empty() { stderr } else { stdout };
            return Err(format!("{} CLI error: {}", bin_owned, msg));
        }
        if stdout.is_empty() {
            Ok("(Respons kosong dari CLI)".into())
        } else {
            Ok(stdout)
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn send_chat_message(app: AppHandle, payload: ChatPayload) -> Result<String, String> {
    let client = http_client();
    let req_id = payload.request_id.as_deref();
    let full_prompt = build_prompt_with_files(
        &payload.prompt,
        &payload.attached_files,
        payload.max_file_kb.unwrap_or(256),
    );
    let sys_prompt = payload
        .system_prompt
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    match payload.provider.as_str() {
        "ollama" => {
            let model = if payload.model.is_empty() {
                "llama3.2"
            } else {
                &payload.model
            };
            let base = payload
                .ollama_endpoint
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or("http://127.0.0.1:11434")
                .trim_end_matches('/');
            let url = format!("{}/api/chat", base);

            let mut messages = Vec::new();
            if let Some(sys) = sys_prompt {
                messages.push(json!({"role": "system", "content": sys}));
            }
            messages.push(json!({"role": "user", "content": full_prompt}));

            let resp = client
                .post(&url)
                .json(&json!({
                    "model": model,
                    "stream": true,
                    "keep_alive": "15m",
                    "options": { "num_predict": 640 },
                    "messages": messages
                }))
                .send()
                .await
                .map_err(|e| format!("Ollama tidak terjangkau di {} ({})", base, e))?;
            let mut resp = check_http_error(resp, "Ollama").await?;

            let mut full = String::new();
            let mut buf = String::new();
            while let Some(chunk) = resp.chunk().await.map_err(|e| e.to_string())? {
                buf.push_str(&String::from_utf8_lossy(&chunk));
                while let Some(pos) = buf.find('\n') {
                    let line = buf[..pos].trim().to_string();
                    buf.drain(..=pos);
                    if line.is_empty() {
                        continue;
                    }
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&line) {
                        if let Some(delta) = val["message"]["content"].as_str() {
                            if !delta.is_empty() {
                                full.push_str(delta);
                                emit_chunk(&app, req_id, delta);
                            }
                        }
                    }
                }
            }
            if full.is_empty() {
                Ok("(Respons kosong dari Ollama)".into())
            } else {
                Ok(full)
            }
        }
        "lmstudio" => {
            let model = if payload.model.is_empty() {
                "local-model"
            } else {
                &payload.model
            };
            let mut messages = Vec::new();
            if let Some(sys) = sys_prompt {
                messages.push(json!({"role": "system", "content": sys}));
            }
            messages.push(json!({"role": "user", "content": full_prompt}));

            let resp = client
                .post("http://127.0.0.1:1234/v1/chat/completions")
                .json(&json!({
                    "model": model,
                    "stream": true,
                    "max_tokens": 640,
                    "messages": messages
                }))
                .send()
                .await
                .map_err(|e| format!("LM Studio Local Server tidak aktif di port 1234 ({})", e))?;
            let resp = check_http_error(resp, "LM Studio").await?;
            let out = stream_sse_response(&app, req_id, resp, "openai").await?;
            Ok(if out.is_empty() { "(Respons kosong dari LM Studio)".into() } else { out })
        }
        "openai" => {
            let key = crate::ai_providers::read_api_key_or_env("openai_api_key", "OPENAI_API_KEY")
                .ok_or_else(|| "OpenAI API Key belum diatur di Pengaturan.".to_string())?;
            let model = if payload.model.is_empty() {
                "gpt-4o-mini"
            } else {
                &payload.model
            };
            let mut messages = Vec::new();
            if let Some(sys) = sys_prompt {
                messages.push(json!({"role": "system", "content": sys}));
            }
            messages.push(json!({"role": "user", "content": full_prompt}));

            let resp = client
                .post("https://api.openai.com/v1/chat/completions")
                .bearer_auth(key)
                .json(&json!({
                    "model": model,
                    "stream": true,
                    "messages": messages
                }))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let resp = check_http_error(resp, "OpenAI").await?;
            let out = stream_sse_response(&app, req_id, resp, "openai").await?;
            Ok(if out.is_empty() { "(Respons kosong dari OpenAI)".into() } else { out })
        }
        "github-models" => {
            let key = crate::secrets::read_secret_internal("github_token")
                .ok_or_else(|| "GitHub PAT belum diatur di Pengaturan.".to_string())?;
            let model = if payload.model.is_empty() {
                "gpt-4o-mini"
            } else {
                &payload.model
            };
            let mut messages = Vec::new();
            if let Some(sys) = sys_prompt {
                messages.push(json!({"role": "system", "content": sys}));
            }
            messages.push(json!({"role": "user", "content": full_prompt}));

            let resp = client
                .post("https://models.inference.ai.azure.com/chat/completions")
                .bearer_auth(key)
                .json(&json!({
                    "model": model,
                    "stream": true,
                    "messages": messages
                }))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let resp = check_http_error(resp, "GitHub Models").await?;
            let out = stream_sse_response(&app, req_id, resp, "openai").await?;
            Ok(if out.is_empty() { "(Respons kosong dari GitHub Models)".into() } else { out })
        }
        "anthropic" => {
            let key = crate::ai_providers::read_api_key_or_env("anthropic_api_key", "ANTHROPIC_API_KEY")
                .ok_or_else(|| "Anthropic API Key belum diatur di Pengaturan.".to_string())?;
            let model = if payload.model.is_empty() {
                "claude-3-5-sonnet-latest"
            } else {
                &payload.model
            };
            let mut body = json!({
                "model": model,
                "max_tokens": 1024,
                "stream": true,
                "messages": [{"role": "user", "content": full_prompt}]
            });
            if let Some(sys) = sys_prompt {
                body["system"] = json!(sys);
            }
            let resp = client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", key)
                .header("anthropic-version", "2023-06-01")
                .json(&body)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let resp = check_http_error(resp, "Anthropic").await?;
            let out = stream_sse_response(&app, req_id, resp, "anthropic").await?;
            Ok(if out.is_empty() { "(Respons kosong dari Anthropic)".into() } else { out })
        }
        "gemini" => {
            let key = crate::ai_providers::read_api_key_or_env("gemini_api_key", "GEMINI_API_KEY")
                .ok_or_else(|| "Gemini API Key belum diatur di Pengaturan.".to_string())?;
            let model = if payload.model.is_empty() {
                "gemini-2.5-flash"
            } else {
                &payload.model
            };
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse",
                model
            );
            let mut body = json!({
                "contents": [{"parts": [{"text": full_prompt}]}]
            });
            if let Some(sys) = sys_prompt {
                body["systemInstruction"] = json!({"parts": [{"text": sys}]});
            }
            let resp = client
                .post(&url)
                .header("x-goog-api-key", key)
                .json(&body)
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let resp = check_http_error(resp, "Gemini").await?;
            let out = stream_sse_response(&app, req_id, resp, "gemini").await?;
            Ok(if out.is_empty() { "(Respons kosong dari Gemini)".into() } else { out })
        }
        "claude-cli" => {
            let combined = match sys_prompt {
                Some(sys) => format!("{}\n\n{}", sys, full_prompt),
                None => full_prompt,
            };
            let mut args = vec!["-p".to_string(), combined];
            if !payload.model.trim().is_empty() {
                args.push("--model".to_string());
                args.push(payload.model.trim().to_string());
            }
            let out = run_local_cli("claude", args).await?;
            emit_chunk(&app, req_id, &out);
            Ok(out)
        }
        "gemini-cli" => {
            let combined = match sys_prompt {
                Some(sys) => format!("{}\n\n{}", sys, full_prompt),
                None => full_prompt,
            };
            let mut args = vec!["-p".to_string(), combined];
            if !payload.model.trim().is_empty() {
                args.push("-m".to_string());
                args.push(payload.model.trim().to_string());
            }
            let out = run_local_cli("gemini", args).await?;
            emit_chunk(&app, req_id, &out);
            Ok(out)
        }
        _ => Err("Provider AI tidak dikenal.".into()),
    }
}
