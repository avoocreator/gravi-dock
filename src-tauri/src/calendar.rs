use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEventItem {
    pub id: String,
    pub title: String,
    pub date: String,       // YYYY-MM-DD
    pub start_time: String, // HH:MM or "Sepanjang hari"
    pub end_time: String,
    pub location: Option<String>,
    pub meeting_url: Option<String>,
    pub source: String,     // "google-api" | "ical"
}

fn parse_ics_datetime(raw: &str) -> Option<(String, String)> {
    let val = raw.trim();
    if val.len() >= 8 {
        if let Some(t_idx) = val.find('T') {
            if val.len() >= t_idx + 5 {
                let date_part = &val[0..8];
                let time_part = &val[t_idx + 1..];
                let hhmmss = if time_part.len() >= 6 {
                    &time_part[0..6]
                } else {
                    return None;
                };
                if val.ends_with('Z') {
                    let stamp = format!("{}T{}", date_part, hhmmss);
                    if let Ok(naive) = NaiveDateTime::parse_from_str(&stamp, "%Y%m%dT%H%M%S") {
                        let local_dt = DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc)
                            .with_timezone(&Local);
                        return Some((
                            local_dt.format("%Y-%m-%d").to_string(),
                            local_dt.format("%H:%M").to_string(),
                        ));
                    }
                }
                let y = &val[0..4];
                let m = &val[4..6];
                let d = &val[6..8];
                let hh = &val[t_idx + 1..t_idx + 3];
                let mm = &val[t_idx + 3..t_idx + 5];
                return Some((format!("{}-{}-{}", y, m, d), format!("{}:{}", hh, mm)));
            }
        }
        let y = &val[0..4];
        let m = &val[4..6];
        let d = &val[6..8];
        return Some((format!("{}-{}-{}", y, m, d), "Sepanjang hari".to_string()));
    }
    None
}

fn parse_rfc3339_to_local(raw: &str) -> Option<(String, String)> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(raw) {
        let local_dt = dt.with_timezone(&Local);
        return Some((
            local_dt.format("%Y-%m-%d").to_string(),
            local_dt.format("%H:%M").to_string(),
        ));
    }
    None
}

fn parse_ics_events(ics_text: &str, year: i32, month: u32) -> Vec<CalendarEventItem> {
    let mut events = Vec::new();
    let mut in_event = false;
    let mut uid = String::new();
    let mut summary = String::new();
    let mut dtstart = String::new();
    let mut dtend = String::new();
    let mut location: Option<String> = None;

    // Unfold RFC 5545 folded lines
    let unfolded = ics_text.replace("\r\n ", "").replace("\n ", "");

    for line in unfolded.lines() {
        let trimmed = line.trim();
        if trimmed == "BEGIN:VEVENT" {
            in_event = true;
            uid.clear();
            summary.clear();
            dtstart.clear();
            dtend.clear();
            location = None;
            continue;
        }
        if trimmed == "END:VEVENT" && in_event {
            in_event = false;
            if let Some((date, start_time)) = parse_ics_datetime(&dtstart) {
                if let Ok(parsed) = NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
                    if parsed.year() == year && parsed.month() == month {
                        let end_time = parse_ics_datetime(&dtend)
                            .map(|(_, t)| t)
                            .unwrap_or_default();
                        events.push(CalendarEventItem {
                            id: if uid.is_empty() {
                                format!("ics-{}-{}", date, events.len())
                            } else {
                                uid.clone()
                            },
                            title: if summary.is_empty() {
                                "(Tanpa judul)".into()
                            } else {
                                summary.replace("\\,", ",").replace("\\n", " ")
                            },
                            date,
                            start_time,
                            end_time,
                            location: location.clone(),
                            meeting_url: location
                                .as_ref()
                                .filter(|l| l.starts_with("http"))
                                .cloned(),
                            source: "ical".into(),
                        });
                    }
                }
            }
            continue;
        }
        if !in_event {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once(':') {
            let key_base = k.split(';').next().unwrap_or(k);
            match key_base {
                "UID" => uid = v.to_string(),
                "SUMMARY" => summary = v.to_string(),
                "DTSTART" => dtstart = v.to_string(),
                "DTEND" => dtend = v.to_string(),
                "LOCATION" => {
                    if !v.trim().is_empty() {
                        location = Some(v.replace("\\,", ","));
                    }
                }
                _ => {}
            }
        }
    }

    events.sort_by(|a, b| a.date.cmp(&b.date).then(a.start_time.cmp(&b.start_time)));
    events
}

#[tauri::command]
pub async fn fetch_calendar_events(
    year: Option<i32>,
    month: Option<u32>,
) -> Result<Vec<CalendarEventItem>, String> {
    let now = Local::now();
    let target_year = year.unwrap_or(now.year());
    let target_month = month.unwrap_or(now.month());

    let client = reqwest::Client::new();

    // Priority 1: Google Calendar API v3 OAuth Access Token
    if let Some(access_token) = crate::secrets::read_secret_internal("google_access_token") {
        let next_month = if target_month == 12 {
            (target_year + 1, 1)
        } else {
            (target_year, target_month + 1)
        };
        let time_min = format!("{:04}-{:02}-01T00:00:00Z", target_year, target_month);
        let time_max = format!("{:04}-{:02}-01T00:00:00Z", next_month.0, next_month.1);

        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/primary/events?singleEvents=true&orderBy=startTime&timeMin={}&timeMax={}",
            time_min, time_max
        );

        let resp = client
            .get(&url)
            .bearer_auth(&access_token)
            .send()
            .await
            .map_err(|e| format!("Gagal menghubungi Google Calendar API: {}", e))?;

        if resp.status().is_success() {
            let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let mut out = Vec::new();
            if let Some(items) = json["items"].as_array() {
                for ev in items {
                    let start_dt = ev["start"]["dateTime"]
                        .as_str()
                        .or_else(|| ev["start"]["date"].as_str())
                        .unwrap_or("");
                    let end_dt = ev["end"]["dateTime"].as_str().unwrap_or("");
                    let (date, start_time) = parse_rfc3339_to_local(start_dt).unwrap_or_else(|| {
                        let d = start_dt.get(0..10).unwrap_or("").to_string();
                        (d, "Sepanjang hari".to_string())
                    });
                    let end_time = parse_rfc3339_to_local(end_dt)
                        .map(|(_, t)| t)
                        .unwrap_or_default();
                    out.push(CalendarEventItem {
                        id: ev["id"].as_str().unwrap_or("").to_string(),
                        title: ev["summary"].as_str().unwrap_or("(Tanpa judul)").to_string(),
                        date,
                        start_time,
                        end_time,
                        location: ev["location"].as_str().map(|s| s.to_string()),
                        meeting_url: ev["hangoutLink"]
                            .as_str()
                            .or_else(|| ev["htmlLink"].as_str())
                            .map(|s| s.to_string()),
                        source: "google-api".into(),
                    });
                }
            }
            return Ok(out);
        }
    }

    // Priority 2: Secret Google Calendar iCal (.ics) URL
    if let Some(ical_url) = crate::secrets::read_secret_internal("google_ical_url") {
        let resp = client
            .get(&ical_url)
            .header("User-Agent", "nobi-island/0.1.0")
            .send()
            .await
            .map_err(|e| format!("Gagal mengunduh iCal URL: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("iCal URL error: HTTP {}", resp.status()));
        }
        let body = resp.text().await.map_err(|e| e.to_string())?;
        return Ok(parse_ics_events(&body, target_year, target_month));
    }

    Err("Google Calendar belum terhubung. Tambahkan Secret iCal URL atau Google OAuth Token di Pengaturan.".into())
}
