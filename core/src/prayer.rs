//! Waktu sholat offline (algoritma PrayTimes), metode Kemenag RI: Subuh 20°, Isya 18°, ihtiyat +2 menit.
//! Semua waktu berupa menit sejak tengah malam LOKAL; tanggal dipakai sebagai (tahun, bulan, hari).

const R: f64 = std::f64::consts::PI / 180.0;
fn sin(d: f64) -> f64 { (d * R).sin() }
fn cos(d: f64) -> f64 { (d * R).cos() }
fn tan(d: f64) -> f64 { (d * R).tan() }
fn asin(x: f64) -> f64 { x.asin() / R }
fn acos(x: f64) -> f64 { x.clamp(-1.0, 1.0).acos() / R }
fn atan2(y: f64, x: f64) -> f64 { y.atan2(x) / R }
fn fix_angle(a: f64) -> f64 { ((a % 360.0) + 360.0) % 360.0 }
fn fix_hour(a: f64) -> f64 { ((a % 24.0) + 24.0) % 24.0 }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prayer { Imsak, Fajr, Sunrise, Dhuhr, Asr, Maghrib, Isha }
pub const FARD: [Prayer; 5] = [Prayer::Fajr, Prayer::Dhuhr, Prayer::Asr, Prayer::Maghrib, Prayer::Isha];

impl Prayer {
    pub fn name(self) -> &'static str {
        match self {
            Prayer::Imsak => "Imsak", Prayer::Fajr => "Subuh", Prayer::Sunrise => "Terbit", Prayer::Dhuhr => "Dzuhur",
            Prayer::Asr => "Ashar", Prayer::Maghrib => "Maghrib", Prayer::Isha => "Isya",
        }
    }
}

fn julian(mut y: i64, mut m: i64, d: i64) -> f64 {
    if m <= 2 { y -= 1; m += 12; }
    let a = y.div_euclid(100);
    let b = 2 - a + a.div_euclid(4);
    ((365.25 * (y + 4716) as f64).floor()) + ((30.6001 * (m + 1) as f64).floor()) + d as f64 + b as f64 - 1524.5
}

fn sun_pos(jd: f64) -> (f64, f64) {
    let d = jd - 2451545.0;
    let g = fix_angle(357.529 + 0.98560028 * d);
    let q = fix_angle(280.459 + 0.98564736 * d);
    let l = fix_angle(q + 1.915 * sin(g) + 0.02 * sin(2.0 * g));
    let e = 23.439 - 0.00000036 * d;
    let ra = fix_hour(atan2(cos(e) * sin(l), cos(l)) / 15.0);
    (asin(sin(e) * sin(l)), q / 15.0 - ra) // (deklinasi, persamaan waktu)
}

/// Menit sejak 00:00 lokal untuk [imsak, subuh, terbit, dzuhur, ashar, maghrib, isya].
pub fn times(y: i64, m: i64, d: i64, lat: f64, lon: f64, tz_hours: f64) -> [i32; 7] {
    let jd = julian(y, m, d) - lon / (15.0 * 24.0);
    let mid = |t: f64| fix_hour(12.0 - sun_pos(jd + t).1);
    let angle_time = |angle: f64, t: f64, ccw: bool| {
        let decl = sun_pos(jd + t).0;
        let v = (-sin(angle) - sin(decl) * sin(lat)) / (cos(decl) * cos(lat));
        mid(t) + (acos(v) / 15.0) * if ccw { -1.0 } else { 1.0 }
    };
    let asr_time = |factor: f64, t: f64| {
        let decl = sun_pos(jd + t).0;
        let angle = -(1.0 / (factor + tan((lat - decl).abs()))).atan() / R;
        angle_time(angle, t, false)
    };
    let adj = tz_hours - lon / 15.0;
    let raw = [
        angle_time(20.0, 5.0 / 24.0, true) - 10.0 / 60.0,
        angle_time(20.0, 5.0 / 24.0, true),
        angle_time(0.833, 6.0 / 24.0, true),
        mid(12.0 / 24.0),
        asr_time(1.0, 13.0 / 24.0),
        angle_time(0.833, 18.0 / 24.0, false),
        angle_time(18.0, 18.0 / 24.0, false),
    ];
    let mut out = [0i32; 7];
    for (i, r) in raw.iter().enumerate() {
        let ihtiyat = if i == 2 { -2.0 } else { 2.0 };
        let h = fix_hour(r + adj) + ihtiyat / 60.0;
        out[i] = (h * 60.0).round() as i32;
    }
    out
}

/* ---- tanggal sipil (tanpa pustaka waktu) ---- */

pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}
pub fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[derive(Debug, Clone, Copy)]
pub struct Next { pub prayer: Prayer, pub at_local_secs: i64, pub left_secs: i64 }

/// `now_local_secs` = detik sejak 1970-01-01 00:00 menurut jam dinding lokal (bukan UTC).
pub fn next_prayer(now_local_secs: i64, lat: f64, lon: f64, tz_hours: f64) -> Next {
    let today = now_local_secs.div_euclid(86400);
    for add in 0..3 {
        let day = today + add;
        let (y, m, d) = civil_from_days(day);
        let t = times(y, m, d, lat, lon, tz_hours);
        for p in FARD {
            let idx = match p { Prayer::Fajr => 1, Prayer::Dhuhr => 3, Prayer::Asr => 4, Prayer::Maghrib => 5, _ => 6 };
            let at = day * 86400 + t[idx] as i64 * 60;
            if at > now_local_secs {
                return Next { prayer: p, at_local_secs: at, left_secs: at - now_local_secs };
            }
        }
    }
    unreachable!("selalu ada sholat dalam 3 hari")
}

pub const CITIES: &[(&str, f64, f64)] = &[
    ("Banda Aceh", 5.5483, 95.3238), ("Medan", 3.5952, 98.6722), ("Padang", -0.9471, 100.4172), ("Pekanbaru", 0.5071, 101.4478),
    ("Batam", 1.0456, 104.0305), ("Jambi", -1.6101, 103.6131), ("Palembang", -2.9761, 104.7754), ("Bengkulu", -3.8004, 102.2655),
    ("Bandar Lampung", -5.4292, 105.2619), ("Jakarta", -6.2088, 106.8456), ("Tangerang", -6.1783, 106.63), ("Bekasi", -6.2383, 106.9756),
    ("Depok", -6.4025, 106.7942), ("Bogor", -6.5971, 106.806), ("Bandung", -6.9175, 107.6191), ("Cirebon", -6.7063, 108.557),
    ("Semarang", -6.9667, 110.4167), ("Yogyakarta", -7.7956, 110.3695), ("Solo", -7.5666, 110.8167), ("Surabaya", -7.2575, 112.7521),
    ("Malang", -7.9666, 112.6326), ("Serang", -6.1201, 106.1503), ("Denpasar", -8.65, 115.2167), ("Mataram", -8.5833, 116.1167),
    ("Kupang", -10.1772, 123.607), ("Pontianak", -0.0263, 109.3425), ("Palangka Raya", -2.2161, 113.9135), ("Banjarmasin", -3.3194, 114.5908),
    ("Balikpapan", -1.2379, 116.8529), ("Samarinda", -0.5022, 117.1536), ("Manado", 1.4748, 124.8421), ("Palu", -0.9003, 119.8779),
    ("Makassar", -5.1477, 119.4327), ("Kendari", -3.9985, 122.5129), ("Gorontalo", 0.5435, 123.0568), ("Ambon", -3.6954, 128.1814),
    ("Ternate", 0.7833, 127.3667), ("Jayapura", -2.5337, 140.7181), ("Sorong", -0.8762, 131.2558),
];

#[cfg(test)]
mod tests {
    use super::*;
    fn hm(m: i32) -> String { format!("{:02}:{:02}", m / 60, m % 60) }

    #[test]
    fn bandung_21_nov_2026_sama_dengan_versi_typescript() {
        let t = times(2026, 11, 21, -6.9175, 107.6191, 7.0);
        let got: Vec<String> = t.iter().map(|m| hm(*m)).collect();
        assert_eq!(got, ["03:50", "04:00", "05:20", "11:37", "15:01", "17:51", "19:06"]);
    }
    #[test]
    fn urutan_wajar_di_kota_besar() {
        for (n, lat, lon) in CITIES {
            let tz = if *lon < 114.0 { 7.0 } else if *lon < 135.0 { 8.0 } else { 9.0 };
            let t = times(2026, 6, 15, *lat, *lon, tz);
            assert!(t.windows(2).all(|w| w[0] < w[1]), "{n}: {t:?}");
        }
    }
    #[test]
    fn civil_roundtrip() {
        for z in [-1000i64, 0, 1, 20000, 20778] {
            let (y, m, d) = civil_from_days(z);
            assert_eq!(days_from_civil(y, m, d), z);
        }
        assert_eq!(civil_from_days(0), (1970, 1, 1));
    }
    #[test]
    fn next_prayer_lompat_ke_besok() {
        let day = days_from_civil(2026, 11, 21);
        let now = day * 86400 + 20 * 3600; // 20:00, sesudah Isya
        let n = next_prayer(now, -6.9175, 107.6191, 7.0);
        assert_eq!(n.prayer, Prayer::Fajr);
        assert!(n.at_local_secs > day * 86400 + 86400);
        let n2 = next_prayer(day * 86400 + 10 * 3600, -6.9175, 107.6191, 7.0);
        assert_eq!(n2.prayer, Prayer::Dhuhr);
    }
}
