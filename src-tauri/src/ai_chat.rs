use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatPayload {
    pub provider: String, // "ollama" | "openai" | "anthropic" | "gemini"
    pub model: String,
    pub prompt: String,
    pub attached_files: Vec<String>,
}

fn build_prompt_with_files(prompt: &str, files: &[String]) -> String {
    if files.is_empty() {
        return prompt.to_string();
    }
    let mut ctx = String::from("File terlampir:\n");
    for path in files.iter().take(4) {
        if let Ok(content) = std::fs::read_to_string(path) {
            let snippet: String = content.chars().take(4000).collect();
            ctx.push_str(&format!("\n--- {} ---\n{}\n", path, snippet));
        } else {
            ctx.push_str(&format!("\n--- {} (file biner/tidak terbaca) ---\n", path));
        }
    }
    format!("{}\nPertanyaan pengguna:\n{}", ctx, prompt)
}

#[tauri::command]
pub async fn send_chat_message(payload: ChatPayload) -> Result<String, String> {
    let client = reqwest::Client::new();
    let full_prompt = build_prompt_with_files(&payload.prompt, &payload.attached_files);

    match payload.provider.as_str() {
        "ollama" => {
            let model = if payload.model.is_empty() {
                "llama3.2"
            } else {
                &payload.model
            };
            let resp = client
                .post("http://127.0.0.1:11434/api/chat")
                .json(&json!({
                    "model": model,
                    "stream": false,
                    "messages": [{"role": "user", "content": full_prompt}]
                }))
                .send()
                .await
                .map_err(|e| format!("Ollama tidak terjangkau di localhost:11434 ({})", e))?;
            let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            Ok(data["message"]["content"]
                .as_str()
                .unwrap_or("(Respons kosong dari Ollama)")
                .to_string())
        }
        "openai" => {
            let key = crate::secrets::read_secret_internal("openai_api_key")
                .ok_or_else(|| "OpenAI API Key belum diatur di Pengaturan.".to_string())?;
            let model = if payload.model.is_empty() {
                "gpt-4o-mini"
            } else {
                &payload.model
            };
            let resp = client
                .post("https://api.openai.com/v1/chat/completions")
                .bearer_auth(key)
                .json(&json!({
                    "model": model,
                    "messages": [{"role": "user", "content": full_prompt}]
                }))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            Ok(data["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("(Respons kosong dari OpenAI)")
                .to_string())
        }
        "anthropic" => {
            let key = crate::secrets::read_secret_internal("anthropic_api_key")
                .ok_or_else(|| "Anthropic API Key belum diatur di Pengaturan.".to_string())?;
            let model = if payload.model.is_empty() {
                "claude-3-5-sonnet-latest"
            } else {
                &payload.model
            };
            let resp = client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", key)
                .header("anthropic-version", "2023-06-01")
                .json(&json!({
                    "model": model,
                    "max_tokens": 1024,
                    "messages": [{"role": "user", "content": full_prompt}]
                }))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            Ok(data["content"][0]["text"]
                .as_str()
                .unwrap_or("(Respons kosong dari Anthropic)")
                .to_string())
        }
        "gemini" => {
            let key = crate::secrets::read_secret_internal("gemini_api_key")
                .ok_or_else(|| "Gemini API Key belum diatur di Pengaturan.".to_string())?;
            let model = if payload.model.is_empty() {
                "gemini-2.5-flash"
            } else {
                &payload.model
            };
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                model, key
            );
            let resp = client
                .post(&url)
                .json(&json!({
                    "contents": [{"parts": [{"text": full_prompt}]}]
                }))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            Ok(data["candidates"][0]["content"]["parts"][0]["text"]
                .as_str()
                .unwrap_or("(Respons kosong dari Gemini)")
                .to_string())
        }
        _ => Err("Provider AI tidak dikenal.".into()),
    }
}
