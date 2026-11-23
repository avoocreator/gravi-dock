//! Suasana hati & 12 ekspresi maskot Gravi Bot (vektor).
//! Murni logika; UI hanya menggambar `MoodView`.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mood {
    Idle, Happy, Working, Focused, Sleepy, Sleeping, Alert, Worried, Hot, Music,
    Break, Prayer, Morning, Night, Excited, Curious, Love, Thinking, Done, Sad,
}

/// 12 ekspresi resmi Gravi Bot sesuai referensi visual.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expression {
    Idle, Happy, Surprised, Sleepy, Confused, Sad,
    Thinking, Searching, Working, Success, Error, Peek,
}

/// Cara menggambar di Slint:
/// - eye_h: skala tinggi mata (0.1..1.5)
/// - eye_w: skala lebar mata (0.5..1.5)
/// - eye_dy: pergeseran vertikal mata (-1.0..1.0)
/// - eye_dx: pergeseran horizontal mata (-1.0..1.0, misal melirik)
/// - eye_shape: 0=Kapsul tegak, 1=Senyum ^^, 2=Bulat OO (Surprised), 3=Tidur --,
///              4=Bingung (asimetris), 5=Sayu/sedih, 6=Menyipit miring (Thinking), 7=Miring linglung (Error)
/// - mouth: 0 senyum halus, 1 datar, 2 terbuka O, 3 sedih, 4 senyum lebar
/// - glyph: teks pendukung atau ikon
/// - glyph_type: 0=none, 1=seru '!', 2=tanya '?', 3=zzz 'Z', 4=bintang '✦', 5=spiral '@', 6=bubble '..', 7=radar ')', 8=spark '\\//', 9=musik '♪'
/// - peek: true jika hanya separuh kepala atas yang muncul mengintip
/// - rgb: override warna mata bila ada
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MoodView {
    pub eye_h: f32,
    pub eye_w: f32,
    pub eye_dy: f32,
    pub eye_dx: f32,
    pub eye_shape: u8,
    pub mouth: u8,
    pub glyph: &'static str,
    pub glyph_type: u8,
    pub peek: bool,
    pub rgb: Option<(u8, u8, u8)>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Ctx {
    pub hour: i64, pub running: i32, pub action: bool, pub cpu: f32, pub mem: f32,
    pub focus: bool, pub brk: bool, pub music: bool, pub hover: bool,
    pub prayer_left_s: i64, pub prayer_now: bool, pub just_done: bool, pub clicked_recently: bool,
    pub is_peek: bool, pub is_searching: bool, pub is_error: bool,
}

pub fn pick(c: &Ctx) -> Mood {
    use Mood::*;
    if c.is_error { return Worried; }
    if c.action { return Alert; }
    if c.prayer_now { return Prayer; }
    if c.just_done { return Done; }
    if c.clicked_recently { return Love; }
    if c.is_searching || c.hover { return Curious; }
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

/// Menghasilkan MoodView untuk 12 ekspresi resmi Gravi Bot.
pub fn expression_view(e: Expression) -> MoodView {
    use Expression::*;
    match e {
        Idle => MoodView {
            eye_h: 1.0, eye_w: 1.0, eye_dy: 0.0, eye_dx: 0.0,
            eye_shape: 0, mouth: 0, glyph: "", glyph_type: 0, peek: false, rgb: None,
        },
        Happy => MoodView {
            eye_h: 0.65, eye_w: 1.05, eye_dy: 0.0, eye_dx: 0.0,
            eye_shape: 1, mouth: 4, glyph: "", glyph_type: 0, peek: false, rgb: None,
        },
        Surprised => MoodView {
            eye_h: 1.15, eye_w: 1.15, eye_dy: -0.1, eye_dx: 0.0,
            eye_shape: 2, mouth: 2, glyph: "!", glyph_type: 1, peek: false,
            rgb: Some((0x5a, 0xe0, 0xff)),
        },
        Sleepy => MoodView {
            eye_h: 0.28, eye_w: 1.15, eye_dy: 0.12, eye_dx: 0.0,
            eye_shape: 3, mouth: 1, glyph: "Z", glyph_type: 3, peek: false,
            rgb: Some((0xa7, 0x8b, 0xfa)),
        },
        Confused => MoodView {
            eye_h: 0.9, eye_w: 1.0, eye_dy: 0.0, eye_dx: 0.0,
            eye_shape: 4, mouth: 1, glyph: "?", glyph_type: 2, peek: false,
            rgb: Some((0x60, 0xa5, 0xfa)),
        },
        Sad => MoodView {
            eye_h: 0.8, eye_w: 1.0, eye_dy: 0.15, eye_dx: 0.0,
            eye_shape: 5, mouth: 3, glyph: "", glyph_type: 0, peek: false,
            rgb: Some((0x93, 0xc5, 0xfd)),
        },
        Thinking => MoodView {
            eye_h: 0.8, eye_w: 1.0, eye_dy: -0.15, eye_dx: 0.0,
            eye_shape: 6, mouth: 1, glyph: "..", glyph_type: 6, peek: false,
            rgb: None,
        },
        Searching => MoodView {
            eye_h: 1.0, eye_w: 1.0, eye_dy: 0.0, eye_dx: 0.35,
            eye_shape: 0, mouth: 0, glyph: "))", glyph_type: 7, peek: false,
            rgb: Some((0x38, 0xbd, 0xf8)),
        },
        Working => MoodView {
            eye_h: 0.95, eye_w: 1.0, eye_dy: 0.05, eye_dx: 0.0,
            eye_shape: 0, mouth: 1, glyph: "\\//", glyph_type: 8, peek: false,
            rgb: Some((0x38, 0xbd, 0xf8)),
        },
        Success => MoodView {
            eye_h: 0.7, eye_w: 1.1, eye_dy: 0.0, eye_dx: 0.0,
            eye_shape: 1, mouth: 4, glyph: "✦", glyph_type: 4, peek: false,
            rgb: Some((0xfb, 0xbf, 0x24)),
        },
        Error => MoodView {
            eye_h: 0.85, eye_w: 1.0, eye_dy: 0.0, eye_dx: 0.0,
            eye_shape: 7, mouth: 3, glyph: "@", glyph_type: 5, peek: false,
            rgb: Some((0xf8, 0x71, 0x71)),
        },
        Peek => MoodView {
            eye_h: 0.9, eye_w: 1.0, eye_dy: -0.1, eye_dx: 0.0,
            eye_shape: 0, mouth: 0, glyph: "✦", glyph_type: 4, peek: true,
            rgb: None,
        },
    }
}

pub fn view(m: Mood) -> MoodView {
    use Mood::*;
    let v = match m {
        Idle => expression_view(Expression::Idle),
        Happy => expression_view(Expression::Happy),
        Working => expression_view(Expression::Working),
        Focused => {
            let mut x = expression_view(Expression::Thinking);
            x.rgb = Some((0x34, 0xd3, 0x99));
            x
        },
        Sleepy => expression_view(Expression::Sleepy),
        Sleeping => {
            let mut x = expression_view(Expression::Sleepy);
            x.eye_h = 0.18;
            x.glyph = "zZ";
            x
        },
        Alert => expression_view(Expression::Surprised),
        Worried => expression_view(Expression::Error),
        Hot => {
            let mut x = expression_view(Expression::Error);
            x.rgb = Some((0xff, 0x6b, 0x6b));
            x.glyph = "~";
            x
        },
        Music => {
            let mut x = expression_view(Expression::Happy);
            x.glyph = "♪";
            x.glyph_type = 9;
            x
        },
        Break => {
            let mut x = expression_view(Expression::Happy);
            x.glyph = "☕";
            x
        },
        Prayer => {
            let mut x = expression_view(Expression::Idle);
            x.eye_h = 0.4;
            x.eye_dy = 0.2;
            x.eye_shape = 3;
            x.glyph = "☾";
            x.rgb = Some((0x34, 0xd3, 0x99));
            x
        },
        Morning => {
            let mut x = expression_view(Expression::Happy);
            x.glyph = "☀";
            x
        },
        Night => {
            let mut x = expression_view(Expression::Sleepy);
            x.glyph = "★";
            x
        },
        Excited => {
            let mut x = expression_view(Expression::Success);
            x.eye_h = 1.15;
            x
        },
        Curious => expression_view(Expression::Searching),
        Love => {
            let mut x = expression_view(Expression::Happy);
            x.glyph = "♥";
            x.rgb = Some((0xff, 0x6b, 0x9a));
            x
        },
        Thinking => expression_view(Expression::Thinking),
        Done => expression_view(Expression::Success),
        Sad => expression_view(Expression::Sad),
    };
    v
}

pub const ALL: [Mood; 20] = [
    Mood::Idle, Mood::Happy, Mood::Working, Mood::Focused, Mood::Sleepy, Mood::Sleeping, Mood::Alert, Mood::Worried, Mood::Hot, Mood::Music,
    Mood::Break, Mood::Prayer, Mood::Morning, Mood::Night, Mood::Excited, Mood::Curious, Mood::Love, Mood::Thinking, Mood::Done, Mood::Sad,
];

pub const ALL_EXPRESSIONS: [Expression; 12] = [
    Expression::Idle, Expression::Happy, Expression::Surprised, Expression::Sleepy,
    Expression::Confused, Expression::Sad, Expression::Thinking, Expression::Searching,
    Expression::Working, Expression::Success, Expression::Error, Expression::Peek,
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
    #[test]
    fn all_12_expressions_exist() {
        for e in ALL_EXPRESSIONS {
            let v = expression_view(e);
            assert!(v.eye_h > 0.0);
        }
        assert_eq!(ALL_EXPRESSIONS.len(), 12);
    }
}
