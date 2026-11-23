//! Parser iCal (secret iCal Google Calendar) sederhana: VEVENT dengan DTSTART/DTEND/SUMMARY. RRULE harian/mingguan/bulanan/tahunan didukung dasar.
use crate::prayer::{civil_from_days, days_from_civil};

#[derive(Debug, Clone, PartialEq)]
pub struct Event { pub title: String, pub start: i64, pub end: i64, pub all_day: bool }

fn unfold(src: &str) -> Vec<String> {
    let mut out: Vec<String> = vec![];
    for l in src.lines() {
        let l = l.trim_end_matches('\r');
        if (l.starts_with(' ') || l.starts_with('\t')) && !out.is_empty() { out.last_mut().unwrap().push_str(&l[1..]); } else { out.push(l.to_string()); }
    }
    out
}

/// "20261121T190000Z" / "20261121T190000" / "20261121" -> detik epoch UTC (waktu lokal dianggap offset `tz` jam).
fn parse_dt(v: &str, tz: f64) -> Option<(i64, bool)> {
    let v = v.trim();
    let num = |a: usize, b: usize| v.get(a..b)?.parse::<i64>().ok();
    let (y, m, d) = (num(0, 4)?, num(4, 6)?, num(6, 8)?);
    let day = days_from_civil(y, m, d) * 86400;
    if v.len() < 15 { return Some((day - (tz * 3600.0) as i64, true)); }
    let (h, mi, s) = (num(9, 11)?, num(11, 13)?, num(13, 15)?);
    let secs = day + h * 3600 + mi * 60 + s;
    Some((if v.ends_with('Z') { secs } else { secs - (tz * 3600.0) as i64 }, false))
}

fn dim(y: i64, m: i64) -> i64 {
    match m { 2 => if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 { 29 } else { 28 }, 4 | 6 | 9 | 11 => 30, _ => 31 }
}

/// Geser `s` sebanyak k bulan, tanggal tetap; None bila tanggal tak ada di bulan itu (mis. 31 Feb).
fn add_months(s: i64, k: i64) -> Option<i64> {
    let (day, tod) = (s.div_euclid(86400), s.rem_euclid(86400));
    let (y, m, d) = civil_from_days(day);
    let t = y * 12 + (m - 1) + k;
    let (ny, nm) = (t.div_euclid(12), t.rem_euclid(12) + 1);
    if d > dim(ny, nm) { None } else { Some(days_from_civil(ny, nm, d) * 86400 + tod) }
}

/// Kejadian pertama (dari pola bulanan/tahunan) yang belum berakhir pada `now`.
fn next_monthly(s: i64, dur: i64, now: i64, step: i64) -> i64 {
    let span = if step == 1 { 31 * 86400 } else { 366 * 86400 };
    let k0 = ((now - s - dur) / span - 1).max(0) / step * step;
    let mut k = k0;
    while k < k0 + 400 * step {
        if let Some(c) = add_months(s, k) { if c + dur > now { return c; } }
        k += step;
    }
    s
}

fn unescape(s: &str) -> String { s.replace("\\n", " ").replace("\\,", ",").replace("\\;", ";").replace("\\\\", "\\") }

/// Kembalikan kejadian yang berakhir setelah `now_utc` dan mulai sebelum `now_utc + horizon_s`, urut waktu.
pub fn upcoming(src: &str, now_utc: i64, horizon_s: i64, tz: f64) -> Vec<Event> {
    let mut out = vec![];
    let (mut inside, mut title, mut st, mut en, mut rr): (bool, String, Option<(i64, bool)>, Option<(i64, bool)>, String) = (false, String::new(), None, None, String::new());
    for l in unfold(src) {
        if l == "BEGIN:VEVENT" { inside = true; title.clear(); st = None; en = None; rr.clear(); continue; }
        if l == "END:VEVENT" {
            inside = false;
            if let Some((s, all)) = st {
                let dur = en.map(|(e, _)| (e - s).max(0)).unwrap_or(if all { 86400 } else { 3600 });
                let step = if rr.contains("FREQ=DAILY") { 86400 } else if rr.contains("FREQ=WEEKLY") { 7 * 86400 } else { 0 };
                let mut s0 = s;
                if step > 0 && s0 + dur < now_utc { s0 += ((now_utc - s0 - dur) / step + 1) * step; }
                if rr.contains("FREQ=MONTHLY") && s0 + dur < now_utc { s0 = next_monthly(s, dur, now_utc, 1); }
                if rr.contains("FREQ=YEARLY") && s0 + dur < now_utc { s0 = next_monthly(s, dur, now_utc, 12); }
                if s0 + dur > now_utc && s0 < now_utc + horizon_s { out.push(Event { title: title.clone(), start: s0, end: s0 + dur, all_day: all }); }
            }
            continue;
        }
        if !inside { continue; }
        let Some((k, v)) = l.split_once(':') else { continue };
        let key = k.split(';').next().unwrap_or("");
        match key {
            "SUMMARY" => title = unescape(v),
            "DTSTART" => st = parse_dt(v, tz),
            "DTEND" => en = parse_dt(v, tz),
            "RRULE" => rr = v.to_string(),
            _ => {}
        }
    }
    out.sort_by_key(|e| e.start);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic() {
        let ics = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nDTSTART:20261123T030000Z\r\nDTEND:20261123T040000Z\r\nSUMMARY:Rapat\\, tim\r\n panjang\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nDTSTART;VALUE=DATE:20200101\r\nSUMMARY:Lama\r\nEND:VEVENT\r\nEND:VCALENDAR";
        let now = days_from_civil(2026, 11, 23) * 86400;
        let ev = upcoming(ics, now, 86400, 7.0);
        assert_eq!(ev.len(), 1);
        assert_eq!(ev[0].title, "Rapat, timpanjang");
        assert_eq!(ev[0].start, now + 3 * 3600);
    }
    #[test]
    fn monthly_yearly() {
        let m = "BEGIN:VEVENT\nDTSTART:20260131T030000Z\nDTEND:20260131T040000Z\nRRULE:FREQ=MONTHLY\nSUMMARY:Bulanan\nEND:VEVENT";
        let now = days_from_civil(2026, 11, 23) * 86400;
        assert_eq!(upcoming(m, now, 60 * 86400, 7.0)[0].start, days_from_civil(2026, 12, 31) * 86400 + 3 * 3600); // Nov tidak punya tgl 31
        let y = "BEGIN:VEVENT\nDTSTART:20200229T030000Z\nDTEND:20200229T040000Z\nRRULE:FREQ=YEARLY\nSUMMARY:Kabisat\nEND:VEVENT";
        let ev = upcoming(y, now, 2000 * 86400, 7.0);
        assert_eq!(ev[0].start, days_from_civil(2028, 2, 29) * 86400 + 3 * 3600);
    }
    #[test]
    fn weekly() {
        let ics = "BEGIN:VEVENT\nDTSTART:20261101T030000Z\nDTEND:20261101T040000Z\nRRULE:FREQ=WEEKLY\nSUMMARY:Mingguan\nEND:VEVENT";
        let now = days_from_civil(2026, 11, 23) * 86400 + 5 * 3600;
        let ev = upcoming(ics, now, 7 * 86400, 7.0);
        assert_eq!(ev[0].start, days_from_civil(2026, 11, 29) * 86400 + 3 * 3600);
    }
}
