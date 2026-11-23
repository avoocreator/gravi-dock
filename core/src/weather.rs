//! Cuaca dari open-meteo (`current_weather`). Pengambilan HTTP dilakukan app lewat curl.exe; di sini hanya URL + parser.
use crate::json;

pub fn url(lat: f64, lon: f64) -> String {
    format!("https://api.open-meteo.com/v1/forecast?latitude={lat:.4}&longitude={lon:.4}&current_weather=true")
}

pub fn describe(code: i64) -> &'static str {
    match code {
        0 => "Cerah", 1..=2 => "Cerah berawan", 3 => "Berawan", 45 | 48 => "Berkabut",
        51..=57 => "Gerimis", 61..=67 | 80..=82 => "Hujan", 71..=77 | 85 | 86 => "Salju", 95..=99 => "Badai petir", _ => "—",
    }
}

/// -> (suhu °C, deskripsi)
pub fn parse(src: &str) -> Option<(f64, &'static str)> {
    let v = json::parse(src)?;
    let c = v.get("current_weather");
    Some((c.get("temperature").num()?, describe(c.get("weathercode").num()? as i64)))
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_ok() {
        let (t, d) = super::parse(r#"{"current_weather":{"temperature":24.5,"weathercode":61}}"#).unwrap();
        assert_eq!((t, d), (24.5, "Hujan"));
        assert!(super::parse("{}").is_none());
    }
}
