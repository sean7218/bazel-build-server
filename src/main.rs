mod json_rpc;
mod logger;
use json_rpc::{JsonRpcRequest, send};
use logger::Logger;
use serde_json;
use std::{
    fs::OpenOptions,
    io::{self, BufRead, BufReader, Read},
};

fn main() -> io::Result<()> {
    let mut logger = Logger {
        file: OpenOptions::new()
            .create(true)
            .append(true)
            .open("/users/sean7218/bazel/buildserver/output.txt")?,
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
                logger.debug(&e);
                continue;
            }
        };

        logger.debug(&request);

        if request.method == "build/initialize" {
            let response = Responses::initialize(&request.id);
            send(&response, &mut stdout);
            logger.debug(&response);
        } else if request.method == "build/initialized" {
            // do not send any response
        } else if request.method == "build/shutdown" {
            // do not send any response
        } else if request.method == "build/exit" {
            // do not send any response
        } else if request.method == "textDocument/registerForChanges" {
            // notification should not include "id": request.id.unwrap(),
            let response = Responses::options();
            send(&response, &mut stdout);
            logger.debug(&response);

            // let change_response = Responses::did_change();
            // _ = send(& change_response, &mut stdout);
            // logger.info(&format!("response send -> {:?}", change_response));
        } else if request.method == "window/showMessage" {
            // do not send any response
        } else {
            let error = format!("unkown request: {:?}", request);
            logger.info(&error);
        }
    }
}

struct Responses {}
impl Responses {
    #[allow(dead_code)]
    fn options() -> serde_json::Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "build/sourceKitOptionsChanged",
            "params": {
                "uri": "file:///Users/sean7218/bazel/hello-bazel/Sources/Components/Button.swift",
                "updatedOptions": {
                    "workingDirectory": "/Users/sean7218/bazel/hello-bazel",
                    "options": [
                        "-target",
                        "arm64-apple-macos15.1",
                        "-sdk",
                        "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX15.1.sdk",
                        "-emit-object",
                        "-output-file-map",
                        "bazel-out/darwin_arm64-fastbuild/bin/Sources/Components/Components.output_file_map.json",
                        "-emit-module-path",
                        "bazel-out/darwin_arm64-fastbuild/bin/Sources/Components/Components.swiftmodule",
                        "-emit-const-values-path",
                        "bazel-out/darwin_arm64-fastbuild/bin/Sources/Components/Components_objs/Button.swift.swiftconstvalues",
                        "-module-cache-path",
                        "bazel-out/darwin_arm64-fastbuild/bin/_swift_module_cache",
                        "-Ibazel-out/darwin_arm64-fastbuild/bin/Sources/Utils",
                        "-module-name",
                        "Components",
                        "-index-store-path",
                        "/Users/sean7218/bazel/hello-bazel/bazel-out/indexstore",
                        "Sources/Components/Button.swift",
                    ]
                },
            }
        })
    }

    #[allow(dead_code)]
    fn did_change() -> serde_json::Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "method": "buildTarget/didChange",
            "params": {
                "changes": [
                    {
                        "target": {
                            "uri": "file:///Users/sean7218/bazel/hello-bazel/Sources/Components/Button.swift",
                        },
                        "kind": 2
                    }
                ]
            }
        })
    }

    #[allow(dead_code)]
    fn initialize(id: &Option<serde_json::Number>) -> serde_json::Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "displayName": "bazel-build-erver",
                "version": "1.0.0",
                "bspVersion": "2.0",
                "rootUri": "file:///Users/sean7218/bazel/hello-bazel/",
                "capabilities": {
                    "compileProvider": {
                        "languageIds": ["c", "cpp", "objective-c", "objective-cpp", "swift"]
                    },
                    "testProvider": null,
                    "runProvider": null,
                    "debugProvider": null,
                    "inverseSourcesProvider": true,
                    "dependencySourcesProvider": true,
                    "resourcesProvider": true,
                    "outputPathsProvider": true,
                    "buildTargetChangedProvider": true,
                },
                "dataKind": "sourceKit",
                "data": {
                    "indexDatabasePath": "/Users/sean7218/bazel/hello-bazel/.index-db",
                    "indexStorePath": "/Users/sean7218/hello-bazel/bazel-out/indexstore",
                    "outputPathsProvider": true,
                    "prepareProvider": true,
                    "sourceKitOptionsProvider": true
                }
            }
        })
    }
}
