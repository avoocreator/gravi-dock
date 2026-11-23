//! 20 suasana hati maskot Gravi, dipilih dari konteks. Murni logika; UI hanya menggambar `MoodView`.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mood {
    Idle, Happy, Working, Focused, Sleepy, Sleeping, Alert, Worried, Hot, Music,
    Break, Prayer, Morning, Night, Excited, Curious, Love, Thinking, Done, Sad,
}

/// Cara menggambar: tinggi mata (0..1), geser mata (-1..1), bentuk mulut, glyph melayang, warna mata (rgb) bila beda.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MoodView { pub eye_h: f32, pub eye_dy: f32, pub mouth: u8, pub glyph: &'static str, pub rgb: Option<(u8, u8, u8)> }
// mouth: 0 senyum ω, 1 datar, 2 terbuka O, 3 sedih, 4 senyum lebar

#[derive(Clone, Copy, Debug, Default)]
pub struct Ctx {
    pub hour: i64, pub running: i32, pub action: bool, pub cpu: f32, pub mem: f32,
    pub focus: bool, pub brk: bool, pub music: bool, pub hover: bool,
    pub prayer_left_s: i64, pub prayer_now: bool, pub just_done: bool, pub clicked_recently: bool,
}

pub fn pick(c: &Ctx) -> Mood {
    use Mood::*;
    if c.action { return Alert; }
    if c.prayer_now { return Prayer; }
    if c.just_done { return Done; }
    if c.clicked_recently { return Love; }
    if c.hover { return Curious; }
    if c.mem > 0.92 || c.cpu > 0.95 { return Hot; }
    if c.mem > 0.85 { return Worried; }
    if c.focus { return Focused; }
    if c.brk { return Break; }
    if c.running >= 3 { return Excited; }
    if c.running > 0 { return if c.cpu > 0.6 { Thinking } else { Working }; }
    if c.music { return Music; }
    if c.prayer_left_s > 0 && c.prayer_left_s < 600 { return Prayer; }
    match c.hour { 0..=3 => Sleeping, 4..=5 => Sleepy, 6..=9 => Morning, 22..=23 => Night, 10..=11 | 13..=17 => Happy, 12 => Happy, _ => Idle }
}

pub fn view(m: Mood) -> MoodView {
    use Mood::*;
    let v = |eye_h, eye_dy, mouth, glyph, rgb| MoodView { eye_h, eye_dy, mouth, glyph, rgb };
    match m {
        Idle => v(1.0, 0.0, 0, "", None),
        Happy => v(0.55, 0.0, 4, "", None),
        Working => v(0.8, 0.3, 1, "", None),
        Focused => v(0.7, 0.0, 1, "", Some((0x34, 0xd3, 0x99))),
        Sleepy => v(0.3, 0.2, 1, "z", None),
        Sleeping => v(0.12, 0.4, 1, "zZ", None),
        Alert => v(1.2, -0.2, 2, "!", Some((0xff, 0xb5, 0x4d))),
        Worried => v(1.0, 0.0, 3, "", Some((0xff, 0xb5, 0x4d))),
        Hot => v(0.6, 0.0, 2, "~", Some((0xff, 0x6b, 0x6b))),
        Music => v(0.55, 0.0, 4, "♪", None),
        Break => v(0.5, 0.0, 4, "☕", None),
        Prayer => v(0.4, 0.2, 0, "☾", Some((0x34, 0xd3, 0x99))),
        Morning => v(0.9, 0.0, 4, "☀", None),
        Night => v(0.45, 0.2, 0, "★", None),
        Excited => v(1.15, -0.1, 4, "✦", None),
        Curious => v(1.1, -0.3, 2, "", None),
        Love => v(0.5, 0.0, 4, "♥", Some((0xff, 0x6b, 0x9a))),
        Thinking => v(0.8, -0.4, 1, "…", None),
        Done => v(0.55, 0.0, 4, "✓", Some((0x34, 0xd3, 0x99))),
        Sad => v(0.8, 0.3, 3, "", None),
    }
}

pub const ALL: [Mood; 20] = [
    Mood::Idle, Mood::Happy, Mood::Working, Mood::Focused, Mood::Sleepy, Mood::Sleeping, Mood::Alert, Mood::Worried, Mood::Hot, Mood::Music,
    Mood::Break, Mood::Prayer, Mood::Morning, Mood::Night, Mood::Excited, Mood::Curious, Mood::Love, Mood::Thinking, Mood::Done, Mood::Sad,
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn priorities() {
        let base = Ctx { hour: 14, ..Default::default() };
        assert_eq!(pick(&base), Mood::Happy);
        assert_eq!(pick(&Ctx { action: true, running: 2, ..base }), Mood::Alert);
        assert_eq!(pick(&Ctx { running: 1, ..base }), Mood::Working);
        assert_eq!(pick(&Ctx { running: 1, cpu: 0.7, ..base }), Mood::Thinking);
        assert_eq!(pick(&Ctx { hour: 2, ..base }), Mood::Sleeping);
        assert_eq!(pick(&Ctx { focus: true, ..base }), Mood::Focused);
        assert_eq!(pick(&Ctx { prayer_left_s: 300, ..base }), Mood::Prayer);
    }
    #[test]
    fn all_distinct_views_exist() {
        for m in ALL { let v = view(m); assert!(v.eye_h > 0.0); }
        assert_eq!(ALL.len(), 20);
    }
}
