use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use std::io::Write;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct JsonRpcRequest {
    pub id: Option<Number>,
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct JsonRpcResponse{
    pub jsonrpc: &'static str,
    pub result: Value,
    pub id: i32,
}

pub fn send(response: &serde_json::Value, stdout: &mut std::io::StdoutLock<'static>) -> std::io::Result<()> {
  let response_str = response.to_string();
  let response_len = response_str.len();

  write!(stdout, "Content-Length: {}\r\n", response_len)?;
  write!(stdout, "\r\n")?;
  write!(stdout, "{}", response_str)?;
  stdout.flush()?;

  // writeln!(file, "buildserver | response send | {}", response_str)?;
  Ok(())
}