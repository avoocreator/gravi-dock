use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

const IPC_ADDR: &str = "127.0.0.1:47891";

#[derive(Serialize)]
struct HookPayload {
    event_type: String,
    agent: String,
    tool: Option<String>,
    command: Option<String>,
    summary: Option<String>,
    cwd: Option<String>,
}

#[derive(Deserialize)]
struct DecisionResp {
    decision: Option<String>,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut agent = "antigravity-cli".to_string();
    let mut event_type = "activity".to_string();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--agent" if i + 1 < args.len() => {
                agent = args[i + 1].clone();
                i += 2;
            }
            "--event" if i + 1 < args.len() => {
                event_type = args[i + 1].clone();
                i += 2;
            }
            _ => i += 1,
        }
    }

    let mut stdin_buf = String::new();
    let _ = io::stdin().read_to_string(&mut stdin_buf);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdin_buf).unwrap_or_else(|_| serde_json::json!({}));

    let command = parsed["tool_input"]["command"]
        .as_str()
        .or_else(|| parsed["command"].as_str())
        .map(|s| s.to_string());
    let tool = parsed["tool_name"]
        .as_str()
        .or_else(|| parsed["tool"].as_str())
        .map(|s| s.to_string());
    let summary = parsed["summary"].as_str().map(|s| s.to_string());
    let cwd = std::env::current_dir()
        .ok()
        .map(|p| p.to_string_lossy().to_string());

    let addr: SocketAddr = IPC_ADDR.parse().unwrap();
    // Fail-open immediately if Nobi Island is not running
    let Ok(mut stream) = TcpStream::connect_timeout(&addr, Duration::from_millis(250)) else {
        println!("{{\"decision\":\"allow\"}}");
        return;
    };

    let payload = HookPayload {
        event_type: event_type.clone(),
        agent,
        tool,
        command,
        summary,
        cwd,
    };

    if let Ok(line) = serde_json::to_string(&payload) {
        let _ = writeln!(stream, "{}", line);
        let _ = stream.flush();
    }

    let mut reader = BufReader::new(stream);
    let mut resp_line = String::new();
    if reader.read_line(&mut resp_line).is_ok() {
        if let Ok(resp) = serde_json::from_str::<DecisionResp>(&resp_line) {
            if resp.decision.as_deref() == Some("deny") {
                println!("{{\"decision\":\"deny\"}}");
                std::process::exit(2);
            }
        }
    }

    println!("{{\"decision\":\"allow\"}}");
}
