//! Tema, skema warna, dan perhitungan posisi island (kiri / kanan / atas). Murni logika.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dock { Left, Right, Top }

impl Dock {
    pub fn parse(s: &str) -> Dock {
        match s.to_ascii_lowercase().as_str() { "left" | "kiri" => Dock::Left, "top" | "atas" => Dock::Top, _ => Dock::Right }
    }
    pub fn as_str(self) -> &'static str { match self { Dock::Left => "left", Dock::Right => "right", Dock::Top => "top" } }
    pub fn index(self) -> i32 { match self { Dock::Left => 0, Dock::Right => 1, Dock::Top => 2 } }
    pub fn from_index(i: i32) -> Dock { match i { 0 => Dock::Left, 2 => Dock::Top, _ => Dock::Right } }
    pub fn vertical(self) -> bool { self != Dock::Top }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode { Dark, Light, Auto }

impl ThemeMode {
    pub fn parse(s: &str) -> ThemeMode { match s { "light" => ThemeMode::Light, "auto" => ThemeMode::Auto, _ => ThemeMode::Dark } }
    pub fn as_str(self) -> &'static str { match self { ThemeMode::Dark => "dark", ThemeMode::Light => "light", ThemeMode::Auto => "auto" } }
    pub fn index(self) -> i32 { match self { ThemeMode::Dark => 0, ThemeMode::Light => 1, ThemeMode::Auto => 2 } }
    pub fn from_index(i: i32) -> ThemeMode { match i { 1 => ThemeMode::Light, 2 => ThemeMode::Auto, _ => ThemeMode::Dark } }
    /// Gelap atau terang, memakai `system_dark` bila mode Auto.
    pub fn is_dark(self, system_dark: bool) -> bool { match self { ThemeMode::Dark => true, ThemeMode::Light => false, ThemeMode::Auto => system_dark } }
}

/// Skema warna: nama, aksen utama, aksen kedua (untuk gradien/grafik).
pub struct Scheme { pub id: &'static str, pub label: &'static str, pub accent: &'static str, pub accent2: &'static str }

pub const SCHEMES: &[Scheme] = &[
    Scheme { id: "ocean", label: "Samudra", accent: "#4da3ff", accent2: "#5ae0ff" },
    Scheme { id: "mint", label: "Mint", accent: "#34d399", accent2: "#a3e635" },
    Scheme { id: "sunset", label: "Senja", accent: "#ff8a4c", accent2: "#ffc94d" },
    Scheme { id: "grape", label: "Anggur", accent: "#a78bfa", accent2: "#f472b6" },
    Scheme { id: "rose", label: "Mawar", accent: "#fb7185", accent2: "#fda4af" },
    Scheme { id: "mono", label: "Mono", accent: "#c7ccd6", accent2: "#8b93a3" },
];

pub fn scheme(id: &str) -> &'static Scheme { SCHEMES.iter().find(|s| s.id == id).unwrap_or(&SCHEMES[0]) }
pub fn scheme_index(id: &str) -> i32 { SCHEMES.iter().position(|s| s.id == id).unwrap_or(0) as i32 }

/// "#rrggbb" -> (r, g, b); None bila tidak valid.
pub fn parse_hex(s: &str) -> Option<(u8, u8, u8)> {
    let t = s.trim().trim_start_matches('#');
    if t.len() != 6 { return None; }
    let p = |a: usize| u8::from_str_radix(&t[a..a + 2], 16).ok();
    Some((p(0)?, p(2)?, p(4)?))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Rect { pub x: i32, pub y: i32, pub w: i32, pub h: i32 }

impl Rect {
    pub fn right(&self) -> i32 { self.x + self.w }
    pub fn bottom(&self) -> i32 { self.y + self.h }
}

/// Hasil penempatan. `win` = persegi jendela (piksel layar); `pill_x/pill_y` = posisi pil di dalam jendela.
/// Saat tertutup jendela = pil (pill_x = pill_y = 0). Saat terbuka jendela = panel, dan pil berada di tempat
/// asalnya sehingga UI bisa menganimasikan pil -> panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement { pub win: Rect, pub pill_x: i32, pub pill_y: i32 }

fn clamp(v: i32, lo: i32, hi: i32) -> i32 { if hi < lo { lo } else { v.clamp(lo, hi) } }

/// `pill`/`panel` dalam piksel; `edge` 0..1 = posisi sepanjang tepi (vertikal untuk kiri/kanan, horizontal untuk atas).
pub fn place(dock: Dock, work: Rect, pill: (i32, i32), panel: (i32, i32), edge: f64, margin: i32, expanded: bool) -> Placement {
    let edge = edge.clamp(0.0, 1.0);
    let (pw, ph) = pill;
    let panel_w = panel.0.min(work.w - 2 * margin).max(pw);
    let panel_h = panel.1.min(work.h - 2 * margin).max(ph);
    let (px, py) = match dock {
        Dock::Right => (work.right() - pw - margin, work.y + margin + (edge * (work.h - ph - 2 * margin).max(0) as f64) as i32),
        Dock::Left => (work.x + margin, work.y + margin + (edge * (work.h - ph - 2 * margin).max(0) as f64) as i32),
        Dock::Top => (work.x + margin + (edge * (work.w - pw - 2 * margin).max(0) as f64) as i32, work.y + margin),
    };
    if !expanded {
        return Placement { win: Rect { x: px, y: py, w: pw, h: ph }, pill_x: 0, pill_y: 0 };
    }
    let (wx, wy) = match dock {
        Dock::Right => (work.right() - panel_w - margin, clamp(py + ph / 2 - panel_h / 2, work.y + margin, work.bottom() - margin - panel_h)),
        Dock::Left => (work.x + margin, clamp(py + ph / 2 - panel_h / 2, work.y + margin, work.bottom() - margin - panel_h)),
        Dock::Top => (clamp(px + pw / 2 - panel_w / 2, work.x + margin, work.right() - margin - panel_w), work.y + margin),
    };
    Placement { win: Rect { x: wx, y: wy, w: panel_w, h: panel_h }, pill_x: px - wx, pill_y: py - wy }
}

#[cfg(test)]
mod tests {
    use super::*;
    const W: Rect = Rect { x: 0, y: 0, w: 1920, h: 1040 };

    #[test]
    fn collapsed_right_is_flush_to_edge() {
        let p = place(Dock::Right, W, (80, 180), (440, 640), 0.5, 12, false);
        assert_eq!(p.win.right(), 1920 - 12);
        assert_eq!((p.pill_x, p.pill_y), (0, 0));
        assert!(p.win.y > 0 && p.win.bottom() < 1040);
    }
    #[test]
    fn expanded_panel_contains_pill_and_stays_on_screen() {
        for dock in [Dock::Left, Dock::Right, Dock::Top] {
            for edge in [0.0, 0.5, 1.0] {
                let p = place(dock, W, (80, 180), (440, 640), edge, 12, true);
                assert!(p.win.x >= 0 && p.win.right() <= 1920, "{dock:?} {edge} {p:?}");
                assert!(p.win.y >= 0 && p.win.bottom() <= 1040, "{dock:?} {edge} {p:?}");
                assert!(p.pill_x >= 0 && p.pill_y >= 0, "{dock:?} {edge} {p:?}");
            }
        }
    }
    #[test]
    fn top_dock_centers_by_default() {
        let p = place(Dock::Top, W, (260, 52), (440, 640), 0.5, 12, false);
        assert!((p.win.x + 130 - 960).abs() < 20);
        assert_eq!(p.win.y, 12);
    }
    #[test]
    fn small_screen_clamps_panel() {
        let small = Rect { x: 0, y: 0, w: 1280, h: 560 };
        let p = place(Dock::Right, small, (80, 180), (440, 640), 0.5, 12, true);
        assert!(p.win.h <= 560 - 24);
    }
    #[test]
    fn parse_helpers() {
        assert_eq!(Dock::parse("KIRI"), Dock::Left);
        assert_eq!(Dock::parse("???"), Dock::Right);
        assert_eq!(parse_hex("#4da3ff"), Some((0x4d, 0xa3, 0xff)));
        assert_eq!(parse_hex("zzz"), None);
        assert!(ThemeMode::Auto.is_dark(true) && !ThemeMode::Light.is_dark(true));
        assert_eq!(scheme("nope").id, "ocean");
    }
}
