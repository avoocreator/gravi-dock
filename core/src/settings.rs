//! Pengaturan aplikasi (settings.json). Kompatibel dengan berkas lama (v0.5): kunci `topPct`, `light`, `focusMin`, dst. tetap dibaca.
use crate::json::{self, Value};
use crate::theme::{Dock, ThemeMode};

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    // lokasi (cuaca + waktu sholat)
    pub lat: f64, pub lon: f64, pub city: String,
    // tampilan
    pub dock: Dock, pub edge_pct: f64, pub scale: f64, pub theme: ThemeMode, pub scheme: String,
    pub accent: String,        // "" = ikut skema; "#rrggbb" = aksen kustom
    pub hover: bool, pub mascot: bool,
    pub renderer: String,      // "software" (hemat memori) | "gpu"
    // suara
    pub sfx: bool, pub sfx_vol: i64,
    // adzan
    pub adzan: bool, pub adzan_file: String, pub adzan_vol: i64,
    // pomodoro
    pub focus_min: i64, pub break_min: i64, pub long_break_min: i64, pub rounds: i64, pub auto_next: bool,
    // kalender
    pub ical: String, pub gcal_client_id: String, pub gcal_client_secret: String,
    // github
    pub gh_user: String, pub gh_token: String,
    // sumber sesi AI
    pub src_claude: bool, pub src_copilot: bool, pub src_agy: bool, pub src_desk: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            lat: -6.9175, lon: 107.6191, city: "Bandung".into(),
            dock: Dock::Right, edge_pct: 0.5, scale: 1.0, theme: ThemeMode::Dark, scheme: "catppuccin".into(), accent: String::new(),
            hover: false, mascot: true, renderer: "software".into(),
            sfx: true, sfx_vol: 60,
            adzan: true, adzan_file: String::new(), adzan_vol: 80,
            focus_min: 25, break_min: 5, long_break_min: 15, rounds: 4, auto_next: true,
            ical: String::new(), gcal_client_id: String::new(), gcal_client_secret: String::new(),
            gh_user: String::new(), gh_token: String::new(),
            src_claude: true, src_copilot: true, src_agy: true, src_desk: true,
        }
    }
}

impl Settings {
    pub fn parse(src: &str) -> Settings {
        let mut s = Settings::default();
        let Some(v) = json::parse(src) else { return s };
        let b = |k: &str, d: bool| { let x = v.get(k); if x.is_null() { d } else { x.truthy() } };
        let text = |k: &str, t: &mut String| { if let Some(x) = v.get(k).str() { *t = x.to_string() } };
        if let Some(x) = v.get("lat").num() { s.lat = x.clamp(-90.0, 90.0) }
        if let Some(x) = v.get("lon").num() { s.lon = x.clamp(-180.0, 180.0) }
        if let Some(x) = v.get("edgePct").num().or(v.get("topPct").num()) { s.edge_pct = x.clamp(0.0, 1.0) }
        if let Some(x) = v.get("scale").num() { s.scale = x.clamp(0.75, 1.40) }
        if let Some(x) = v.get("sfxVol").num() { s.sfx_vol = (x as i64).clamp(0, 100) }
        if let Some(x) = v.get("adzanVol").num() { s.adzan_vol = (x as i64).clamp(0, 100) }
        if let Some(x) = v.get("focusMin").num() { s.focus_min = (x as i64).clamp(1, 180) }
        if let Some(x) = v.get("breakMin").num() { s.break_min = (x as i64).clamp(1, 60) }
        if let Some(x) = v.get("longBreakMin").num() { s.long_break_min = (x as i64).clamp(1, 120) }
        if let Some(x) = v.get("rounds").num() { s.rounds = (x as i64).clamp(1, 12) }
        text("city", &mut s.city); text("scheme", &mut s.scheme); text("accent", &mut s.accent);
        text("renderer", &mut s.renderer); text("ical", &mut s.ical);
        text("adzanFile", &mut s.adzan_file); text("adzanWav", &mut s.adzan_file);
        text("gcalClientId", &mut s.gcal_client_id); text("gcalClientSecret", &mut s.gcal_client_secret);
        text("ghUser", &mut s.gh_user); text("ghToken", &mut s.gh_token);
        if let Some(x) = v.get("dock").str() { s.dock = Dock::parse(x) }
        if let Some(x) = v.get("theme").str() { s.theme = ThemeMode::parse(x) } else if !v.get("light").is_null() { s.theme = if v.get("light").truthy() { ThemeMode::Light } else { ThemeMode::Dark } }
        s.hover = b("hover", s.hover); s.mascot = b("mascot", s.mascot);
        s.sfx = b("sfx", s.sfx); s.adzan = b("adzan", s.adzan); s.auto_next = b("autoNext", s.auto_next);
        s.src_claude = b("srcClaude", true); s.src_copilot = b("srcCopilot", true); s.src_agy = b("srcAgy", true); s.src_desk = b("srcDesk", true);
        if s.renderer != "gpu" { s.renderer = "software".into() }
        s
    }

    pub fn to_json(&self) -> String {
        Value::obj(vec![
            ("lat", Value::n(self.lat)), ("lon", Value::n(self.lon)), ("city", Value::s(&self.city)),
            ("dock", Value::s(self.dock.as_str())), ("edgePct", Value::n(self.edge_pct)), ("scale", Value::n(self.scale)),
            ("theme", Value::s(self.theme.as_str())), ("scheme", Value::s(&self.scheme)), ("accent", Value::s(&self.accent)),
            ("hover", Value::b(self.hover)), ("mascot", Value::b(self.mascot)), ("renderer", Value::s(&self.renderer)),
            ("sfx", Value::b(self.sfx)), ("sfxVol", Value::n(self.sfx_vol as f64)),
            ("adzan", Value::b(self.adzan)), ("adzanFile", Value::s(&self.adzan_file)), ("adzanVol", Value::n(self.adzan_vol as f64)),
            ("focusMin", Value::n(self.focus_min as f64)), ("breakMin", Value::n(self.break_min as f64)),
            ("longBreakMin", Value::n(self.long_break_min as f64)), ("rounds", Value::n(self.rounds as f64)), ("autoNext", Value::b(self.auto_next)),
            ("ical", Value::s(&self.ical)), ("gcalClientId", Value::s(&self.gcal_client_id)), ("gcalClientSecret", Value::s(&self.gcal_client_secret)),
            ("ghUser", Value::s(&self.gh_user)), ("ghToken", Value::s(&self.gh_token)),
            ("srcClaude", Value::b(self.src_claude)), ("srcCopilot", Value::b(self.src_copilot)),
            ("srcAgy", Value::b(self.src_agy)), ("srcDesk", Value::b(self.src_desk)),
        ]).pretty()
    }

    pub fn enabled(&self, provider: &str) -> bool {
        match provider { "claude" => self.src_claude, "copilot" => self.src_copilot, "agy" => self.src_agy, "claudedesk" => self.src_desk, _ => true }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip() {
        let mut s = Settings::default();
        s.city = "Jakarta \"Pusat\"".into(); s.hover = true; s.src_agy = false; s.ical = "https://x/y.ics".into();
        s.dock = Dock::Top; s.theme = ThemeMode::Auto; s.scheme = "catppuccin".into(); s.scale = 1.25; s.sfx_vol = 33; s.gh_user = "avoo".into();
        assert_eq!(Settings::parse(&s.to_json()), s);
        assert_eq!(Settings::parse("rusak{"), Settings::default());
    }
    #[test]
    fn reads_legacy_v05_file() {
        let old = r##"{"lat": -6.2, "lon": 106.8, "city": "Jakarta", "hover": true, "topPct": 0.25, "light": true, "accent": "#5ae0ff", "adzanWav": "C:\\a.wav", "focusMin": 50}"##;
        let s = Settings::parse(old);
        assert_eq!(s.theme, ThemeMode::Light);
        assert_eq!(s.edge_pct, 0.25);
        assert_eq!(s.scale, 1.0);
        assert_eq!(s.adzan_file, "C:\\a.wav");
        assert_eq!(s.focus_min, 50);
        assert_eq!(s.dock, Dock::Right);
    }
    #[test]
    fn clamps_bad_values() {
        let s = Settings::parse(r#"{"sfxVol": 900, "focusMin": -5, "renderer": "weird", "edgePct": 4, "scale": 5}"#);
        assert_eq!((s.sfx_vol, s.focus_min, s.edge_pct, s.scale), (100, 1, 1.0, 1.40));
        assert_eq!(s.renderer, "software");
    }
}
