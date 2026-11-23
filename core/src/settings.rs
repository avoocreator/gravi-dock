//! Pengaturan aplikasi (settings.json) + penyimpanan sederhana tanpa pustaka.
use crate::json;

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub lat: f64, pub lon: f64, pub city: String,
    pub hover: bool, pub top_pct: f64, pub light: bool,
    pub accent: String,       // #rrggbb
    pub ical: String,         // URL secret iCal
    pub adzan: bool, pub adzan_wav: String,
    pub focus_min: i64, pub break_min: i64,
    pub src_claude: bool, pub src_copilot: bool, pub src_agy: bool, pub src_desk: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { lat: -6.9175, lon: 107.6191, city: "Bandung".into(), hover: false, top_pct: 0.0, light: false, accent: "#5ae0ff".into(),
            ical: String::new(), adzan: true, adzan_wav: String::new(), focus_min: 25, break_min: 5,
            src_claude: true, src_copilot: true, src_agy: true, src_desk: true }
    }
}

fn q(s: &str) -> String { format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")) }

impl Settings {
    pub fn parse(src: &str) -> Settings {
        let mut s = Settings::default();
        let Some(v) = json::parse(src) else { return s };
        let b = |k: &str, d: bool| { let x = v.get(k); if x.is_null() { d } else { x.truthy() } };
        if let Some(x) = v.get("lat").num() { s.lat = x }
        if let Some(x) = v.get("lon").num() { s.lon = x }
        if let Some(x) = v.get("topPct").num() { s.top_pct = x.clamp(0.0, 1.0) }
        if let Some(x) = v.get("focusMin").num() { s.focus_min = (x as i64).clamp(1, 180) }
        if let Some(x) = v.get("breakMin").num() { s.break_min = (x as i64).clamp(1, 60) }
        for (k, t) in [("city", &mut s.city), ("accent", &mut s.accent), ("ical", &mut s.ical), ("adzanWav", &mut s.adzan_wav)] {
            if let Some(x) = v.get(k).str() { *t = x.to_string() }
        }
        s.hover = b("hover", s.hover); s.light = b("light", s.light); s.adzan = b("adzan", s.adzan);
        s.src_claude = b("srcClaude", true); s.src_copilot = b("srcCopilot", true); s.src_agy = b("srcAgy", true); s.src_desk = b("srcDesk", true);
        s
    }
    pub fn to_json(&self) -> String {
        format!("{{\n \"lat\": {}, \"lon\": {}, \"city\": {}, \"hover\": {}, \"topPct\": {}, \"light\": {}, \"accent\": {},\n \"ical\": {}, \"adzan\": {}, \"adzanWav\": {}, \"focusMin\": {}, \"breakMin\": {},\n \"srcClaude\": {}, \"srcCopilot\": {}, \"srcAgy\": {}, \"srcDesk\": {}\n}}\n",
            self.lat, self.lon, q(&self.city), self.hover, self.top_pct, self.light, q(&self.accent), q(&self.ical), self.adzan, q(&self.adzan_wav),
            self.focus_min, self.break_min, self.src_claude, self.src_copilot, self.src_agy, self.src_desk)
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
        assert_eq!(Settings::parse(&s.to_json()), s);
        assert_eq!(Settings::parse("rusak{"), Settings::default());
    }
}
