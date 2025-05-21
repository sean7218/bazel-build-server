use std::path::{PathBuf};
use serde::{Deserialize, Serialize};
use serde_json::Value;


#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildTarget {
    pub id: BuildTargetIdentifier,
    pub display_name: String,
    pub base_directory: PathBuf,
    /// tags can be ["application", "test", "library"]
    pub tags: Vec<String>, 
    pub language_ids: Vec<String>,
    pub dependencies: Vec<BuildTargetIdentifier>,
    pub capabilities: BuildTargetCapabilities,
    /// if data_kind = "sourceKit" then
    /// `data` field must contain a SourceKitBuildTarget object.
    pub data_kind: String,
    pub data: Value
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildTargetIdentifier {
    pub uri: PathBuf
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildTargetCapabilities {
    /// This target can be compiled by the BSP server.
    pub can_compile: Option<bool>,
    /// This target can be tested by the BSP server.
    pub can_test: Option<bool>,
    /// This target can be run by the BSP server.
    pub can_run: Option<bool>,
    /// This target can be debugged by the BSP server.
    pub can_debug: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceKitBuildTarget {
    pub toolchain: PathBuf
}
