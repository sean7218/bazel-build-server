#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use serde_json::{Value, to_value};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeBuildRequest {
    /// Name of the client
    pub display_name: String,

    /// The version of the client
    pub version: String,

    /// The BSP version tht client speaks to
    pub bsp_version: String,

    /// The rootUri of the workspace
    pub root_uri: String,

    /// The capabilities of the client
    pub capabilities: BuildClientCapabilities,

    /// Kind of data expected in `data` field
    pub data_kind: Option<String>,

    /// Additional metadata about the client
    pub data: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildClientCapabilities {
    /// The languages that this client supports.
    /// The ID strings for each language is defined in the LSP.
    /// The server must never respond with build targets for other
    /// languages than those that appear in this list.
    pub language_ids: Vec<String>,
}

/// if data_kind = "sourceKit", then the following
/// `data` field must contain a `SourceKitInitializeBuildResponseData` object.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeBuildResponse {
    pub display_name: &'static str,
    pub version: &'static str,
    pub bsp_version: &'static str,
    pub capabilities: BuildServerCapabilities,
    pub data_kind: Option<&'static str>,
    pub data: Option<Value>,
}

impl InitializeBuildResponse {
    pub fn new(
        display_name: &'static str,
        version: &'static str,
        bsp_version: &'static str,
        capabilities: BuildServerCapabilities,
        data_kind: &'static str,
        data: SourceKitInitializeBuildResponseData,
    ) -> Self {
        InitializeBuildResponse {
            display_name,
            version,
            bsp_version,
            capabilities,
            data_kind: Some(data_kind),
            data: Some(data.into()),
        }
    }
}

impl From<InitializeBuildResponse> for Value {
    fn from(value: InitializeBuildResponse) -> Self {
        to_value(value).expect("Failed to serialize InitializeBuildResponse")
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildServerCapabilities {
    /// The languages the server supports compilation via method buildTarget/compile.
    pub compile_provider: Option<CompileProvider>,

    /// The languages the server supports test execution via method buildTarget/test
    pub test_provider: Option<TestProvider>,

    /// The languages the server supports run via method buildTarget/run
    pub run_provider: Option<RunProvider>,

    /// The languages the server supports debugging via method debugSession/start.
    pub debug_provider: Option<DebugProvider>,

    /// The server can provide a list of targets that contain a
    /// single text document via the method buildTarget/inverseSources
    pub inverse_sources_provider: Option<bool>,

    /// The server provides sources for library dependencies
    /// via method buildTarget/dependencySources
    pub dependency_sources_provider: Option<bool>,

    /// The server provides all the resource dependencies
    /// via method buildTarget/resources
    pub resources_provider: Option<bool>,

    /// The server provides all output paths
    /// via method buildTarget/outputPaths
    pub output_paths_provider: Option<bool>,

    /// The server sends notifications to the client on build
    /// target change events via `buildTarget/didChange`
    pub build_target_changed_provider: Option<bool>,

    /// Reloading the build state through workspace/reload is supported
    pub can_reload: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceKitInitializeBuildResponseData {
    /// The directory to which the index store is written during compilation, ie. the path passed to `-index-store-path`
    /// for `swiftc` or `clang` invocations
    pub index_database_path: Option<String>,

    /// The path at which SourceKit-LSP can store its index database, aggregating data from `indexStorePath`
    pub index_store_path: Option<String>,

    /// Whether the server implements the `buildTarget/outputPaths` request.
    pub output_paths_provider: Option<bool>,

    /// Whether the build server supports the `buildTarget/prepare` request.
    pub prepare_provider: Option<bool>,

    /// Whether the server implements the `textDocument/sourceKitOptions` request.
    pub source_kit_options_provider: Option<bool>,
}

impl From<SourceKitInitializeBuildResponseData> for Value {
    fn from(value: SourceKitInitializeBuildResponseData) -> Self {
        let value = match to_value(value) {
            Ok(v) => v,
            Err(e) => {
                println!("Failed to serialize SourceKitInitializeBuildResponseData");
                println!("{}", e);
                Value::Null
            }
        };
        return value;
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileProvider {
    pub language_ids: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestProvider {
    pub language_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunProvider {
    pub language_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugProvider {
    pub language_ids: Vec<String>,
}
