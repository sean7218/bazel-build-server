use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use std::io::Write;

#[derive(Debug, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct JsonRpcRequest {
    pub id: Option<Number>,
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    pub result: Value,
    pub id: Option<Number>,
}

impl JsonRpcResponse {
    pub fn new(id: Option<Number>, result: Value) -> Self {
        JsonRpcResponse {
            jsonrpc: "2.0",
            result,
            id,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct JsonRpcNotification {
    pub jsonrpc: &'static str,
    pub method: &'static str,
    pub params: Value,
}

impl JsonRpcNotification {
    pub fn new(method: &'static str, params: Value) -> Self {
        JsonRpcNotification {
            jsonrpc: "2.0",
            method,
            params,
        }
    }
}

pub fn send(response: &serde_json::Value, stdout: &mut std::io::StdoutLock<'static>) {
    let response_str = response.to_string();
    let response_len = response_str.len();

    if let Err(e) = write!(stdout, "Content-Length: {}\r\n", response_len) {
        println!("{:?}", e);
    }

    if let Err(e) = write!(stdout, "\r\n") {
        println!("{:?}", e);
    }

    if let Err(e) = write!(stdout, "{}", response_str) {
        println!("{:?}", e);
    }

    stdout.flush().expect("stdout flush failed");
}
