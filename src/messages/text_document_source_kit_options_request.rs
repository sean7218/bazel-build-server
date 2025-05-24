use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use crate::support_types::build_target::BuildTargetIdentifier;


#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentSourceKitOptionsRequest {
    pub text_document: TextDocumentIdentifier,
    pub target: BuildTargetIdentifier,
    pub language: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TextDocumentIdentifier {
    pub uri: Url
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentSourceKitOptionsResponse {
    pub compiler_arguments: Vec<String>,
    pub working_directory: Option<PathBuf>, // assuming this is /Users/Path-To-Root
    pub data: Option<Value>,
}



