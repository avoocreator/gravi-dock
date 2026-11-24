//! Ekspresi maskot Gravi. 12 wajah (sesuai lembar referensi) + aksesori melayang (glyph) yang dipilih dari konteks.
//! Murni logika: UI hanya menganimasikan angka-angka di `FaceView` (transisi mulus antar wajah).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Face { Idle, Happy, Surprised, Sleepy, Confused, Sad, Thinking, Searching, Working, Success, Error, Peek }

pub const ALL: [Face; 12] = [
    Face::Idle, Face::Happy, Face::Surprised, Face::Sleepy, Face::Confused, Face::Sad,
    Face::Thinking, Face::Searching, Face::Working, Face::Success, Face::Error, Face::Peek,
];

impl Face {
    pub fn name(self) -> &'static str {
        match self {
            Face::Idle => "Idle", Face::Happy => "Happy", Face::Surprised => "Surprised", Face::Sleepy => "Sleepy", Face::Confused => "Confused",
            Face::Sad => "Sad", Face::Thinking => "Thinking", Face::Searching => "Searching", Face::Working => "Working", Face::Success => "Success",
            Face::Error => "Error", Face::Peek => "Peek",
        }
    }
    pub fn index(self) -> i32 { ALL.iter().position(|f| *f == self).unwrap_or(0) as i32 }
    pub fn from_index(i: i32) -> Face { ALL.get(i.max(0) as usize).copied().unwrap_or(Face::Idle) }
}

/// Parameter gambar. Semua angka dianimasikan di UI.
/// Mata: pil (tinggi `eye_h`, lebar `eye_w`), busur ^ (`arc`), cincin (`ring`), kelopak miring (`lid_out`/`lid_in`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FaceView {
    pub eye_h: f32, pub eye_w: f32,   // mata kiri; 1.0 = ukuran normal
    pub r_h: f32, pub r_w: f32,       // mata kanan
    pub arc: f32, pub ring: f32,      // 0..1 opasitas lapisan busur/cincin
    pub lid_out: f32, pub lid_in: f32, // 0..1 bagian mata yang tertutup kelopak (sisi luar / dalam)
    pub look_x: f32, pub look_y: f32, // -1..1 arah pandang
    pub peek: f32,                    // 0..1 seberapa jauh maskot "mengintip" dari tepi bawah
    pub glyph: &'static str,          // aksesori melayang ("" = tak ada)
    pub glyph_rgb: (u8, u8, u8),
}

const BLUE: (u8, u8, u8) = (0x4d, 0x7c, 0xff);
const ORANGE: (u8, u8, u8) = (0xff, 0x9a, 0x3d);
const YELLOW: (u8, u8, u8) = (0xff, 0xc9, 0x4d);
const PURPLE: (u8, u8, u8) = (0xa7, 0x8b, 0xfa);
const GRAY: (u8, u8, u8) = (0x8b, 0x93, 0xa3);

const BASE: FaceView = FaceView { eye_h: 1.0, eye_w: 1.0, r_h: 1.0, r_w: 1.0, arc: 0.0, ring: 0.0, lid_out: 0.0, lid_in: 0.0, look_x: 0.0, look_y: 0.0, peek: 0.0, glyph: "", glyph_rgb: GRAY };

pub fn view(f: Face) -> FaceView {
    match f {
        Face::Idle => BASE,
        Face::Happy => FaceView { eye_h: 0.0, r_h: 0.0, arc: 1.0, glyph: "✦", glyph_rgb: ORANGE, ..BASE },
        Face::Surprised => FaceView { eye_h: 0.0, r_h: 0.0, ring: 1.0, glyph: "!!", glyph_rgb: BLUE, ..BASE },
        Face::Sleepy => FaceView { eye_h: 0.1, r_h: 0.1, eye_w: 1.25, r_w: 1.25, look_y: 0.2, glyph: "zZ", glyph_rgb: PURPLE, ..BASE },
        Face::Confused => FaceView { eye_h: 1.0, r_h: 0.16, r_w: 1.35, look_x: 0.1, glyph: "?", glyph_rgb: BLUE, ..BASE },
        Face::Sad => FaceView { eye_h: 0.8, r_h: 0.8, lid_out: 0.55, lid_in: 0.0, look_y: 0.25, ..BASE },
        Face::Thinking => FaceView { eye_h: 0.8, r_h: 0.8, lid_out: 0.0, lid_in: 0.5, look_x: 0.15, look_y: -0.2, glyph: "…", glyph_rgb: GRAY, ..BASE },
        Face::Searching => FaceView { eye_h: 0.85, r_h: 0.85, eye_w: 0.9, r_w: 0.9, look_x: -0.8, glyph: ")))", glyph_rgb: GRAY, ..BASE },
        Face::Working => FaceView { eye_h: 0.95, r_h: 0.95, look_y: 0.1, glyph: "//", glyph_rgb: BLUE, ..BASE },
        Face::Success => FaceView { eye_h: 0.0, r_h: 0.0, arc: 1.0, glyph: "✦", glyph_rgb: YELLOW, ..BASE },
        Face::Error => FaceView { eye_h: 0.9, r_h: 0.45, r_w: 1.15, lid_out: 0.0, lid_in: 0.0, look_x: 0.1, glyph: "@", glyph_rgb: GRAY, ..BASE },
        Face::Peek => FaceView { peek: 1.0, ..BASE },
    }
}

/// Konteks untuk memilih wajah.
#[derive(Clone, Copy, Debug, Default)]
pub struct Ctx {
    pub hour: i64,
    pub ai_running: i32,      // jumlah sesi AI yang sedang berjalan
    pub ai_action: bool,      // ada sesi yang menunggu persetujuan
    pub cpu: f32, pub mem: f32,
    pub battery_low: bool,
    pub focus: bool, pub on_break: bool,
    pub music: bool,
    pub searching: bool,      // sedang mengetik di kotak cari / memuat data
    pub offline: bool,
    pub prayer_now: bool, pub prayer_soon: bool,
    pub just_done: bool,      // sesuatu baru saja selesai (sesi AI, timer, pomodoro)
    pub clicked: bool,        // maskot baru diklik
    pub alarm_ringing: bool,
    pub idle_long: bool,      // lama tak ada interaksi
}

/// Pilih wajah + aksesori. Urutan menunjukkan prioritas.
pub fn pick(c: &Ctx) -> FaceView {
    let with = |f: Face, glyph: &'static str, rgb: (u8, u8, u8)| FaceView { glyph, glyph_rgb: rgb, ..view(f) };
    if c.ai_action || c.alarm_ringing { return with(Face::Surprised, "!!", BLUE); }
    if c.cpu > 0.95 || c.mem > 0.93 { return view(Face::Error); }
    if c.just_done { return view(Face::Success); }
    if c.clicked { return view(Face::Happy); }
    if c.prayer_now { return with(Face::Happy, "☾", YELLOW); }
    if c.battery_low { return view(Face::Sad); }
    if c.searching { return view(Face::Searching); }
    if c.offline { return view(Face::Confused); }
    if c.ai_running > 0 { return if c.cpu > 0.6 { view(Face::Thinking) } else { view(Face::Working) }; }
    if c.focus { return with(Face::Working, "◔", BLUE); }
    if c.on_break { return with(Face::Happy, "☕", ORANGE); }
    if c.music { return with(Face::Happy, "♪", ORANGE); }
    if c.prayer_soon { return with(Face::Idle, "☾", YELLOW); }
    if c.idle_long && (6..22).contains(&c.hour) { return view(Face::Peek); }
    match c.hour { 0..=4 => view(Face::Sleepy), 22..=23 => with(Face::Sleepy, "", GRAY), _ => view(Face::Idle) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn twelve_faces_all_valid() {
        assert_eq!(ALL.len(), 12);
        for (i, f) in ALL.iter().enumerate() {
            let v = view(*f);
            assert_eq!(f.index() as usize, i);
            assert_eq!(Face::from_index(i as i32), *f);
            for x in [v.eye_h, v.r_h, v.arc, v.ring, v.lid_out, v.lid_in, v.peek] { assert!((0.0..=1.2).contains(&x), "{f:?}"); }
        }
        assert_eq!(Face::from_index(99), Face::Idle);
    }
    #[test]
    fn faces_are_distinct() {
        for (i, a) in ALL.iter().enumerate() { for b in &ALL[i + 1..] { assert_ne!(view(*a), view(*b), "{a:?} == {b:?}"); } }
    }
    #[test]
    fn priorities() {
        let base = Ctx { hour: 14, ..Default::default() };
        assert_eq!(pick(&base), view(Face::Idle));
        assert_eq!(pick(&Ctx { ai_action: true, ai_running: 2, ..base }).ring, 1.0);
        assert_eq!(pick(&Ctx { ai_running: 1, ..base }), view(Face::Working));
        assert_eq!(pick(&Ctx { ai_running: 1, cpu: 0.7, ..base }), view(Face::Thinking));
        assert_eq!(pick(&Ctx { hour: 2, ..base }), view(Face::Sleepy));
        assert_eq!(pick(&Ctx { just_done: true, ai_running: 1, ..base }), view(Face::Success));
        assert_eq!(pick(&Ctx { music: true, ..base }).glyph, "♪");
        assert_eq!(pick(&Ctx { cpu: 0.99, ai_running: 1, ..base }), view(Face::Error));
        assert_eq!(pick(&Ctx { idle_long: true, ..base }).peek, 1.0);
    }
}
