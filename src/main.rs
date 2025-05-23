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
    JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, read_request, send, send_response,
};
use messages::initialize_build_request::{
    BuildServerCapabilities, CompileProvider, InitializeBuildRequest, InitializeBuildResponse,
    SourceKitInitializeBuildResponseData,
};
use serde_json::{self, from_value, to_value};
use std::{
    io::{self, BufReader, StdoutLock},
    path::PathBuf,
};
use support_types::{
    build_server_config, build_target::{BuildTarget, BuildTargetCapabilities, BuildTargetIdentifier}, methods::RequestMethod
};
use url::Url;

fn handle_initialize_request(
    reader: &mut BufReader<io::StdinLock<'static>>,
    stdout: &mut StdoutLock<'static>,
) -> Result<RequestHandler> {
    match read_request(reader) {
        Ok(request) => {
            let request_handler = RequestHandler::initialize(&request)?;
            let response = request_handler.build_initialize(&request);
            let value = to_value(&response)?;
            send(&value, stdout);
            return Ok(request_handler);
        }
        Err(e) => {
            log_str!("Failed to read_request");
            return Err(e);
        }
    }
}

fn main() -> Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut reader = BufReader::new(stdin.lock());

    let request_handler = match handle_initialize_request(&mut reader, &mut stdout) {
        Ok(v) => v,
        Err(e) => {
            log_str!("[Error] Build server crashed due to invalid initial request");
            log_debug!(&e);
            return Ok(());
        }
    };

    log_str!("🟢 Build Server Initialized");

    loop {
        let request = match read_request(&mut reader) {
            Ok(v) => v,
            Err(e) => {
                log_debug!(&e);
                continue;
            }
        };

        log_str!("➡️ {:#?}", &request);

        match RequestMethod::from_str(&request.method) {
            RequestMethod::BuildInitialized => {
                log_str!("[success] 🤩 oh yay: build server initialized");
            }
            RequestMethod::WorkspaceBuildTargets => {
                let response = request_handler.workspace_build_targets(request);
                send_response(&response, &mut stdout);
                log_debug!(&response);
            }
            RequestMethod::BuildTargetSources => {
                let response = Responses::target_sources(request.id);
                send_response(&response, &mut stdout);
                log_debug!(&response);
            }
            RequestMethod::TextDocumentSourceKitOptions => {
                let response = Responses::sourcekit_options(request);
                send_response(&response, &mut stdout);
                log_debug!(&response);
            }
            RequestMethod::TextDocumentRegisterForChanges => {
                // INFO: this endpoint is for push model (legacy)
                let response = Responses::options_changed();
                let value = to_value(&response)?;
                send(&value, &mut stdout);
                log_debug!(&response);
            }
            RequestMethod::BuildTargetPrepare => {}
            RequestMethod::BuildTargetDidChange => {}
            RequestMethod::BuildShutDown => {}
            RequestMethod::BuildExit => {}
            RequestMethod::WindowShowMessage => {}
            RequestMethod::WorkspaceWaitForBuildSystemUpdates => {}
            RequestMethod::Unknown => {
                log_str!(&format!("[Warn] 🤷 Unkown request: {:?}", request));
                return Ok(());
            }
        }
    }
}

struct RequestHandler {
    config: BuildServerConfig,
    root_path: PathBuf,
}

impl RequestHandler {
    fn initialize(request: &JsonRpcRequest) -> Result<Self> {
        let build_request: InitializeBuildRequest = from_value(request.params.clone())?;

        let config = BuildServerConfig::parse(&build_request.root_uri)
            .ok_or(Error::from("Failed to parse BuildServerConfig"))?;

        let root_url = Url::parse(&build_request.root_uri)?;

        let root_path = root_url
            .to_file_path()
            .map_err(|_| "Failed to convert root_uri to file path")?;

        Ok(RequestHandler { config, root_path })
    }

    fn root_string(&self) -> String {
        let root = self.root_path.clone();
        root.to_string_lossy().into_owned()
    }

    fn build_initialize(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
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
        let value = to_value(result).expect("Failed to serialize InitializeBuildResponse.");
        JsonRpcResponse::new(request.id.clone(), value)
    }

    fn workspace_build_targets(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let dir = PathBuf::from("/Users/sean7218/bazel/buildserver/example/");
        let targets = aquery::aquery("//Sources/Components", &dir);
        log_debug!(&self.root_path);
        // let mut build_targets: Vec<BuildTarget> = vec![];
        // for target in targets {
        //     let build_target: BuildTarget = target.into();
        //     build_targets.push(build_target);
        // }
        // log_debug!(&build_targets);
        // return JsonRpcResponse {
        //     id: request.id,
        //     jsonrpc: "2.0",
        //     result: serde_json::to_value(build_targets).expect("")
        // }

        JsonRpcResponse {
            id: request.id,
            jsonrpc: "2.0",
            result: serde_json::json!({
                "targets": [
                {
                    "id": { "uri": "file:///Users/sean7218/bazel/buildserver/example/Sources/Utils:Utils" },
                    "tags": ["library"],
                    "languageIds": ["swift"],
                    "dependencies": [],
                    "capabilities": {
                        "canCompile": true,
                        "canTest": true,
                        "canRun": false,
                        "canDebug": false,
                    },
                    // "dataKind": "sourceKit",
                    // "data": {
                    //     "toolchain": "file:///Users/sean7218/Library/Developer/Toolchains/swift-6.1-RELEASE.xctoolchain/"
                    //     // "toolchain": "file:///Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain"
                    // }
                },
                {
                    "id": { "uri": "file:///Users/sean7218/bazel/buildserver/example/Sources/Components:Components" },
                    "tags": ["library"],
                    "languageIds": ["swift"],
                    "dependencies": [
                    {
                        "uri": "file:///Users/sean7218/bazel/hello-bazel/Sources/Utils/"
                    }
                    ],
                    "capabilities": {
                        "canCompile": true,
                        "canTest": true,
                        "canRun": false,
                        "canDebug": false,
                    },
                    // "dataKind": "sourceKit",
                    // "data": {
                    //     "toolchain": "file:///Users/sean7218/Library/Developer/Toolchains/swift-6.1-RELEASE.xctoolchain/"
                    //     // "toolchain": "file:///Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain"
                    // }
                }
                ]
            }),
        }
    }
}

struct Responses {}

impl Responses {
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
    fn target_sources(id: Option<serde_json::Number>) -> JsonRpcResponse {
        let response = JsonRpcResponse {
            id,
            jsonrpc: "2.0",
            result: serde_json::json!({
                "items": [
                {
                    "target": { "uri": "file:///Users/sean7218/bazel/hello-bazel/Sources/Utils/" },
                    "sources": [
                    {
                        "kind": 1,
                        "generated": false,
                        "uri": "file:///Users/sean7218/bazel/hello-bazel/Sources/Utils/AwesomeUtils.swift"
                    }
                    ]
                },
                {
                    "target": { "uri": "file:///Users/sean7218/bazel/hello-bazel/Sources/Components/" },
                    "sources": [
                    {
                        "kind": 1,
                        "generated": false,
                        "uri": "file:///Users/sean7218/bazel/hello-bazel/Sources/Components/Button.swift"
                    }
                    ]
                }
                ]
            }),
        };
        return response;
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
                uri: value.uri.to_string(),
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
// swiftc Sources/Components/Button.swift -module-name Components -I bazel-bin/Sources/Utils/ -L bazel-bin/Sources/Utils/ -l Utils -target arm64-apple-macos15.1 -sdk /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX15.1.sdk
