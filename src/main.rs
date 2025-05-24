mod aquery;
mod json_rpc;
mod messages;
mod support_types;
mod utils;
mod error;

use crate::error::{Error, Result};
use aquery::BazelTarget;
use build_server_config::BuildServerConfig;
use json_rpc::{
    read_request, send, send_notification, send_response, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse
};
use messages::{build_target_sources_request::{BuildTargetSourcesRequest, BuildTargetSourcesResponse, SourceItem, SourcesItem}, initialize_build_request::{
    BuildServerCapabilities, CompileProvider, InitializeBuildRequest, InitializeBuildResponse,
    SourceKitInitializeBuildResponseData,
}, text_document_source_kit_options_request::{TextDocumentSourceKitOptionsRequest, TextDocumentSourceKitOptionsResponse}};
use serde_json::{self, from_value, to_value};
use std::{
    io::{self, BufReader, StdoutLock}, path::PathBuf
};
use support_types::{
    build_server_config, build_target::{BuildTarget, BuildTargetCapabilities, BuildTargetIdentifier}, methods::RequestMethod
};

fn main() -> Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut reader = BufReader::new(stdin.lock());

    let mut request_handler = match handle_initialize_request(&mut reader, &mut stdout) {
        Ok(v) => v,
        Err(e) => {
            log_str!("👻 handle_initialize_request failed. \n {:#?}", &e);
            return Ok(());
        }
    };

    log_str!("🟢 Build Server Initialized");

    loop {
        let request = match read_request(&mut reader) {
            Ok(v) => v,
            Err(e) => {
                log_str!("👻 read_request(&mut reader) failed. \n {:#?}", &e);
                return Ok(());
            }
        };

        log_str!("➡️ {:#?}", &request);

        match RequestMethod::from_str(&request.method) {
            RequestMethod::BuildInitialized => {
                log_str!("🤩 oh yay: build server initialized");
            }
            RequestMethod::WorkspaceBuildTargets => {
                let response = request_handler.workspace_build_targets(request)?;
                send_response(&response, &mut stdout);
                log_str!("↩️ {:#?}", response);
            }
            RequestMethod::BuildTargetSources => {
                let response = request_handler.build_target_sources(request)?;
                send_response(&response, &mut stdout);
                log_str!("↩️ {:#?}", response);
            }
            RequestMethod::TextDocumentSourceKitOptions => {
                let response = request_handler.sourcekit_options(request)?;
                send_response(&response, &mut stdout);
                log_str!("↩️ {:#?}", response);
            }
            RequestMethod::TextDocumentRegisterForChanges => {
                // INFO: this endpoint is for legacy push-based model
                let response = Responses::options_changed();
                send_notification(&response, &mut stdout);
                log_str!("↩️ {:#?}", response);
            }
            RequestMethod::BuildTargetPrepare => {}
            RequestMethod::BuildTargetDidChange => {}
            RequestMethod::BuildShutDown => { return Ok(()) }
            RequestMethod::BuildExit => { return Ok(()) }
            RequestMethod::WindowShowMessage => {}
            RequestMethod::WorkspaceWaitForBuildSystemUpdates => {}
            RequestMethod::Unknown => {
                log_str!(&format!("🤷 Unkown request: {:#?}", request));
                return Ok(());
            }
        }
    }
}

fn handle_initialize_request(
    reader: &mut BufReader<io::StdinLock<'static>>,
    stdout: &mut StdoutLock<'static>,
) -> Result<RequestHandler> {
    match read_request(reader) {
        Ok(request) => {
            let request_handler = RequestHandler::initialize(&request)?;
            let response = request_handler.build_initialize(&request)?;
            let value = to_value(&response)?;
            send(&value, stdout);
            return Ok(request_handler);
        }
        Err(e) => return Err(e)
    }
}

struct RequestHandler {
    config: BuildServerConfig,
    root_path: PathBuf,
    targets: Vec<BazelTarget>
}

impl RequestHandler {
    fn initialize(request: &JsonRpcRequest) -> Result<Self> {
        let build_request: InitializeBuildRequest = from_value(request.params.clone())?;

        let config = BuildServerConfig::parse(&build_request.root_uri)
            .ok_or(Error::from("Failed to parse BuildServerConfig"))?;

        let root_path = build_request.root_uri
            .to_file_path()
            .map_err(|_| "Failed to convert root_uri to file path")?;

        Ok(RequestHandler { config, root_path, targets: vec![] })
    }

    fn build_initialize(&self, request: &JsonRpcRequest) -> Result<JsonRpcResponse> {
        let root_path = self.root_path.clone();
        let index_database_path = root_path.join(".index-db").to_string_lossy().into_owned();
        let index_store_path = root_path.join(".indexstore").to_string_lossy().into_owned();

        let result = InitializeBuildResponse::new(
            "bazel-build-server",
            "1.0.0",
            "2.0",
            BuildServerCapabilities {
                compile_provider: Some(CompileProvider {
                    language_ids: vec!["c", "cpp", "objective-c", "objective-cpp", "swift"],
                }),
                test_provider: None,
                run_provider: None,
                debug_provider: None,
                inverse_sources_provider: Some(true),
                dependency_sources_provider: Some(true),
                resources_provider: Some(true),
                output_paths_provider: Some(true),
                build_target_changed_provider: Some(true),
                can_reload: Some(false),
            },
            "sourceKit",
            SourceKitInitializeBuildResponseData {
                index_database_path: Some(index_database_path),
                index_store_path: Some(index_store_path),
                output_paths_provider: Some(true),
                prepare_provider: Some(true),
                source_kit_options_provider: Some(true),
            },
        );
        let value = to_value(result)?;
        let response = JsonRpcResponse::new(request.id.clone(), value);
        Ok(response)
    }

    fn workspace_build_targets(&mut self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let dir = &self.root_path;
        let target = &self.config.target;
        let targets = aquery::aquery(&target, &dir);

        let mut build_targets: Vec<BuildTarget> = vec![];

        for target in targets {
            let build_target: BuildTarget = target.clone().into();
            build_targets.push(build_target);
            self.targets.push(target);
        }

        let response = JsonRpcResponse {
            id: request.id,
            jsonrpc: "2.0",
            result: serde_json::json!({
                "targets": serde_json::to_value(build_targets)?
            })
        };

        Ok(response)
    }

    fn build_target_sources(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let source_request: BuildTargetSourcesRequest = serde_json::from_value(request.params)?;
        let mut items: Vec<SourcesItem> = vec![];

        for target in source_request.targets {
            let bazel_target = match self.targets
                .iter()
                .find(|t| t.uri.eq(&target.uri)) {
                    Some(v) => v,
                    None => { continue; }
                };

            let sources: Vec<SourceItem> = bazel_target.input_files
                .iter()
                .map(SourceItem::from_url)
                .collect();

            items.push(SourcesItem { target, sources });
        }

        let result = BuildTargetSourcesResponse { items };
        let resp = JsonRpcResponse {
            id: request.id,
            jsonrpc: "2.0",
            result: serde_json::to_value(result)?
        };
        Ok(resp)
    }

    fn sourcekit_options(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let req: TextDocumentSourceKitOptionsRequest = from_value(request.params)?;
        let target = match self.targets
            .iter()
            .find(|it| it.uri.eq(&req.target.uri)) {
                Some(v) => v,
                None => return Err("Failed to find target for sourcekit_options".into())
            };

        log_str!(">>> yes this is equal");

        let result = TextDocumentSourceKitOptionsResponse {
            compiler_arguments: target.compiler_arguments.clone(),
            working_directory: Some(self.root_path.clone()),
            data: None
        };

        let response = JsonRpcResponse {
            id: request.id,
            jsonrpc: "2.0",
            result: to_value(result)?
        };
        Ok(response)
    }
}

struct Responses {}

impl Responses {
    #[allow(dead_code)]
    fn options_changed() -> JsonRpcNotification {
        let params = serde_json::json!({
            "uri": "file:///Users/sean7218/bazel/hello-bazel/Sources/Components/Button.swift",
            "updatedOptions": {
                "workingDirectory": "/Users/sean7218/bazel/hello-bazel",
                "options": [
                    "-target",
                    "arm64-apple-macos15.4",
                    "-sdk",
                    "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk",
                    "-Ibazel-out/darwin_arm64-fastbuild/bin/Sources/Utils",
                    "-module-name",
                    "Components",
                    "-index-store-path",
                    "/Users/sean7218/bazel/hello-bazel/bazel-out/indexstore",
                    "Sources/Components/Button.swift",
                ]
            },
        });
        JsonRpcNotification::new("build/sourceKitOptionsChanged", params)
    }

    #[allow(dead_code)]
    fn did_change() -> JsonRpcNotification {
        let params = serde_json::json!({
            "changes": [
            {
                "target": {
                    "uri": "file:///Users/sean7218/bazel/hello-bazel/Sources/Components/Button.swift",
                },
                "kind": 2
            }
            ]
        });
        JsonRpcNotification::new("buildTarget/didChange", params)
    }

    #[allow(dead_code)]
    fn sourcekit_options(request: JsonRpcRequest) -> JsonRpcResponse {
        let uri = request.params["textDocument"]["uri"].as_str().unwrap();
        if uri.contains("AwesomeUtils.swift") {
            return JsonRpcResponse {
                id: request.id,
                jsonrpc: "2.0",
                result: serde_json::json!({
                    "compilerArguments": [
                        "-target",
                        "arm64-apple-macos15.4",
                        "-sdk",
                        "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk",
                        "-debug-prefix-map",
                        "__BAZEL_XCODE_DEVELOPER_DIR__=/PLACEHOLDER_DEVELOPER_DIR",
                        "-file-prefix-map",
                        "__BAZEL_XCODE_DEVELOPER_DIR__=/PLACEHOLDER_DEVELOPER_DIR",
                        "-emit-object",
                        "-output-file-map",
                        "bazel-out/darwin_arm64-fastbuild/bin/Sources/Utils/Utils.output_file_map.json",
                        "-Xfrontend",
                        "-no-clang-module-breadcrumbs",
                        "-emit-module-path",
                        "bazel-out/darwin_arm64-fastbuild/bin/Sources/Utils/Utils.swiftmodule",
                        "-enforce-exclusivity=checked",
                        "-emit-const-values-path",
                        "bazel-out/darwin_arm64-fastbuild/bin/Sources/Utils/Utils_objs/AwesomeUtils.swift.swiftconstvalues",
                        "-Xfrontend",
                        "-const-gather-protocols-file",
                        "-Xfrontend",
                        "external/rules_swift+/swift/toolchains/config/const_protocols_to_gather.json",
                        "-DDEBUG",
                        "-Onone",
                        "-Xfrontend",
                        "-internalize-at-link",
                        "-Xfrontend",
                        "-no-serialize-debugging-options",
                        "-enable-testing",
                        "-disable-sandbox",
                        "-gline-tables-only",
                        "-module-cache-path",
                        "/Users/sean7218/bazel/hello-bazel/bazel-out/darwin_arm64-fastbuild/bin/_swift_module_cache",
                        "-Xcc",
                        "-iquote.",
                        "-Xcc",
                        "-iquote/Users/sean7218/bazel/hello-bazel/bazel-out/darwin_arm64-fastbuild/bin",
                        "-Xfrontend",
                        "-color-diagnostics",
                        "-enable-batch-mode",
                        "-module-name",
                        "Utils",
                        "-file-prefix-map",
                        "__BAZEL_XCODE_DEVELOPER_DIR__=DEVELOPER_DIR",
                        "-enable-bare-slash-regex",
                        "-Xfrontend",
                        "-disable-clang-spi",
                        "-enable-experimental-feature",
                        "AccessLevelOnImport",
                        "-parse-as-library",
                        "-index-store-path",
                        "/Users/sean7218/bazel/hello-bazel/.indexstore",
                        "-static",
                        "-Xcc",
                        "-O0",
                        "-Xcc",
                        "-DDEBUG=1",
                        "/Users/sean7218/bazel/hello-bazel/Sources/Utils/AwesomeUtils.swift"

                            // "-target",
                            // "arm64-apple-macos15.4",
                            // "-sdk",
                            // "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk",
                            // "-swift-version",
                            // "6",
                            // "-module-name",
                            // "Utils",
                            // "-index-store-path",
                            // "/Users/sean7218/bazel/hello-bazel/.indexstore",
                            // "/Users/sean7218/bazel/hello-bazel/Sources/Utils/AwesomeUtils.swift"
                    ]
                }),
            };
        } else if uri.contains("Button.swift") {
            return JsonRpcResponse {
                id: request.id,
                jsonrpc: "2.0",
                result: serde_json::json!({
                    "compilerArguments": [
                        "-target",
                        "arm64-apple-macos15.4",
                        "-sdk",
                        "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk",
                        "-debug-prefix-map",
                        "__BAZEL_XCODE_DEVELOPER_DIR__=/PLACEHOLDER_DEVELOPER_DIR",
                        "-file-prefix-map",
                        "__BAZEL_XCODE_DEVELOPER_DIR__=/PLACEHOLDER_DEVELOPER_DIR",
                        "-emit-object",
                        "-output-file-map",
                        "/Users/sean7218/bazel/hello-bazel/bazel-out/darwin_arm64-fastbuild/bin/Sources/Components/Components.output_file_map.json",
                        "-Xfrontend",
                        "-no-clang-module-breadcrumbs",
                        "-emit-module-path",
                        "/Users/sean7218/bazel/hello-bazel/bazel-out/darwin_arm64-fastbuild/bin/Sources/Components/Components.swiftmodule",
                        "-enforce-exclusivity=checked",
                        "-emit-const-values-path",
                        "/Users/sean7218/bazel/hello-bazel/bazel-out/darwin_arm64-fastbuild/bin/Sources/Components/Components_objs/Button.swift.swiftconstvalues",
                        "-Xfrontend",
                        "-const-gather-protocols-file",
                        "-Xfrontend",
                        "external/rules_swift+/swift/toolchains/config/const_protocols_to_gather.json",
                        "-DDEBUG",
                        "-Onone",
                        "-Xfrontend",
                        "-internalize-at-link",
                        "-Xfrontend",
                        "-no-serialize-debugging-options",
                        "-enable-testing",
                        "-disable-sandbox",
                        "-gline-tables-only",
                        "-module-cache-path",
                        "/Users/sean7218/bazel/hello-bazel/bazel-out/darwin_arm64-fastbuild/bin/_swift_module_cache",
                        "-I/Users/sean7218/bazel/hello-bazel/bazel-out/darwin_arm64-fastbuild/bin/Sources/Utils",
                        "-Xcc",
                        "-iquote.",
                        "-Xcc",
                        "-iquote/Users/sean7218/bazel/hello-bazel/bazel-out/darwin_arm64-fastbuild/bin",
                        "-Xfrontend",
                        "-color-diagnostics",
                        "-enable-batch-mode",
                        "-module-name",
                        "Components",
                        "-file-prefix-map",
                        "__BAZEL_XCODE_DEVELOPER_DIR__=DEVELOPER_DIR",
                        "-enable-bare-slash-regex",
                        "-Xfrontend",
                        "-disable-clang-spi",
                        "-enable-experimental-feature",
                        "AccessLevelOnImport",
                        "-parse-as-library",
                        "-index-store-path",
                        "/Users/sean7218/bazel/hello-bazel/.indexstore",
                        "-static",
                        "-Xcc",
                        "-O0",
                        "-Xcc",
                        "-DDEBUG=1",
                        "/Users/sean7218/bazel/hello-bazel/Sources/Components/Button.swift"

                            // "-target",
                            // "arm64-apple-macos15.4",
                            // "-sdk",
                            // "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk",
                            // "-swift-version",
                            // "6",
                            // "-Ibazel-out/darwin_arm64-fastbuild/bin/Sources/Utils",
                            // "-module-name",
                            // "Components",
                            // "-index-store-path",
                            // "/Users/sean7218/bazel/hello-bazel/.indexstore",
                            // "/Users/sean7218/bazel/hello-bazel/Sources/Components/Button.swift"
                            ]
                }),
            };
        } else {
            return JsonRpcResponse {
                id: request.id,
                jsonrpc: "2.0",
                result: serde_json::json!({
                    "compilerArguments": []
                }),
            };
        }
    }
}

impl From<BazelTarget> for BuildTarget {
    fn from(value: BazelTarget) -> Self {
        BuildTarget {
            id: BuildTargetIdentifier {
                uri: value.uri,
            },
            display_name: Some(value.label),
            base_directory: None,
            tags: vec![],
            language_ids: vec![
                "c".to_string(),
                "cpp".to_string(),
                "objective-c".to_string(),
                "objective-cpp".to_string(),
                "swift".to_string(),
            ],
            dependencies: vec![],
            capabilities: BuildTargetCapabilities {
                can_compile: Some(true),
                can_test: Some(true),
                can_run: Some(false),
                can_debug: Some(false),
            },
            data_kind: None,
            data: None,
        }
    }
}

