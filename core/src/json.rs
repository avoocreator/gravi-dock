//! Parser JSON kecil (cukup untuk membaca baris .jsonl dan berkas pengaturan).

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Value>),
    Obj(Vec<(String, Value)>),
}

impl Value {
    pub fn get(&self, key: &str) -> &Value {
        static NULL: Value = Value::Null;
        if let Value::Obj(kv) = self {
            for (k, v) in kv {
                if k == key {
                    return v;
                }
            }
        }
        &NULL
    }
    pub fn str(&self) -> Option<&str> {
        if let Value::Str(s) = self { Some(s) } else { None }
    }
    pub fn text(&self) -> &str {
        self.str().unwrap_or("")
    }
    pub fn arr(&self) -> &[Value] {
        if let Value::Arr(a) = self { a } else { &[] }
    }
    pub fn num(&self) -> Option<f64> {
        if let Value::Num(n) = self { Some(*n) } else { None }
    }
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
    pub fn truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Num(n) => *n != 0.0,
            Value::Str(s) => !s.is_empty(),
            _ => true,
        }
    }
}

pub fn parse(src: &str) -> Option<Value> {
    let mut p = P { s: src.as_bytes(), i: 0 };
    p.ws();
    let v = p.value(0)?;
    p.ws();
    if p.i == p.s.len() { Some(v) } else { None }
}

struct P<'a> {
    s: &'a [u8],
    i: usize,
}

impl<'a> P<'a> {
    fn ws(&mut self) {
        while self.i < self.s.len() && matches!(self.s[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }
    fn eat(&mut self, c: u8) -> bool {
        if self.s.get(self.i) == Some(&c) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn lit(&mut self, w: &str, v: Value) -> Option<Value> {
        if self.s[self.i..].starts_with(w.as_bytes()) {
            self.i += w.len();
            Some(v)
        } else {
            None
        }
    }
    fn value(&mut self, depth: usize) -> Option<Value> {
        if depth > 64 {
            return None;
        }
        match *self.s.get(self.i)? {
            b'{' => {
                self.i += 1;
                let mut kv = Vec::new();
                self.ws();
                if self.eat(b'}') {
                    return Some(Value::Obj(kv));
                }
                loop {
                    self.ws();
                    let k = self.string()?;
                    self.ws();
                    if !self.eat(b':') {
                        return None;
                    }
                    self.ws();
                    let v = self.value(depth + 1)?;
                    kv.push((k, v));
                    self.ws();
                    if self.eat(b',') {
                        continue;
                    }
                    if self.eat(b'}') {
                        return Some(Value::Obj(kv));
                    }
                    return None;
                }
            }
            b'[' => {
                self.i += 1;
                let mut a = Vec::new();
                self.ws();
                if self.eat(b']') {
                    return Some(Value::Arr(a));
                }
                loop {
                    self.ws();
                    a.push(self.value(depth + 1)?);
                    self.ws();
                    if self.eat(b',') {
                        continue;
                    }
                    if self.eat(b']') {
                        return Some(Value::Arr(a));
                    }
                    return None;
                }
            }
            b'"' => self.string().map(Value::Str),
            b't' => self.lit("true", Value::Bool(true)),
            b'f' => self.lit("false", Value::Bool(false)),
            b'n' => self.lit("null", Value::Null),
            _ => self.number(),
        }
    }
    fn number(&mut self) -> Option<Value> {
        let st = self.i;
        while self.i < self.s.len() && matches!(self.s[self.i], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9') {
            self.i += 1;
        }
        std::str::from_utf8(&self.s[st..self.i]).ok()?.parse::<f64>().ok().map(Value::Num)
    }
    fn hex4(&mut self) -> Option<u32> {
        let h = std::str::from_utf8(self.s.get(self.i..self.i + 4)?).ok()?;
        self.i += 4;
        u32::from_str_radix(h, 16).ok()
    }
    fn string(&mut self) -> Option<String> {
        if !self.eat(b'"') {
            return None;
        }
        let mut out: Vec<u8> = Vec::new();
        loop {
            let c = *self.s.get(self.i)?;
            self.i += 1;
            match c {
                b'"' => break,
                b'\\' => {
                    let e = *self.s.get(self.i)?;
                    self.i += 1;
                    match e {
                        b'n' => out.push(b'\n'),
                        b't' => out.push(b'\t'),
                        b'r' => out.push(b'\r'),
                        b'b' => out.push(8),
                        b'f' => out.push(12),
                        b'/' | b'\\' | b'"' => out.push(e),
                        b'u' => {
                            let mut cp = self.hex4()?;
                            if (0xD800..0xDC00).contains(&cp) && self.s[self.i..].starts_with(b"\\u") {
                                self.i += 2;
                                let lo = self.hex4()?;
                                cp = 0x10000 + ((cp - 0xD800) << 10) + (lo.wrapping_sub(0xDC00) & 0x3FF);
                            }
                            let ch = char::from_u32(cp).unwrap_or('\u{FFFD}');
                            let mut b = [0u8; 4];
                            out.extend_from_slice(ch.encode_utf8(&mut b).as_bytes());
                        }
                        _ => return None,
                    }
                }
                _ => out.push(c),
            }
        }
        Some(String::from_utf8_lossy(&out).into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_nested_and_escapes() {
        let v = parse(r#"{"a":[1,2.5,{"b":"x\nyé😀"}],"t":true,"n":null}"#).unwrap();
        assert_eq!(v.get("a").arr().len(), 3);
        assert_eq!(v.get("a").arr()[1].num(), Some(2.5));
        assert_eq!(v.get("a").arr()[2].get("b").text(), "x\nyé😀");
        assert!(v.get("t").truthy());
        assert!(v.get("n").is_null());
        assert!(v.get("zzz").is_null());
    }
    #[test]
    fn rejects_garbage() {
        assert!(parse("{potong").is_none());
        assert!(parse("bukan json").is_none());
        assert!(parse("").is_none());
        assert!(parse("[1,2] x").is_none());
    }
}

/* ---- penulis JSON ---- */

/// Escape string untuk JSON (termasuk tanda kutip pembuka/penutup).
pub fn quote(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

impl Value {
    pub fn obj(kv: Vec<(&str, Value)>) -> Value {
        Value::Obj(kv.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
    }
    pub fn s(x: &str) -> Value { Value::Str(x.to_string()) }
    pub fn n(x: f64) -> Value { Value::Num(x) }
    pub fn b(x: bool) -> Value { Value::Bool(x) }

    /// Setel/ganti kunci pada objek (tidak melakukan apa-apa bila bukan objek).
    pub fn set(&mut self, key: &str, v: Value) {
        if let Value::Obj(kv) = self {
            if let Some(slot) = kv.iter_mut().find(|(k, _)| k == key) { slot.1 = v; } else { kv.push((key.to_string(), v)); }
        }
    }
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        if let Value::Obj(kv) = self { kv.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v) } else { None }
    }

    /// Serialisasi ringkas.
    pub fn to_json(&self) -> String {
        match self {
            Value::Null => "null".into(),
            Value::Bool(b) => b.to_string(),
            Value::Num(n) => if n.fract() == 0.0 && n.abs() < 1e15 { format!("{}", *n as i64) } else { format!("{}", n) },
            Value::Str(s) => quote(s),
            Value::Arr(a) => format!("[{}]", a.iter().map(|v| v.to_json()).collect::<Vec<_>>().join(",")),
            Value::Obj(kv) => format!("{{{}}}", kv.iter().map(|(k, v)| format!("{}:{}", quote(k), v.to_json())).collect::<Vec<_>>().join(",")),
        }
    }

    /// Serialisasi berindentasi (untuk berkas yang dibaca manusia).
    pub fn pretty(&self) -> String {
        fn go(v: &Value, ind: usize, out: &mut String) {
            let pad = "  ".repeat(ind + 1);
            let end = "  ".repeat(ind);
            match v {
                Value::Arr(a) if !a.is_empty() => {
                    out.push_str("[\n");
                    for (i, x) in a.iter().enumerate() { out.push_str(&pad); go(x, ind + 1, out); if i + 1 < a.len() { out.push(','); } out.push('\n'); }
                    out.push_str(&end); out.push(']');
                }
                Value::Obj(kv) if !kv.is_empty() => {
                    out.push_str("{\n");
                    for (i, (k, x)) in kv.iter().enumerate() { out.push_str(&pad); out.push_str(&quote(k)); out.push_str(": "); go(x, ind + 1, out); if i + 1 < kv.len() { out.push(','); } out.push('\n'); }
                    out.push_str(&end); out.push('}');
                }
                other => out.push_str(&other.to_json()),
            }
        }
        let mut s = String::new();
        go(self, 0, &mut s);
        s.push('\n');
        s
    }
}

#[cfg(test)]
mod writer_tests {
    use super::*;
    #[test]
    fn roundtrip_writer() {
        let v = Value::obj(vec![("a", Value::s("x\"y\n")), ("n", Value::n(2.5)), ("i", Value::n(7.0)), ("l", Value::Arr(vec![Value::b(true), Value::Null]))]);
        assert_eq!(parse(&v.to_json()).unwrap(), v);
        assert_eq!(parse(&v.pretty()).unwrap(), v);
    }
    #[test]
    fn set_replaces_or_adds() {
        let mut v = Value::obj(vec![("a", Value::n(1.0))]);
        v.set("a", Value::n(2.0)); v.set("b", Value::b(true));
        assert_eq!(v.get("a").num(), Some(2.0));
        assert!(v.get("b").truthy());
    }
}
