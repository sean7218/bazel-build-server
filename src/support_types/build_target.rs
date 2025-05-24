#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildTarget {
    pub id: BuildTargetIdentifier,
    pub display_name: Option<String>,
    pub base_directory: Option<Url>, /// file uri
    pub tags: Vec<String>, /// tags can be ["application", "test", "library"]
    pub language_ids: Vec<String>,
    pub dependencies: Vec<BuildTargetIdentifier>,
    pub capabilities: BuildTargetCapabilities,
    /// if data_kind = "sourceKit" then `data` field must contain a 
    /// SourceKitBuildTarget object.
    pub data_kind: Option<String>,
    pub data: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildTargetIdentifier {
    pub uri: Url,
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
    /// The toolchain that should be used to build this target. The URI should point to the directory that contains the
    /// `usr` directory. On macOS, this is typically a bundle ending in `.xctoolchain`. If the toolchain is installed to
    /// `/` on Linux, the toolchain URI would point to `/`.
    ///
    /// If no toolchain is given, SourceKit-LSP will pick a toolchain to use for this target.
    pub toolchain: Url,
}

