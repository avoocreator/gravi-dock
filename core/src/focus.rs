//! Pomodoro berbasis waktu dinding (tahan terhadap timer UI yang tertunda). Mendukung jeda panjang tiap N putaran.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase { Idle, Focus, Break, LongBreak }

#[derive(Clone, Debug)]
pub struct Focus {
    pub phase: Phase,
    pub ends_at: i64,
    pub paused_left: Option<i64>,
    pub focus_s: i64,
    pub break_s: i64,
    pub long_s: i64,
    pub rounds: u32,
    pub auto_next: bool,
    /// Putaran fokus yang sudah selesai (total).
    pub done: u32,
}

impl Focus {
    pub fn new(focus_min: i64, break_min: i64) -> Self {
        Focus { phase: Phase::Idle, ends_at: 0, paused_left: None, focus_s: focus_min * 60, break_s: break_min * 60, long_s: 15 * 60, rounds: 4, auto_next: true, done: 0 }
    }
    pub fn configure(&mut self, focus_min: i64, break_min: i64, long_min: i64, rounds: i64, auto_next: bool) {
        self.focus_s = focus_min.max(1) * 60; self.break_s = break_min.max(1) * 60; self.long_s = long_min.max(1) * 60;
        self.rounds = rounds.max(1) as u32; self.auto_next = auto_next;
    }
    pub fn start(&mut self, now: i64) { self.phase = Phase::Focus; self.ends_at = now + self.focus_s; self.paused_left = None; }
    pub fn stop(&mut self) { self.phase = Phase::Idle; self.paused_left = None; }
    pub fn running(&self) -> bool { self.phase != Phase::Idle && self.paused_left.is_none() }
    pub fn paused(&self) -> bool { self.paused_left.is_some() }
    pub fn phase_len(&self) -> i64 {
        match self.phase { Phase::Idle | Phase::Focus => self.focus_s, Phase::Break => self.break_s, Phase::LongBreak => self.long_s }
    }
    pub fn left(&self, now: i64) -> i64 {
        match (self.phase, self.paused_left) { (Phase::Idle, _) => self.focus_s, (_, Some(l)) => l, _ => (self.ends_at - now).max(0) }
    }
    /// 0..1 sisa waktu pada fase ini (untuk cincin progres).
    pub fn progress(&self, now: i64) -> f32 {
        let len = self.phase_len().max(1) as f32;
        (self.left(now) as f32 / len).clamp(0.0, 1.0)
    }
    pub fn toggle_pause(&mut self, now: i64) {
        if self.phase == Phase::Idle { return; }
        match self.paused_left { Some(l) => { self.ends_at = now + l; self.paused_left = None } None => self.paused_left = Some(self.left(now)) }
    }
    /// Lewati ke fase berikutnya sekarang juga.
    pub fn skip(&mut self, now: i64) { if self.phase != Phase::Idle { self.advance(now); } }

    fn advance(&mut self, now: i64) {
        match self.phase {
            Phase::Focus => {
                self.done += 1;
                if self.done % self.rounds == 0 { self.phase = Phase::LongBreak; self.ends_at = now + self.long_s; }
                else { self.phase = Phase::Break; self.ends_at = now + self.break_s; }
            }
            _ => { self.phase = Phase::Focus; self.ends_at = now + self.focus_s; }
        }
        self.paused_left = None;
    }

    /// Panggil tiap detik; true bila fase baru saja berganti (waktunya bunyi).
    /// Bila `auto_next` mati, fase berikutnya dimulai dalam keadaan jeda.
    pub fn tick(&mut self, now: i64) -> bool {
        if self.phase == Phase::Idle || self.paused_left.is_some() || now < self.ends_at { return false; }
        self.advance(now);
        if !self.auto_next { self.paused_left = Some(self.phase_len()); }
        true
    }
    pub fn label(&self, now: i64) -> String { let l = self.left(now); format!("{:02}:{:02}", l / 60, l % 60) }
    pub fn phase_name(&self) -> &'static str {
        match self.phase { Phase::Idle => "Siap fokus", Phase::Focus => "Fokus", Phase::Break => "Istirahat", Phase::LongBreak => "Istirahat panjang" }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cycle() {
        let mut f = Focus::new(25, 5);
        f.start(0);
        assert_eq!(f.label(60), "24:00");
        f.toggle_pause(100); assert!(!f.tick(99999)); f.toggle_pause(200);
        assert_eq!(f.left(200), 25 * 60 - 100);
        assert!(f.tick(f.ends_at));
        assert_eq!((f.phase, f.done), (Phase::Break, 1));
    }
    #[test]
    fn long_break_after_n_rounds() {
        let mut f = Focus::new(1, 1);
        f.configure(1, 1, 10, 2, true);
        f.start(0);
        let mut t = 0;
        let mut seen = vec![];
        for _ in 0..4 { t = f.ends_at; assert!(f.tick(t)); seen.push(f.phase); }
        assert_eq!(seen, [Phase::Break, Phase::Focus, Phase::LongBreak, Phase::Focus]);
        assert_eq!(f.done, 2);
        let _ = t;
    }
    #[test]
    fn manual_next_waits_paused() {
        let mut f = Focus::new(1, 1);
        f.configure(1, 1, 5, 4, false);
        f.start(0);
        assert!(f.tick(60));
        assert!(f.paused());
        assert!(!f.tick(10_000));
        f.toggle_pause(10_000);
        assert!(f.running());
    }
    #[test]
    fn progress_and_skip() {
        let mut f = Focus::new(10, 5);
        f.start(0);
        assert!((f.progress(300) - 0.5).abs() < 1e-4);
        f.skip(300);
        assert_eq!(f.phase, Phase::Break);
    }
}
