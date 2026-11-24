//! Statistik sistem: hitungan CPU dari selisih counter, riwayat untuk grafik, dan pemformat.
use std::collections::VecDeque;

/// Hitung persen CPU (0..1) dari counter kumulatif (idle, total) antar dua pembacaan.
#[derive(Default, Clone, Debug)]
pub struct CpuSampler { last: Option<(u64, u64)> }

impl CpuSampler {
    pub fn update(&mut self, idle: u64, total: u64) -> f32 {
        let r = match self.last {
            Some((li, lt)) if total > lt => {
                let (di, dt) = (idle.saturating_sub(li) as f64, (total - lt) as f64);
                (1.0 - di / dt).clamp(0.0, 1.0) as f32
            }
            _ => 0.0,
        };
        self.last = Some((idle, total));
        r
    }
}

/// Buffer melingkar untuk sparkline.
#[derive(Clone, Debug)]
pub struct History { buf: VecDeque<f32>, cap: usize }

impl History {
    pub fn new(cap: usize) -> Self { History { buf: VecDeque::with_capacity(cap), cap: cap.max(2) } }
    pub fn push(&mut self, v: f32) { if self.buf.len() == self.cap { self.buf.pop_front(); } self.buf.push_back(v); }
    pub fn len(&self) -> usize { self.buf.len() }
    pub fn is_empty(&self) -> bool { self.buf.is_empty() }
    pub fn last(&self) -> f32 { self.buf.back().copied().unwrap_or(0.0) }
    pub fn values(&self) -> Vec<f32> { self.buf.iter().copied().collect() }

    /// Perintah path SVG untuk garis (viewbox w x h, nilai 0..1, 1 = paling atas). Diisi dari kiri.
    pub fn svg_line(&self, w: f32, h: f32) -> String {
        path_points(&self.pad(), w, h, false)
    }
    /// Sama seperti `svg_line` tetapi ditutup ke dasar untuk isian gradien.
    pub fn svg_area(&self, w: f32, h: f32) -> String {
        path_points(&self.pad(), w, h, true)
    }
    fn pad(&self) -> Vec<f32> {
        let mut v = vec![0.0; self.cap.saturating_sub(self.buf.len())];
        v.extend(self.buf.iter().copied());
        v
    }
}

fn path_points(v: &[f32], w: f32, h: f32, close: bool) -> String {
    if v.len() < 2 { return String::new(); }
    let step = w / (v.len() - 1) as f32;
    let mut s = String::new();
    for (i, x) in v.iter().enumerate() {
        let (px, py) = (i as f32 * step, h - x.clamp(0.0, 1.0) * h);
        s.push_str(if i == 0 { "M " } else { " L " });
        s.push_str(&format!("{:.1} {:.1}", px, py));
    }
    if close { s.push_str(&format!(" L {:.1} {:.1} L 0 {:.1} Z", w, h, h)); }
    s
}

pub fn fmt_uptime(secs: u64) -> String {
    let (d, h, m) = (secs / 86400, secs % 86400 / 3600, secs % 3600 / 60);
    if d > 0 { format!("{d}h {h}j") } else if h > 0 { format!("{h}j {m:02}m") } else { format!("{m}m") }
}

pub fn fmt_gb(bytes: u64) -> String { format!("{:.1} GB", bytes as f64 / 1_073_741_824.0) }

pub fn fmt_rate(bytes_per_s: f64) -> String {
    if bytes_per_s >= 1_048_576.0 { format!("{:.1} MB/s", bytes_per_s / 1_048_576.0) } else { format!("{:.0} KB/s", bytes_per_s / 1024.0) }
}

/// Keluaran `MSAcpi_ThermalZoneTemperature.CurrentTemperature` (sepersepuluh Kelvin) -> °C. Mengambil nilai tertinggi bila banyak zona.
pub fn parse_thermal(out: &str) -> Option<f32> {
    out.split(|c: char| !(c.is_ascii_digit() || c == '.')).filter_map(|t| t.parse::<f32>().ok()).filter(|v| *v > 2000.0 && *v < 4000.0)
        .map(|k| k / 10.0 - 273.15).fold(None, |m: Option<f32>, c| Some(m.map_or(c, |x| x.max(c))))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cpu_percent() {
        let mut c = CpuSampler::default();
        assert_eq!(c.update(100, 200), 0.0);
        let p = c.update(150, 300); // 50 idle dari 100 total
        assert!((p - 0.5).abs() < 1e-6);
        assert_eq!(c.update(150, 300), 0.0); // tak ada selisih -> aman
    }
    #[test]
    fn history_ring_and_paths() {
        let mut h = History::new(4);
        for v in [0.1, 0.2, 0.3, 0.4, 0.5] { h.push(v); }
        assert_eq!(h.values(), vec![0.2, 0.3, 0.4, 0.5]);
        let l = h.svg_line(90.0, 30.0);
        assert!(l.starts_with("M 0.0 ") && l.matches('L').count() == 3);
        assert!(h.svg_area(90.0, 30.0).ends_with('Z'));
        assert!(History::new(4).svg_line(10.0, 10.0).starts_with("M")); // dipad nol
    }
    #[test]
    fn formatters() {
        assert_eq!(fmt_uptime(90_000), "1h 1j");
        assert_eq!(fmt_uptime(3 * 3600 + 5 * 60), "3j 05m");
        assert_eq!(fmt_gb(8 * 1_073_741_824), "8.0 GB");
        assert_eq!(fmt_rate(2_097_152.0), "2.0 MB/s");
        assert_eq!(fmt_rate(51_200.0), "50 KB/s");
    }
    #[test]
    fn thermal_parse() {
        let t = parse_thermal("3182\r\n3232\r\n").unwrap();
        assert!((t - 50.05).abs() < 0.1, "{t}");
        assert_eq!(parse_thermal("tidak ada"), None);
    }
}
