#![allow(dead_code)]
mod aquery;
mod json_rpc;
mod logger;
mod messages;

use json_rpc::{send, send_response, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};
use logger::Logger;
use messages::initialize_build_request::{BuildServerCapabilities, CompileProvider, InitializeBuildRequest, InitializeBuildResponse, SourceKitInitializeBuildResponseData};
use serde_json::{self, from_value, to_value, Value};
use std::{
    fs::OpenOptions,
    io::{self, BufRead, BufReader, Read},
};

fn main() -> io::Result<()> {
    let home_dir = dirs::home_dir().expect("Failed to get home directory.");
    let config_dir = home_dir.join(".config");
    let server_dir = config_dir.join("sourcekit-bsp");
    let log_file = server_dir.join("bsp.log");

    if !server_dir.exists() {
        std::fs::create_dir_all(&server_dir)?;
    }

    let mut logger = Logger {
        file: OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)?,
    };

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut reader = BufReader::new(stdin.lock());

    loop {
        let mut content_length = None;
        let mut buffer = String::new();
        loop {
            buffer.clear();
            let bytes = reader.read_line(&mut buffer)?;
            if bytes == 0 {
                logger.info("eof -> exist");
                return Ok(()); // EOF
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
            None => {
                logger.info("Missing Content-Length header");
                continue;
            }
        };

        let mut body: Vec<u8> = vec![0; content_length];
        reader.read_exact(&mut body)?;

        let request: JsonRpcRequest = match serde_json::from_slice(&body) {
            Ok(json) => json,
            Err(e) => {
                logger.info("Failed to parse the request");
                logger.debug(&e);
                continue;
            }
        };

        logger.debug(&request);

        if request.method == "build/initialize" {
            let response = Responses::initialize_build_response(request);
            let value = to_value(&response)?;
            send(&value, &mut stdout);
            logger.debug(&response);
        } else if request.method == "build/initialized" {
            // do not send any response
        } else if request.method == "build/shutdown" {
            // do not send any response
        } else if request.method == "build/exit" {
            // do not send any response
            return Ok(())
        } else if request.method == "workspace/buildTargets" {
            let response = Responses::build_targets(request.id);
            send_response(&response, &mut stdout);
            logger.debug(&response);
        } else if request.method == "buildTarget/sources" {
            let response = Responses::target_sources(request.id);
            send_response(&response, &mut stdout);
            logger.debug(&response);
        } else if request.method == "textDocument/sourceKitOptions" {
            let response = Responses::sourcekit_options(request);
            send_response(&response, &mut stdout);
            logger.debug(&response);
        } else if request.method == "buildTarget/didChange" {
            // TODO: buildTarget/didChange
        } else if request.method == "workspace/waitForBuildSystemUpdates" {
            // TODO: waitForBuildSystemUpdates
        } else if request.method == "buildTarget/prepare" {
            // TODO: buildTarget/prepare
        } else if request.method == "textDocument/registerForChanges" {
            // INFO: notification should not include "id": request.id.unwrap(),
            // this endpoint is for push model (legacy)
            let response = Responses::options_changed();
            let value = to_value(&response)?;
            send(&value, &mut stdout);
            logger.debug(&response);
        } else if request.method == "window/showMessage" {
            // TODO: send to editor notification
        } else {
            let error = format!("unkown request: {:?}", request);
            logger.info(&error);
        }
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
                    "arm64-apple-macos15.1",
                    "-sdk",
                    "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX15.1.sdk",
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
    fn initialize_build_response(request: JsonRpcRequest) -> JsonRpcResponse {
        let result = InitializeBuildResponse::new(
            "bazel-build-server",
            "1.0.0",
            "2.0",
            BuildServerCapabilities {
                compile_provider: Some(CompileProvider {
                    language_ids: vec!["c", "cpp", "objective-c", "objective-cpp", "swift"]
                }),
                test_provider: None,
                run_provider: None,
                debug_provider: None,
                inverse_sources_provider: Some(true),
                dependency_sources_provider: Some(true),
                resources_provider: Some(true),
                output_paths_provider: Some(true),
                build_target_changed_provider: Some(true),
                can_reload: Some(false)
            },
            "sourceKit",
            SourceKitInitializeBuildResponseData {
                index_database_path: Some(String::from("/Users/sean7218/bazel/hello-bazel/.index-db")),
                index_store_path: Some(String::from("/Users/sean7218/hello-bazel/bazel-out/indexstore")),
                output_paths_provider: Some(true),
                prepare_provider: Some(true),
                source_kit_options_provider: Some(true),
            }
        );
        let value = to_value(result)
            .expect("Failed to serialize InitializeBuildResponse.");
        JsonRpcResponse::new(request.id, value)
    }

    #[allow(dead_code)]
    fn build_targets(id: Option<serde_json::Number>) -> JsonRpcResponse {
        JsonRpcResponse {
            id: id,
            jsonrpc: "2.0",
            result: serde_json::json!({
                "targets": [
                {
                    "id": { "uri": "file:///Users/sean7218/bazel/hello-bazel/Sources/Utils/" },
                    "tags": ["library"],
                    "languageIds": ["swift"],
                    "dependencies": [],
                    "capabilities": {
                        "canCompile": true,
                        "canTest": true,
                        "canRun": false,
                        "canDebug": false,
                    }
                },
                {
                    "id": { "uri": "file:///Users/sean7218/bazel/hello-bazel/Sources/Components/" },
                    "tags": ["library"],
                    "languageIds": ["swift"],
                    "dependencies": [
                    { "uri": "file:///Users/sean7218/bazel/hello-bazel/Sources/Utils/" }
                    ],
                    "capabilities": {
                        "canCompile": true,
                        "canTest": true,
                        "canRun": false,
                        "canDebug": false,
                    }
                }
                ]
            })
        }
    }

    #[allow(dead_code)]
    fn target_sources(id: Option<serde_json::Number>) -> JsonRpcResponse {
        let response = JsonRpcResponse {
            id: id,
            jsonrpc: "2.0",
            result: serde_json::json!({
                "items": [
                {
                    "target": { "uri": "file:///Users/sean7218/bazel/hello-bazel/Sources/Utils/" },
                    "sources": [
                    { "kind": 1, "generated": false, "uri": "file:///Users/sean7218/bazel/hello-bazel/Sources/Utils/AwesomeUtils.swift" }
                    ]
                },
                {
                    "target": { "uri": "file:///Users/sean7218/bazel/hello-bazel/Sources/Components/" },
                    "sources": [
                    { "kind": 1, "generated": false, "uri": "file:///Users/sean7218/bazel/hello-bazel/Sources/Components/Button.swift" }
                    ]
                }
                ]
            })
        };
        return response
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
                        "-swift-version",
                        "6.1.0",
                        "-target",
                        "arm64-apple-macos15.1",
                        "-sdk",
                        "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX15.1.sdk",
                        "-module-name",
                        "Utils",
                        "-index-store-path",
                        "/Users/sean7218/bazel/hello-bazel/bazel-out/indexstore",
                        "/Users/sean7218/bazel/hello-bazel/Sources/Utils/AwesomeUtils.swift",
                    ]
                })
            };
        } else if uri.contains("Button.swift") {
            return JsonRpcResponse {
                id: request.id,
                jsonrpc: "2.0",
                result: serde_json::json!({
                    "compilerArguments": [
                        "-module-name",
                        "Components",
                        "-swift-version",
                        "6.1.0",
                        "-target",
                        "arm64-apple-macos15.1",
                        "-sdk",
                        "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX15.1.sdk",
                        "-I/Users/sean7218/bazel/hello-bazel/bazel-bin/Sources/Utils",
                        "/Users/sean7218/bazel/hello-bazel/bazel-out/indexstore",
                        "/Users/sean7218/bazel/hello-bazel/Sources/Components/Button.swift",
                        ]
                })
            };
        } else {
            return JsonRpcResponse {
                id: request.id,
                jsonrpc: "2.0",
                result: serde_json::json!({
                    "compilerArguments": []
                })
            };
        }
    }
}

                        // swiftc Sources/Components/Button.swift -module-name Components -I bazel-bin/Sources/Utils/ -L bazel-bin/Sources/Utils/ -l Utils -target arm64-apple-macos15.1 -sdk /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX15.1.sdk
