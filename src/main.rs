use std::{fs::OpenOptions, io::{self, BufRead, BufReader, Read, Write }};

use json_rpc::{send, JsonRpcRequest};
use serde_json::{to_string, Value};
mod json_rpc;

fn main() -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/users/sean7218/bazel/buildserver/output.txt")?;

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
                write!(file, "buildserver | exist")?;
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
                write!(file, "buildserver | error | Missing Content-Length header")?;
                continue;
            }
        };

        let mut body: Vec<u8> = vec![0; content_length];
        reader.read_exact(&mut body)?;

        let request: JsonRpcRequest = match serde_json::from_slice(&body) {
            Ok(json) => json,
            Err(e) => {
                writeln!(file, "buildserver | error | {:?}", e)?;
                continue;
            }
        };

        writeln!(file, "buildserver | request received | {:?}", request)?;
      
        if request.method == "build/initialize" {
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
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
                        "dependencySourcesProvider": false,
                        "resourcesProvider": false,
                        "outputPathsProvider": false,
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
            });
            _ = send(& response, &mut stdout);
            writeln!(file, "buildserver | response send | {}", response.to_string())?;
        } else if request.method == "build/initialized" {
            // do not send any response
        } else if request.method == "build/shutdown" {
            // do not send any response
        } else if request.method == "build/exit" {
            // do not send any response
        } else if request.method == "textDocument/registerForChanges" {
            // notification should not include "id": request.id.unwrap(),
            let response = serde_json::json!({
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
            });
            _ = send(& response, &mut stdout);
            writeln!(file, "buildserver | response send | {}", response.to_string())?;
        } else if request.method == "window/showMessage" {
            
        } else {
            write!(file, "buildserver | error | unknown request {}", request.method)?;
        }
    }
}

