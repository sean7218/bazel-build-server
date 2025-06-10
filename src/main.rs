mod aquery;
mod error;
mod json_rpc;
mod messages;
mod support_types;
mod utils;

use crate::error::Result;
use aquery::BazelTarget;
use build_server_config::BuildServerConfig;
use json_rpc::{
    JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, read_request, send, send_notification,
    send_response,
};
use messages::{
    build_target_sources_request::{
        BuildTargetSourcesRequest, BuildTargetSourcesResponse, SourceItem, SourcesItem,
    },
    initialize_build_request::{
        BuildServerCapabilities, CompileProvider, InitializeBuildRequest, InitializeBuildResponse,
        SourceKitInitializeBuildResponseData,
    },
    register_for_changes::{FileOptionsChangedNotification, Options, RegisterForChanges},
    text_document_source_kit_options_request::{
        TextDocumentSourceKitOptionsRequest, TextDocumentSourceKitOptionsResponse,
    },
};
use serde_json::{self, from_value, to_value};
use std::{
    io::{self, BufReader, StdoutLock},
    panic,
    path::PathBuf,
    process::Command,
};
use support_types::{
    build_server_config,
    build_target::{BuildTarget, BuildTargetCapabilities, BuildTargetIdentifier},
    methods::RequestMethod,
};

fn main() {
    panic::set_hook(Box::new(|panic_info| {
        if let Some(location) = panic_info.location() {
            log_str!("bsp_server crashed: {:#?}", location);
        } else {
            log_str!("bsp_server crashed: unknown location");
        }
    }));

    // capture any error here
    if let Err(error) = run() {
        log_str!("bsp_server error: {:#?}", error);
    }
}

fn run() -> Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    let mut reader = BufReader::new(stdin.lock());

    let mut request_handler = match handle_initialize_request(&mut reader, &mut stdout) {
        Ok(v) => v,
        Err(e) => {
            log_str!("👻 handle_initialize_request failed -> {:#?}", &e);
            return Ok(());
        }
    };

    log_str!("🟢 Build Server Initialized");

    loop {
        let request = match read_request(&mut reader) {
            Ok(v) => v,
            Err(e) => {
                log_str!("👻 read_request(&mut reader) failed -> {:#?}", &e);
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
                let notification = request_handler.legacy_register_for_changes(request)?;
                send_notification(&notification, &mut stdout);
                log_str!("↩️ {:#?}", notification);
            }
            RequestMethod::WorkspaceWaitForBuildSystemUpdates => {
                let response = request_handler.wait_for_updates(request)?;
                send_response(&response, &mut stdout);
                log_str!("↩️ {:#?}", response);
            }
            RequestMethod::BuildTargetPrepare => {
                let response = request_handler.build_target_repare(request)?;
                send_response(&response, &mut stdout);
                log_str!("↩️ {:#?}", response);
            }
            RequestMethod::BuildTargetDidChange => {}
            RequestMethod::BuildShutDown => return Ok(()),
            RequestMethod::BuildExit => return Ok(()),
            RequestMethod::WindowShowMessage => {}
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
    let request = read_request(reader)?;
    let request_handler = RequestHandler::initialize(&request)?;

    let response = request_handler.build_initialize(&request)?;
    log_str!("↩️ {:#?}", response);

    let value = to_value(&response)?;
    send(&value, stdout);

    Ok(request_handler)
}

struct RequestHandler {
    config: BuildServerConfig,
    root_path: PathBuf,
    targets: Vec<BazelTarget>,
}

impl RequestHandler {
    fn root_path_string(&self) -> String {
        self.root_path.to_string_lossy().into_owned()
    }

    fn initialize(request: &JsonRpcRequest) -> Result<Self> {
        let build_request: InitializeBuildRequest = from_value(request.params.clone())?;

        let config = BuildServerConfig::parse(&build_request.root_uri)?;

        let root_path = build_request
            .root_uri
            .to_file_path()
            .map_err(|_| "Failed to convert root_uri to file path")?;

        Ok(RequestHandler {
            config,
            root_path,
            targets: vec![],
        })
    }

    fn build_initialize(&self, request: &JsonRpcRequest) -> Result<JsonRpcResponse> {
        let index_database_path = self.config.index_database_path.clone();
        let index_store_path = self.config.index_store_path.clone();

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
                inverse_sources_provider: Some(false),
                dependency_sources_provider: Some(false),
                resources_provider: Some(false),
                // bazel doesn't support `-index-unit-output-path
                output_paths_provider: Some(false), 
                build_target_changed_provider: Some(true),
                can_reload: Some(false),
            },
            "sourceKit",
            SourceKitInitializeBuildResponseData {
                index_database_path: Some(index_database_path),
                index_store_path: Some(index_store_path),
                output_paths_provider: Some(false),
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
        let targets = aquery::aquery(
            &target,
            &dir,
            &self.config.sdk,
            self.config.aquery_args.clone(),
            self.config.bazel_out.clone()
        )?;

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
            }),
        };

        Ok(response)
    }

    fn build_target_sources(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let source_request: BuildTargetSourcesRequest = serde_json::from_value(request.params)?;
        let mut items: Vec<SourcesItem> = vec![];

        for target in source_request.targets {
            let bazel_target = match self.targets.iter().find(|t| t.uri.eq(&target.uri)) {
                Some(v) => v,
                None => {
                    continue;
                }
            };

            let sources: Vec<SourceItem> = bazel_target
                .input_files
                .iter()
                .map(SourceItem::from_url)
                .collect();

            items.push(SourcesItem {
                target,
                sources,
                roots: None,
            });
        }

        let result = BuildTargetSourcesResponse { items };
        let resp = JsonRpcResponse {
            id: request.id,
            jsonrpc: "2.0",
            result: serde_json::to_value(result)?,
        };
        Ok(resp)
    }

    fn wait_for_updates(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        Ok(JsonRpcResponse::new(request.id, serde_json::Value::Null))
    }

    fn build_target_repare(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let directory = self.root_path.clone();
        let target = self.config.target.clone();
        let output = Command::new("bazel")
            .args(&["build", &target])
            .current_dir(directory)
            .output()?;

        log_str!("🟢 target/prepare {:#?}", output);
        Ok(JsonRpcResponse::new(request.id, serde_json::Value::Null))
    }

    fn sourcekit_options(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let req: TextDocumentSourceKitOptionsRequest = from_value(request.params)?;
        let target = match self.targets.iter().find(|it| it.uri.eq(&req.target.uri)) {
            Some(v) => v,
            None => return Err("Failed to find target for sourcekit_options".into()),
        };

        log_str!("✨ Found target for sourcekit_options.");

        let result = TextDocumentSourceKitOptionsResponse {
            compiler_arguments: target.compiler_arguments.clone(),
            working_directory: Some(self.root_path.clone()),
            data: None,
        };

        let response = JsonRpcResponse {
            id: request.id,
            jsonrpc: "2.0",
            result: to_value(result)?,
        };
        Ok(response)
    }

    // INFO: this endpoint is for legacy push-based model
    fn legacy_register_for_changes(
        &mut self,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcNotification> {
        // if bazel targets is empty, we know it is the initial request
        if self.targets.is_empty() {
            let targets = aquery::aquery(
                &self.config.target,
                &self.root_path,
                &self.config.sdk,
                self.config.aquery_args.clone(),
                self.config.bazel_out.clone()
            )?;

            self.targets = targets;
        }

        let req: RegisterForChanges = from_value(request.params)?;

        let mut options: Vec<String> = vec![];
        for target in &self.targets {
            for file in &target.input_files {
                if file.eq(&req.uri) {
                    for arg in &target.compiler_arguments {
                        options.push(arg.clone());
                    }
                }
            }
        }

        let params = FileOptionsChangedNotification {
            uri: req.uri,
            updated_options: Options {
                options,
                working_directory: Some(self.root_path_string()),
            },
        };

        let response = JsonRpcNotification::new(
            String::from(FileOptionsChangedNotification::METHOD),
            to_value(params)?,
        );

        Ok(response)
    }
}

impl From<BazelTarget> for BuildTarget {
    fn from(value: BazelTarget) -> Self {
        BuildTarget {
            id: BuildTargetIdentifier { uri: value.uri },
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
