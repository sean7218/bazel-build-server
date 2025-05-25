use crate::{error::Result};
use serde::{Deserialize, Serialize};
use serde_json::{to_value, Number, Value};
use std::io::{self, BufRead, BufReader, Read, Write};

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
    pub method: String,
    pub params: Value,
}

impl JsonRpcNotification {
    pub fn new(method: String, params: Value) -> Self {
        JsonRpcNotification {
            jsonrpc: "2.0",
            method: method,
            params,
        }
    }
}

pub fn send_response(response: &JsonRpcResponse, stdout: &mut std::io::StdoutLock<'static>) {
    let value = to_value(&response).unwrap();
    send(&value, stdout);
}

pub fn send_notification(response: &JsonRpcNotification, stdout: &mut std::io::StdoutLock<'static>) {
    let value = to_value(&response).unwrap();
    send(&value, stdout);
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

pub fn read_request(reader: &mut BufReader<io::StdinLock<'static>>) -> Result<JsonRpcRequest> {
    let mut content_length = None;
    let mut buffer = String::new();
    loop {
        buffer.clear();
        let bytes = reader.read_line(&mut buffer)?;
        if bytes == 0 {
            return Err("eof -> exiting".into()); // EOF
        }

        if buffer == "\r\n" {
            break; // End of headers
        }

        if let Some(colon_position) = buffer.find(":") {
            let (key, value) = buffer.split_at(colon_position);
            if key.eq_ignore_ascii_case("Content-Length") {
                content_length = value[1..].trim().parse::<usize>().ok();
            }
        }
    }

    let content_length = match content_length {
        Some(len) => len,
        None => return Err("Missing Content-Length header".into())
    };

    let mut body: Vec<u8> = vec![0; content_length];
    reader.read_exact(&mut body)?;
    let request: JsonRpcRequest = serde_json::from_slice(&body)?;

    Ok(request)
}
