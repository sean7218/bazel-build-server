use std::io::{self, BufRead, Write};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Value,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct JsonRpcResponse{
    jsonrpc: &'static str,
    result: Value,
    id: i32,
}

fn main() -> io::Result<()> {
    env_logger::Builder::from_default_env()
        .target(env_logger::Target::Stdout)
        .init();

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdin_lock = stdin.lock();
    let mut stdout_lock = stdout.lock();

    loop {
        let mut line = String::new();
        if let Ok(_) = stdin_lock.read_line(&mut line) {
            if line.is_empty() == false {
                let res = JsonRpcResponse { jsonrpc: "2.0", result: "{}".into(), id: 1 };
                if let Ok(txt) = serde_json::to_string(&res) {
                    let b = txt.as_bytes();
                    let _ = stdout_lock.write_all(b);
                }
                break
            }
        }
    }

    return Ok(())
}
