//! Google Calendar lewat OAuth 2.0 untuk aplikasi desktop (loopback + PKCE). Tanpa dependensi: SHA-256 dan base64url ditulis di sini.
//! Pengambilan HTTP dilakukan app (curl.exe); modul ini hanya membuat URL/badan permintaan dan mem-parse respons.
use crate::ics::Event;
use crate::json;
use crate::prayer::days_from_civil;
use crate::timeutil::{fmt_rfc3339, parse_rfc3339};

pub const SCOPE: &str = "https://www.googleapis.com/auth/calendar.readonly";
pub const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

/* ---------------- SHA-256 ---------------- */

pub fn sha256(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];
    let mut h: [u32; 8] = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 { msg.push(0); }
    msg.extend_from_slice(&((data.len() as u64) * 8).to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 { w[i] = u32::from_be_bytes([chunk[i * 4], chunk[i * 4 + 1], chunk[i * 4 + 2], chunk[i * 4 + 3]]); }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g; g = f; f = e; e = d.wrapping_add(t1); d = c; c = b; b = a; a = t1.wrapping_add(t2);
        }
        for (x, y) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) { *x = x.wrapping_add(y); }
    }
    let mut out = [0u8; 32];
    for (i, x) in h.iter().enumerate() { out[i * 4..i * 4 + 4].copy_from_slice(&x.to_be_bytes()); }
    out
}

pub fn b64url(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut s = String::new();
    for c in data.chunks(3) {
        let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
        s.push(T[(n >> 18) as usize & 63] as char);
        s.push(T[(n >> 12) as usize & 63] as char);
        if c.len() > 1 { s.push(T[(n >> 6) as usize & 63] as char); }
        if c.len() > 2 { s.push(T[n as usize & 63] as char); }
    }
    s
}

/// Byte acak dari RandomState (diseed OS) dicampur waktu. Cukup untuk verifier PKCE dan `state`.
pub fn random_bytes(n: usize) -> Vec<u8> {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0);
    let mut out = Vec::with_capacity(n + 8);
    let mut i = 0u64;
    while out.len() < n {
        let mut h = RandomState::new().build_hasher();
        h.write_u64(nanos ^ i.wrapping_mul(0x9e3779b97f4a7c15));
        out.extend_from_slice(&h.finish().to_le_bytes());
        i += 1;
    }
    out.truncate(n);
    out
}

pub fn pkce_verifier() -> String { b64url(&random_bytes(48)) }
pub fn pkce_challenge(verifier: &str) -> String { b64url(&sha256(verifier.as_bytes())) }

/* ---------------- URL & badan permintaan ---------------- */

pub fn url_encode(s: &str) -> String {
    let mut o = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') { o.push(b as char); } else { o.push_str(&format!("%{:02X}", b)); }
    }
    o
}

pub fn url_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'%' if b.len() >= i + 3 => {
                if let Ok(v) = u8::from_str_radix(std::str::from_utf8(&b[i + 1..i + 3]).unwrap_or("zz"), 16) { out.push(v); i += 3; } else { out.push(b'%'); i += 1; }
            }
            b'+' => { out.push(b' '); i += 1; }
            c => { out.push(c); i += 1; }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub fn auth_url(client_id: &str, redirect: &str, challenge: &str, state: &str) -> String {
    format!("{AUTH_URL}?client_id={}&redirect_uri={}&response_type=code&scope={}&code_challenge={challenge}&code_challenge_method=S256&state={state}&access_type=offline&prompt=consent",
        url_encode(client_id), url_encode(redirect), url_encode(SCOPE))
}

fn form(kv: &[(&str, &str)]) -> String { kv.iter().filter(|(_, v)| !v.is_empty()).map(|(k, v)| format!("{k}={}", url_encode(v))).collect::<Vec<_>>().join("&") }

pub fn exchange_body(client_id: &str, secret: &str, code: &str, verifier: &str, redirect: &str) -> String {
    form(&[("client_id", client_id), ("client_secret", secret), ("code", code), ("code_verifier", verifier), ("grant_type", "authorization_code"), ("redirect_uri", redirect)])
}
pub fn refresh_body(client_id: &str, secret: &str, refresh_token: &str) -> String {
    form(&[("client_id", client_id), ("client_secret", secret), ("refresh_token", refresh_token), ("grant_type", "refresh_token")])
}

/// Ambil `code` dan `state` dari baris permintaan HTTP loopback: "GET /?code=..&state=.. HTTP/1.1".
pub fn parse_redirect(request_line: &str) -> Result<(String, String), String> {
    let path = request_line.split_whitespace().nth(1).ok_or("permintaan kosong")?;
    let q = path.split_once('?').map(|x| x.1).ok_or("tidak ada parameter")?;
    let (mut code, mut state, mut err) = (String::new(), String::new(), String::new());
    for kv in q.split('&') {
        if let Some((k, v)) = kv.split_once('=') {
            match k { "code" => code = url_decode(v), "state" => state = url_decode(v), "error" => err = url_decode(v), _ => {} }
        }
    }
    if !err.is_empty() { return Err(err); }
    if code.is_empty() { return Err("kode tidak ada".into()); }
    Ok((code, state))
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token { pub access: String, pub refresh: Option<String>, pub expires_in: i64 }

pub fn parse_token(src: &str) -> Result<Token, String> {
    let v = json::parse(src).ok_or("respons bukan JSON")?;
    if let Some(e) = v.get("error").str() { return Err(format!("{e}: {}", v.get("error_description").text())); }
    let access = v.get("access_token").str().ok_or("access_token tidak ada")?.to_string();
    Ok(Token { access, refresh: v.get("refresh_token").str().map(String::from), expires_in: v.get("expires_in").num().unwrap_or(3600.0) as i64 })
}

pub fn events_url(time_min: i64, time_max: i64) -> String {
    format!("https://www.googleapis.com/calendar/v3/calendars/primary/events?singleEvents=true&orderBy=startTime&maxResults=50&timeMin={}&timeMax={}",
        url_encode(&fmt_rfc3339(time_min)), url_encode(&fmt_rfc3339(time_max)))
}

/// Respons `events.list` -> daftar `ics::Event` (pakai ulang UI kalender). Acara sehari penuh = tengah malam lokal `tz`.
pub fn parse_events(src: &str, tz: f64) -> Result<Vec<Event>, String> {
    let v = json::parse(src).ok_or("respons bukan JSON")?;
    if let Some(e) = v.get("error").get("message").str() { return Err(e.to_string()); }
    let mut out = vec![];
    for it in v.get("items").arr() {
        if it.get("status").text() == "cancelled" { continue; }
        let side = |k: &str| -> Option<(i64, bool)> {
            let s = it.get(k);
            if let Some(t) = s.get("dateTime").str() { return parse_rfc3339(t).map(|x| (x, false)); }
            let d = s.get("date").str()?;
            let n = |a: usize, b: usize| d.get(a..b)?.parse::<i64>().ok();
            Some((days_from_civil(n(0, 4)?, n(5, 7)?, n(8, 10)?) * 86400 - (tz * 3600.0) as i64, true))
        };
        let Some((start, all_day)) = side("start") else { continue };
        let end = side("end").map(|x| x.0).unwrap_or(start + 3600);
        let title = { let t = it.get("summary").text(); if t.is_empty() { "(Tanpa judul)" } else { t } };
        out.push(Event { title: title.to_string(), start, end, all_day });
    }
    out.sort_by_key(|e| e.start);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{x:02x}")).collect() }

    #[test]
    fn sha256_vectors() {
        assert_eq!(hex(&sha256(b"")), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert_eq!(hex(&sha256(b"abc")), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        assert_eq!(hex(&sha256(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq")), "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1");
        assert_eq!(hex(&sha256(&vec![b'a'; 1000])).len(), 64);
    }
    #[test]
    fn pkce_matches_rfc7636_appendix_b() {
        assert_eq!(pkce_challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"), "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
        let v = pkce_verifier();
        assert!(v.len() >= 43 && v.len() <= 128 && v.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
        assert_ne!(pkce_verifier(), v);
    }
    #[test]
    fn urls_and_redirect() {
        assert_eq!(url_encode("a b/c"), "a%20b%2Fc");
        assert_eq!(url_decode("a%20b+c%2F"), "a b c/");
        let u = auth_url("id.apps", "http://127.0.0.1:5555", "CH", "ST");
        assert!(u.contains("code_challenge=CH") && u.contains("redirect_uri=http%3A%2F%2F127.0.0.1%3A5555") && u.contains("access_type=offline"));
        assert_eq!(parse_redirect("GET /?state=ST&code=4%2F0Abc HTTP/1.1").unwrap(), ("4/0Abc".to_string(), "ST".to_string()));
        assert!(parse_redirect("GET /?error=access_denied HTTP/1.1").is_err());
        assert!(parse_redirect("GET /favicon.ico HTTP/1.1").is_err());
        assert!(exchange_body("id", "", "c", "v", "r").find("client_secret").is_none(), "secret kosong tidak dikirim");
    }
    #[test]
    fn token_parse() {
        let t = parse_token(r#"{"access_token":"ya29","expires_in":3599,"refresh_token":"1//r"}"#).unwrap();
        assert_eq!((t.access.as_str(), t.refresh.as_deref(), t.expires_in), ("ya29", Some("1//r"), 3599));
        assert!(parse_token(r#"{"error":"invalid_grant","error_description":"Bad"}"#).unwrap_err().starts_with("invalid_grant"));
    }
    #[test]
    fn events_parse_timed_and_all_day() {
        let src = r#"{"items":[
          {"summary":"Rapat","start":{"dateTime":"2026-11-24T09:00:00+07:00"},"end":{"dateTime":"2026-11-24T10:00:00+07:00"}},
          {"summary":"Libur","start":{"date":"2026-11-23"},"end":{"date":"2026-11-24"}},
          {"status":"cancelled","summary":"batal","start":{"date":"2026-11-25"}},
          {"start":{"dateTime":"2026-11-24T14:00:00Z"}}]}"#;
        let ev = parse_events(src, 7.0).unwrap();
        assert_eq!(ev.len(), 3);
        assert!(ev[0].all_day && ev[0].title == "Libur");
        assert_eq!(ev[1].end - ev[1].start, 3600);
        assert_eq!(ev[2].title, "(Tanpa judul)");
        assert!(parse_events(r#"{"error":{"message":"Invalid Credentials"}}"#, 7.0).is_err());
    }
}
