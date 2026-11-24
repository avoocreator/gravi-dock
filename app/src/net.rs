//! HTTP lewat curl.exe (bawaan Windows 10+): tanpa dependensi, tanpa jendela konsol. Rahasia dikirim lewat stdin/berkas config, bukan argumen.
use std::io::Write;
use std::process::{Command, Stdio};

pub struct Req<'a> { pub url: &'a str, pub headers: Vec<String>, pub body: Option<&'a str>, pub timeout_s: u32 }

impl<'a> Req<'a> {
    pub fn get(url: &'a str) -> Self { Req { url, headers: vec![], body: None, timeout_s: 20 } }
    pub fn post(url: &'a str, body: &'a str) -> Self { Req { url, headers: vec![], body: Some(body), timeout_s: 20 } }
    pub fn header(mut self, h: impl Into<String>) -> Self { self.headers.push(h.into()); self }
}

fn curl_bin() -> &'static str { if cfg!(windows) { "curl.exe" } else { "curl" } }

/// Konfigurasi curl (URL + header) dikirim lewat stdin (`-K -`) supaya token tidak muncul di daftar proses.
fn config_for(r: &Req) -> String {
    let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', " ").replace('\r', " ");
    let mut c = format!("url = \"{}\"\n", esc(r.url));
    for h in &r.headers { c.push_str(&format!("header = \"{}\"\n", esc(h))); }
    c
}

/// -> (status HTTP, badan). None bila curl tidak bisa dijalankan / jaringan gagal total.
pub fn send(r: &Req) -> Option<(u16, String)> {
    let mut cmd = Command::new(curl_bin());
    cmd.args(["-s", "-L", "-m", &r.timeout_s.to_string(), "-A", "GraviIsland/1.0", "-w", "\n%{http_code}", "-K", "-"]);
    let body_file = r.body.map(|b| {
        let p = std::env::temp_dir().join(format!("gravi-req-{}-{}.tmp", std::process::id(), b.len()));
        let _ = std::fs::write(&p, b);
        p
    });
    if let Some(p) = &body_file { cmd.arg("--data-binary").arg(format!("@{}", p.display())); }
    cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null());
    #[cfg(windows)]
    { use std::os::windows::process::CommandExt; cmd.creation_flags(0x0800_0000); }
    let mut child = cmd.spawn().ok()?;
    child.stdin.take()?.write_all(config_for(r).as_bytes()).ok()?;
    let out = child.wait_with_output().ok();
    if let Some(p) = body_file { let _ = std::fs::remove_file(p); }
    let out = out?;
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let (body, code) = text.rsplit_once('\n')?;
    Some((code.trim().parse().ok()?, body.to_string()))
}

/// GET sederhana; hanya mengembalikan badan bila status 2xx.
pub fn get_text(url: &str) -> Option<String> {
    let (c, b) = send(&Req::get(url))?;
    if (200..300).contains(&c) { Some(b) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn config_hides_secrets_and_escapes() {
        let r = Req::get("https://x/y?a=\"b\"").header("Authorization: Bearer abc\"d");
        let c = config_for(&r);
        assert!(c.contains("url = \"https://x/y?a=\\\"b\\\"\"") && c.contains("header = \"Authorization: Bearer abc\\\"d\""), "{c}");
    }
    #[test]
    fn missing_network_is_none_or_error_status_not_panic() {
        let r = Req { url: "http://127.0.0.1:9/", headers: vec![], body: None, timeout_s: 2 };
        match send(&r) { None => {} Some((c, _)) => assert!(c == 0 || c >= 400) }
    }
}
