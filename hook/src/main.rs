//! gravi-hook: dipanggil CLI AI pada event permission. Pemakaian: `gravi-hook <claude|copilot>` (JSON event dibaca dari stdin).
//! Bila Gravi Island tidak berjalan / tidak menjawab, tidak mencetak apa pun dan keluar 0 -> CLI memakai alur izin normal.
#![cfg_attr(windows, windows_subsystem = "console")]

use island_core::hookproto::{self, Decision, Endpoint, Request, Src};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

fn ask_island(req: &Request, ep: &Endpoint) -> Option<Decision> {
    let addr: SocketAddr = ([127, 0, 0, 1], ep.port).into();
    let mut s = TcpStream::connect_timeout(&addr, Duration::from_millis(800)).ok()?;
    s.set_read_timeout(Some(Duration::from_secs(hookproto::WAIT_SECS + 5))).ok()?;
    s.set_write_timeout(Some(Duration::from_secs(3))).ok()?;
    writeln!(s, "{}", req.to_line()).ok()?;
    s.flush().ok()?;
    let mut line = String::new();
    BufReader::new(s).read_line(&mut line).ok()?;
    if line.trim().is_empty() { None } else { Some(Decision::parse_line(&line)) }
}

fn main() {
    let Some(src) = std::env::args().nth(1).and_then(|a| Src::parse(&a)) else {
        eprintln!("pemakaian: gravi-hook <claude|copilot>");
        return;
    };
    let mut stdin = String::new();
    let _ = std::io::stdin().read_to_string(&mut stdin);
    let input = hookproto::parse_input(&stdin);

    let Some(ep) = std::fs::read_to_string(hookproto::data_dir().join("hook.json")).ok().and_then(|t| Endpoint::parse(&t)) else { return };
    let id = format!("{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0));
    let req = Request { token: ep.token.clone(), src: src.as_str().into(), id, session: input.session, tool: input.tool, summary: input.summary, cwd: input.cwd };

    if let Some(d) = ask_island(&req, &ep) {
        let out = hookproto::render_output(src, &d);
        if !out.is_empty() {
            println!("{out}");
        }
    }
}
