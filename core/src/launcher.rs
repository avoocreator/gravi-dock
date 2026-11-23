//! Pencarian peluncur: skor fuzzy dan kalkulator aman (tanpa eval).

pub fn fuzzy(q: &str, name: &str) -> i32 {
    let q = q.to_lowercase();
    let n = name.to_lowercase();
    if q.is_empty() { return 0; }
    if n == q { return 1000; }
    if n.starts_with(&q) { return 800 - n.chars().count() as i32; }
    if let Some(i) = n.find(&format!(" {q}")) { return 600 - i as i32; }
    if let Some(i) = n.find(&q) { return 400 - i as i32; }
    let nc: Vec<char> = n.chars().collect();
    let qc: Vec<char> = q.chars().collect();
    let (mut qi, mut score, mut prev) = (0usize, 0i32, -2i32);
    for (k, c) in nc.iter().enumerate() {
        if qi >= qc.len() { break; }
        if *c == qc[qi] {
            let word_start = k == 0 || nc[k - 1] == ' ' || nc[k - 1] == '-';
            score += 10 + if word_start { 14 } else { 0 } + if k as i32 == prev + 1 { 8 } else { 0 };
            prev = k as i32;
            qi += 1;
        }
    }
    if qi == qc.len() { (score - (nc.len() as i32 - qc.len() as i32)).max(1) } else { 0 }
}

pub fn rank<'a, T>(items: &'a [T], name: impl Fn(&T) -> &str, q: &str, limit: usize) -> Vec<&'a T> {
    let mut scored: Vec<(i32, &T)> = items.iter().map(|a| (fuzzy(q, name(a)), a)).filter(|(s, _)| *s > 0).collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.into_iter().take(limit).map(|x| x.1).collect()
}

/// Kalkulator: + - * / ^ % ( ) dan sqrt(); koma desimal diterima. None bila bukan ekspresi.
pub fn calc(input: &str) -> Option<f64> {
    let mut src: String = input.trim().chars().filter(|c| !c.is_whitespace()).collect();
    src = src.replace(['×', 'x'], "*").replace(['÷', ':'], "/").replace(',', ".");
    let has_op = src.trim_start_matches('-').chars().any(|c| "+-*/^%()".contains(c)) || src.contains("sqrt");
    if !has_op { return None; }
    let mut toks: Vec<String> = Vec::new();
    let b: Vec<char> = src.chars().collect();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if c.is_ascii_digit() || c == '.' {
            let st = i;
            while i < b.len() && (b[i].is_ascii_digit() || b[i] == '.') { i += 1; }
            toks.push(b[st..i].iter().collect());
        } else if b[i..].iter().take(4).collect::<String>() == "sqrt" {
            toks.push("sqrt".into());
            i += 4;
        } else if "+-*/^%()".contains(c) {
            toks.push(c.to_string());
            i += 1;
        } else {
            return None;
        }
    }
    let mut p = Parser { t: &toks, i: 0 };
    let v = p.expr()?;
    if p.i == toks.len() && v.is_finite() { Some(v) } else { None }
}

struct Parser<'a> { t: &'a [String], i: usize }
impl Parser<'_> {
    fn peek(&self) -> Option<&str> { self.t.get(self.i).map(|s| s.as_str()) }
    fn expr(&mut self) -> Option<f64> {
        let mut v = self.term()?;
        while matches!(self.peek(), Some("+") | Some("-")) {
            let op = self.t[self.i].clone(); self.i += 1;
            let r = self.term()?;
            v = if op == "+" { v + r } else { v - r };
        }
        Some(v)
    }
    fn term(&mut self) -> Option<f64> {
        let mut v = self.pow()?;
        while matches!(self.peek(), Some("*") | Some("/") | Some("%")) {
            let op = self.t[self.i].clone(); self.i += 1;
            let r = self.pow()?;
            v = match op.as_str() { "*" => v * r, "/" => v / r, _ => v % r };
        }
        Some(v)
    }
    fn pow(&mut self) -> Option<f64> {
        let b = self.unary()?;
        if self.peek() == Some("^") { self.i += 1; let e = self.pow()?; return Some(b.powf(e)); }
        Some(b)
    }
    fn unary(&mut self) -> Option<f64> {
        match self.peek() {
            Some("-") => { self.i += 1; Some(-self.unary()?) }
            Some("+") => { self.i += 1; self.unary() }
            _ => self.atom(),
        }
    }
    fn atom(&mut self) -> Option<f64> {
        let t = self.t.get(self.i)?.clone();
        self.i += 1;
        match t.as_str() {
            "(" => { let v = self.expr()?; if self.peek() == Some(")") { self.i += 1; Some(v) } else { None } }
            "sqrt" => {
                if self.peek() != Some("(") { return None; }
                self.i += 1;
                let v = self.expr()?;
                if self.peek() == Some(")") { self.i += 1; Some(v.sqrt()) } else { None }
            }
            n => n.parse::<f64>().ok(),
        }
    }
}

pub fn detect_url(q: &str) -> Option<String> {
    let t = q.trim();
    if t.is_empty() || t.contains(char::is_whitespace) { return None; }
    let l = t.to_lowercase();
    if l.starts_with("http://") || l.starts_with("https://") { return Some(t.to_string()); }
    let host = t.split('/').next().unwrap_or("");
    let tlds = [".com", ".net", ".org", ".id", ".co.id", ".io", ".dev", ".app", ".ai", ".me", ".tv", ".xyz", ".co", ".edu", ".gov"];
    if host.contains('.') && tlds.iter().any(|d| host.to_lowercase().ends_with(d)) { Some(format!("https://{t}")) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fuzzy_urutan() {
        assert!(fuzzy("chrome", "Google Chrome") > fuzzy("chm", "Google Chrome"));
        assert!(fuzzy("vsc", "Visual Studio Code") > 0);
        assert_eq!(fuzzy("zzz", "Notepad"), 0);
        assert_eq!(fuzzy("notepad", "Notepad"), 1000);
    }
    #[test]
    fn kalkulator() {
        assert_eq!(calc("2+3*4"), Some(14.0));
        assert_eq!(calc("(2+3)*4"), Some(20.0));
        assert_eq!(calc("2^3^2"), Some(512.0));
        assert_eq!(calc("sqrt(16)+1"), Some(5.0));
        assert_eq!(calc("1,5*2"), Some(3.0));
        assert_eq!(calc("chrome"), None);
        assert_eq!(calc("5"), None);
        assert_eq!(calc("2+"), None);
        assert_eq!(calc("1/0"), None);
    }
    #[test]
    fn url() {
        assert_eq!(detect_url("github.com/x").as_deref(), Some("https://github.com/x"));
        assert_eq!(detect_url("halo dunia"), None);
        assert!(detect_url("https://a.b").is_some());
    }
}
