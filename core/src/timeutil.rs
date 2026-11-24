//! Utilitas waktu tanpa pustaka: RFC 3339, hari dalam pekan, nama bulan/hari Indonesia, kunci tanggal lokal.
use crate::prayer::{civil_from_days, days_from_civil};

pub const MONTHS_ID: [&str; 12] = ["Januari", "Februari", "Maret", "April", "Mei", "Juni", "Juli", "Agustus", "September", "Oktober", "November", "Desember"];
pub const MONTHS_SHORT: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "Mei", "Jun", "Jul", "Agu", "Sep", "Okt", "Nov", "Des"];
pub const DAYS_ID: [&str; 7] = ["Senin", "Selasa", "Rabu", "Kamis", "Jumat", "Sabtu", "Minggu"];
pub const DAYS_SHORT: [&str; 7] = ["Sen", "Sel", "Rab", "Kam", "Jum", "Sab", "Min"];

/// Hari dalam pekan untuk jumlah hari sejak 1970-01-01: Senin = 0 ... Minggu = 6.
pub fn weekday(days_since_epoch: i64) -> usize { (days_since_epoch + 3).rem_euclid(7) as usize }

/// "2026-11-23T19:02:05+07:00" / "...Z" / dengan pecahan detik -> detik epoch UTC.
pub fn parse_rfc3339(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    if b.len() < 19 { return None; }
    let n = |a: usize, e: usize| s.get(a..e)?.parse::<i64>().ok();
    if b[4] != b'-' || b[7] != b'-' || !(b[10] == b'T' || b[10] == b't' || b[10] == b' ') { return None; }
    let (y, mo, d, h, mi, sec) = (n(0, 4)?, n(5, 7)?, n(8, 10)?, n(11, 13)?, n(14, 16)?, n(17, 19)?);
    let mut i = 19;
    if b.get(i) == Some(&b'.') { i += 1; while i < b.len() && b[i].is_ascii_digit() { i += 1; } }
    let off = match b.get(i) {
        None | Some(b'Z') | Some(b'z') => 0,
        Some(&c) if c == b'+' || c == b'-' => {
            let oh = n(i + 1, i + 3)?; let om = n(i + 4, i + 6).unwrap_or(0);
            (oh * 3600 + om * 60) * if c == b'-' { -1 } else { 1 }
        }
        _ => return None,
    };
    Some(days_from_civil(y, mo, d) * 86400 + h * 3600 + mi * 60 + sec - off)
}

/// Detik epoch UTC -> "2026-11-23T12:02:05Z".
pub fn fmt_rfc3339(secs: i64) -> String {
    let (y, m, d) = civil_from_days(secs.div_euclid(86400));
    let t = secs.rem_euclid(86400);
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", t / 3600, t % 3600 / 60, t % 60)
}

/// Detik epoch UTC -> hari lokal (jumlah hari sejak epoch menurut jam dinding lokal).
pub fn local_day(utc_secs: i64, tz_hours: f64) -> i64 { (utc_secs + (tz_hours * 3600.0) as i64).div_euclid(86400) }

/// "2026-11-23" untuk hari lokal tersebut.
pub fn date_key(local_day: i64) -> String { let (y, m, d) = civil_from_days(local_day); format!("{y:04}-{m:02}-{d:02}") }

/// "HH:MM" dari detik epoch UTC pada zona waktu `tz_hours`.
pub fn hhmm(utc_secs: i64, tz_hours: f64) -> String {
    let t = (utc_secs + (tz_hours * 3600.0) as i64).rem_euclid(86400);
    format!("{:02}:{:02}", t / 3600, t % 3600 / 60)
}

/// "Sen, 23 Nov" untuk hari lokal.
pub fn day_label(local_day: i64) -> String {
    let (_, m, d) = civil_from_days(local_day);
    format!("{}, {} {}", DAYS_SHORT[weekday(local_day)], d, MONTHS_SHORT[(m - 1) as usize])
}

/// Format durasi pendek: "2j 05m", "12m", "45d".
pub fn fmt_duration_short(secs: i64) -> String {
    let s = secs.max(0);
    if s >= 86400 { format!("{}h {}j", s / 86400, s % 86400 / 3600) }
    else if s >= 3600 { format!("{}j {:02}m", s / 3600, s % 3600 / 60) }
    else if s >= 60 { format!("{}m", s / 60) }
    else { format!("{}d", s) }
}

/// "baru saja", "5 mnt", "3 j", "2 h" untuk selisih milidetik.
pub fn ago(now_ms: i64, then_ms: i64) -> String {
    let s = (now_ms - then_ms).max(0) / 1000;
    if s < 60 { "baru saja".into() } else if s < 3600 { format!("{} mnt", s / 60) } else if s < 86400 { format!("{} j", s / 3600) } else { format!("{} h", s / 86400) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rfc3339_variants() {
        let z = parse_rfc3339("2026-11-23T12:02:05Z").unwrap();
        assert_eq!(parse_rfc3339("2026-11-23T19:02:05+07:00"), Some(z));
        assert_eq!(parse_rfc3339("2026-11-23T19:02:05.123+07:00"), Some(z));
        assert_eq!(parse_rfc3339("2026-11-23T05:02:05-07:00"), Some(z));
        assert_eq!(fmt_rfc3339(z), "2026-11-23T12:02:05Z");
        assert_eq!(parse_rfc3339("kemarin"), None);
        assert_eq!(parse_rfc3339("2026-11-23"), None);
    }
    #[test]
    fn weekday_known_dates() {
        assert_eq!(weekday(days_from_civil(1970, 1, 1)), 3); // Kamis
        assert_eq!(weekday(days_from_civil(2026, 11, 23)), 0); // Senin
        assert_eq!(DAYS_ID[weekday(days_from_civil(2026, 11, 29))], "Minggu");
    }
    #[test]
    fn labels() {
        let z = parse_rfc3339("2026-11-23T12:02:05Z").unwrap();
        assert_eq!(hhmm(z, 7.0), "19:02");
        assert_eq!(date_key(local_day(z, 7.0)), "2026-11-23");
        assert_eq!(day_label(local_day(z, 7.0)), "Sen, 23 Nov");
        assert_eq!(fmt_duration_short(7500), "2j 05m");
        assert_eq!(ago(100_000, 70_000), "baru saja");
        assert_eq!(ago(10_000_000, 7_000_000), "50 mnt");
    }
}
