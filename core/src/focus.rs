//! Timer fokus/pomodoro berbasis waktu dinding (tahan terhadap timer UI yang tertunda).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase { Idle, Focus, Break }

#[derive(Clone, Debug)]
pub struct Focus { pub phase: Phase, pub ends_at: i64, pub paused_left: Option<i64>, pub focus_s: i64, pub break_s: i64, pub done: u32 }

impl Focus {
    pub fn new(focus_min: i64, break_min: i64) -> Self { Focus { phase: Phase::Idle, ends_at: 0, paused_left: None, focus_s: focus_min * 60, break_s: break_min * 60, done: 0 } }
    pub fn start(&mut self, now: i64) { self.phase = Phase::Focus; self.ends_at = now + self.focus_s; self.paused_left = None; }
    pub fn stop(&mut self) { self.phase = Phase::Idle; self.paused_left = None; }
    pub fn left(&self, now: i64) -> i64 { match (self.phase, self.paused_left) { (Phase::Idle, _) => self.focus_s, (_, Some(l)) => l, _ => (self.ends_at - now).max(0) } }
    pub fn toggle_pause(&mut self, now: i64) {
        if self.phase == Phase::Idle { return; }
        match self.paused_left { Some(l) => { self.ends_at = now + l; self.paused_left = None } None => self.paused_left = Some(self.left(now)) }
    }
    /// Panggil tiap detik; true bila fase baru saja berganti (waktunya bunyi).
    pub fn tick(&mut self, now: i64) -> bool {
        if self.phase == Phase::Idle || self.paused_left.is_some() || now < self.ends_at { return false; }
        if self.phase == Phase::Focus { self.done += 1; self.phase = Phase::Break; self.ends_at = now + self.break_s; } else { self.phase = Phase::Focus; self.ends_at = now + self.focus_s; }
        true
    }
    pub fn label(&self, now: i64) -> String { let l = self.left(now); format!("{:02}:{:02}", l / 60, l % 60) }
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
}
