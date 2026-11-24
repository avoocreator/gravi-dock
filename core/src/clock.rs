//! Stopwatch, timer hitung mundur, dan alarm. Semuanya berbasis waktu dinding (ms epoch) agar tahan terhadap tick yang tertunda.
use crate::json::{self, Value};

/* ---------------- Stopwatch ---------------- */

#[derive(Clone, Debug, Default)]
pub struct Stopwatch { pub running: bool, start_ms: i64, acc_ms: i64, pub laps: Vec<i64> }

impl Stopwatch {
    pub fn elapsed(&self, now_ms: i64) -> i64 { self.acc_ms + if self.running { (now_ms - self.start_ms).max(0) } else { 0 } }
    pub fn toggle(&mut self, now_ms: i64) {
        if self.running { self.acc_ms = self.elapsed(now_ms); self.running = false; } else { self.start_ms = now_ms; self.running = true; }
    }
    /// Catat lap (waktu total saat ini); mengembalikan indeks lap.
    pub fn lap(&mut self, now_ms: i64) -> Option<usize> {
        if !self.running { return None; }
        self.laps.push(self.elapsed(now_ms));
        Some(self.laps.len() - 1)
    }
    pub fn reset(&mut self) { *self = Stopwatch::default(); }
    pub fn is_zero(&self, now_ms: i64) -> bool { self.elapsed(now_ms) == 0 && self.laps.is_empty() }
}

/// "MM:SS.cc" atau "H:MM:SS.cc".
pub fn fmt_stopwatch(ms: i64) -> String {
    let ms = ms.max(0);
    let (cs, s) = ((ms % 1000) / 10, ms / 1000);
    let (h, m, sec) = (s / 3600, s % 3600 / 60, s % 60);
    if h > 0 { format!("{h}:{m:02}:{sec:02}.{cs:02}") } else { format!("{m:02}:{sec:02}.{cs:02}") }
}

/// "MM:SS" atau "H:MM:SS" dari detik.
pub fn fmt_hms(secs: i64) -> String {
    let s = secs.max(0);
    if s >= 3600 { format!("{}:{:02}:{:02}", s / 3600, s % 3600 / 60, s % 60) } else { format!("{:02}:{:02}", s / 60, s % 60) }
}

/* ---------------- Timer hitung mundur ---------------- */

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerState { Idle, Running, Paused, Finished }

#[derive(Clone, Debug)]
pub struct Countdown { pub total_s: i64, ends_ms: i64, paused_left_ms: i64, pub state: TimerState }

impl Default for Countdown { fn default() -> Self { Countdown { total_s: 300, ends_ms: 0, paused_left_ms: 0, state: TimerState::Idle } } }

impl Countdown {
    pub fn set(&mut self, secs: i64) { if self.state == TimerState::Idle || self.state == TimerState::Finished { self.total_s = secs.clamp(1, 99 * 3600); self.state = TimerState::Idle; } }
    pub fn add(&mut self, secs: i64) { self.set((self.total_s + secs).max(1)); }
    pub fn start(&mut self, now_ms: i64) {
        match self.state {
            TimerState::Idle | TimerState::Finished => { self.ends_ms = now_ms + self.total_s * 1000; self.state = TimerState::Running; }
            TimerState::Paused => { self.ends_ms = now_ms + self.paused_left_ms; self.state = TimerState::Running; }
            TimerState::Running => {}
        }
    }
    pub fn pause(&mut self, now_ms: i64) { if self.state == TimerState::Running { self.paused_left_ms = (self.ends_ms - now_ms).max(0); self.state = TimerState::Paused; } }
    pub fn toggle(&mut self, now_ms: i64) { if self.state == TimerState::Running { self.pause(now_ms) } else { self.start(now_ms) } }
    pub fn reset(&mut self) { self.state = TimerState::Idle; }
    pub fn left_ms(&self, now_ms: i64) -> i64 {
        match self.state { TimerState::Idle | TimerState::Finished => if self.state == TimerState::Idle { self.total_s * 1000 } else { 0 }, TimerState::Paused => self.paused_left_ms, TimerState::Running => (self.ends_ms - now_ms).max(0) }
    }
    pub fn progress(&self, now_ms: i64) -> f32 { (self.left_ms(now_ms) as f32 / (self.total_s.max(1) * 1000) as f32).clamp(0.0, 1.0) }
    /// True tepat sekali saat timer habis.
    pub fn tick(&mut self, now_ms: i64) -> bool {
        if self.state == TimerState::Running && now_ms >= self.ends_ms { self.state = TimerState::Finished; true } else { false }
    }
}

/* ---------------- Alarm ---------------- */

/// `days`: bit 0 = Senin ... bit 6 = Minggu; 0 = sekali saja (berikutnya yang terdekat), 0x7f = setiap hari.
#[derive(Clone, Debug, PartialEq)]
pub struct Alarm { pub id: u32, pub hour: u8, pub min: u8, pub days: u8, pub label: String, pub enabled: bool, pub last_fired_day: i64 }

impl Alarm {
    pub fn time_label(&self) -> String { format!("{:02}:{:02}", self.hour, self.min) }
    pub fn repeat_label(&self) -> String {
        match self.days {
            0 => "Sekali".into(),
            0x7f => "Setiap hari".into(),
            0b0011111 => "Senin–Jumat".into(),
            d => crate::timeutil::DAYS_SHORT.iter().enumerate().filter(|(i, _)| d & (1 << i) != 0).map(|(_, n)| *n).collect::<Vec<_>>().join(" "),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Alarms { pub list: Vec<Alarm>, next_id: u32 }

impl Alarms {
    pub fn add(&mut self, hour: u8, min: u8, days: u8, label: &str) -> u32 {
        self.next_id += 1;
        self.list.push(Alarm { id: self.next_id, hour: hour.min(23), min: min.min(59), days: days & 0x7f, label: label.to_string(), enabled: true, last_fired_day: i64::MIN });
        self.list.sort_by_key(|a| (a.hour, a.min));
        self.next_id
    }
    pub fn remove(&mut self, id: u32) { self.list.retain(|a| a.id != id); }
    pub fn toggle(&mut self, id: u32) { if let Some(a) = self.list.iter_mut().find(|a| a.id == id) { a.enabled = !a.enabled; } }

    /// `local_secs` = detik sejak epoch menurut jam dinding lokal. Mengembalikan alarm yang baru berbunyi (sekali per hari per alarm).
    /// Alarm sekali-saja dimatikan setelah berbunyi.
    pub fn due(&mut self, local_secs: i64) -> Vec<Alarm> {
        let day = local_secs.div_euclid(86400);
        let tod = local_secs.rem_euclid(86400);
        let wd = crate::timeutil::weekday(day);
        let mut fired = vec![];
        for a in self.list.iter_mut() {
            if !a.enabled || a.last_fired_day == day { continue; }
            let at = a.hour as i64 * 3600 + a.min as i64 * 60;
            // berbunyi dalam jendela 2 menit supaya tidak terlewat bila aplikasi sempat tertidur sebentar
            if tod >= at && tod < at + 120 && (a.days == 0 || a.days & (1 << wd) != 0) {
                a.last_fired_day = day;
                if a.days == 0 { a.enabled = false; }
                fired.push(a.clone());
            }
        }
        fired
    }

    /// Alarm aktif berikutnya: (alarm, detik lagi).
    pub fn next(&self, local_secs: i64) -> Option<(&Alarm, i64)> {
        let day = local_secs.div_euclid(86400);
        let tod = local_secs.rem_euclid(86400);
        let mut best: Option<(&Alarm, i64)> = None;
        for a in self.list.iter().filter(|a| a.enabled) {
            let at = a.hour as i64 * 3600 + a.min as i64 * 60;
            for add in 0..8i64 {
                let d = day + add;
                let ok = a.days == 0 || a.days & (1 << crate::timeutil::weekday(d)) != 0;
                let left = add * 86400 + at - tod;
                if ok && left > 0 { if best.map_or(true, |(_, b)| left < b) { best = Some((a, left)); } break; }
            }
        }
        best
    }

    pub fn to_json(&self) -> String {
        Value::Arr(self.list.iter().map(|a| Value::obj(vec![
            ("id", Value::n(a.id as f64)), ("h", Value::n(a.hour as f64)), ("m", Value::n(a.min as f64)), ("days", Value::n(a.days as f64)),
            ("label", Value::s(&a.label)), ("on", Value::b(a.enabled)),
        ])).collect()).pretty()
    }
    pub fn parse(src: &str) -> Alarms {
        let mut out = Alarms::default();
        if let Some(v) = json::parse(src) {
            for x in v.arr() {
                let id = out.add(x.get("h").num().unwrap_or(7.0) as u8, x.get("m").num().unwrap_or(0.0) as u8, x.get("days").num().unwrap_or(0.0) as u8, x.get("label").text());
                if !x.get("on").truthy() { out.toggle(id); }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prayer::days_from_civil;

    #[test]
    fn stopwatch_flow() {
        let mut s = Stopwatch::default();
        s.toggle(1000);
        assert_eq!(s.elapsed(3500), 2500);
        assert_eq!(s.lap(3500), Some(0));
        s.toggle(4000);
        assert_eq!(s.elapsed(99999), 3000);
        s.toggle(10_000);
        assert_eq!(s.elapsed(11_000), 4000);
        s.reset();
        assert!(s.is_zero(0));
        assert_eq!(fmt_stopwatch(61_230), "01:01.23");
        assert_eq!(fmt_stopwatch(3_661_000), "1:01:01.00");
    }
    #[test]
    fn countdown_flow() {
        let mut c = Countdown::default();
        c.set(60);
        c.start(0);
        assert_eq!(c.left_ms(10_000), 50_000);
        c.pause(10_000);
        assert_eq!(c.left_ms(999_999), 50_000);
        c.start(20_000);
        assert!(!c.tick(60_000));
        assert!(c.tick(70_000));
        assert!(!c.tick(80_000));
        assert_eq!(c.state, TimerState::Finished);
        c.set(30);
        assert_eq!((c.state, c.total_s), (TimerState::Idle, 30));
        assert_eq!(fmt_hms(3725), "1:02:05");
    }
    #[test]
    fn alarm_fires_once_per_day_and_respects_weekdays() {
        let mut a = Alarms::default();
        a.add(5, 0, 0b0011111, "Subuh"); // Senin-Jumat
        a.add(8, 30, 0, "Sekali");
        let mon = days_from_civil(2026, 11, 23) * 86400; // Senin
        let sat = days_from_civil(2026, 11, 28) * 86400;
        assert_eq!(a.due(mon + 5 * 3600 + 10).len(), 1);
        assert_eq!(a.due(mon + 5 * 3600 + 20).len(), 0, "tidak boleh dobel");
        assert_eq!(a.due(sat + 5 * 3600 + 10).len(), 0, "Sabtu tidak aktif");
        assert_eq!(a.due(mon + 8 * 3600 + 30 * 60 + 5).len(), 1);
        assert!(!a.list.iter().find(|x| x.label == "Sekali").unwrap().enabled, "sekali-saja otomatis mati");
    }
    #[test]
    fn alarm_next_and_persistence() {
        let mut a = Alarms::default();
        let id = a.add(6, 0, 0x7f, "Pagi");
        a.add(22, 0, 0x7f, "Malam");
        let mon = days_from_civil(2026, 11, 23) * 86400;
        let (n, left) = a.next(mon + 7 * 3600).unwrap();
        assert_eq!((n.label.as_str(), left), ("Malam", 15 * 3600));
        a.toggle(id);
        let b = Alarms::parse(&a.to_json());
        assert_eq!(b.list.len(), 2);
        assert!(!b.list.iter().find(|x| x.label == "Pagi").unwrap().enabled);
        assert_eq!(b.list[0].repeat_label(), "Setiap hari");
    }
}
