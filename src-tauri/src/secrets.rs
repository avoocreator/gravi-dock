use keyring::Entry;
use serde::Serialize;

const SERVICE_NAME: &str = "app.nobi.island";

#[derive(Serialize)]
pub struct SecretStatus {
    pub github_token: bool,
    pub google_ical_url: bool,
    pub google_access_token: bool,
    pub openai_api_key: bool,
    pub anthropic_api_key: bool,
    pub gemini_api_key: bool,
}

pub fn read_secret_internal(key: &str) -> Option<String> {
    let entry = Entry::new(SERVICE_NAME, key).ok()?;
    entry.get_password().ok().filter(|v| !v.trim().is_empty())
}

#[tauri::command]
pub fn save_secret(key: String, value: String) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, &key).map_err(|e| e.to_string())?;
    if value.trim().is_empty() {
        let _ = entry.delete_credential();
        return Ok(());
    }
    entry.set_password(value.trim()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_secret(key: String) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, &key).map_err(|e| e.to_string())?;
    entry.delete_credential().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_secret_status() -> SecretStatus {
    SecretStatus {
        github_token: read_secret_internal("github_token").is_some(),
        google_ical_url: read_secret_internal("google_ical_url").is_some(),
        google_access_token: read_secret_internal("google_access_token").is_some(),
        openai_api_key: read_secret_internal("openai_api_key").is_some(),
        anthropic_api_key: read_secret_internal("anthropic_api_key").is_some(),
        gemini_api_key: read_secret_internal("gemini_api_key").is_some(),
    }
}
