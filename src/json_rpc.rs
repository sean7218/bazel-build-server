use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct JsonRpcRequest {
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