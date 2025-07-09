import ActionQuery
import Foundation
import Logging
import ShellCommand
import SystemPackage

/// Handles Build Server Protocol requests
public class RequestHandler {
    public let logger: Logger
    public let config: BuildServerConfig
    public let rootPath: URL
    public let execrootPath: URL
    package var targets: [BazelTarget] = []

    private init(logger: Logger, config: BuildServerConfig, rootPath: URL, execrootPath: URL) {
        self.logger = logger
        self.config = config
        self.rootPath = rootPath
        self.execrootPath = execrootPath
    }

    /// Initialize the request handler from a build/initialize request
    public static func initialize(request: JSONRPCRequest, logger: Logger) throws -> RequestHandler {
        guard let params = request.params else {
            throw JSONRPCError.invalidRequest("Missing initialization parameters")
        }

        let buildRequest = try InitializeBuildRequest.from(jsonValue: params)
        let config = try BuildServerConfig.parse(rootUri: buildRequest.rootUri)

        guard let rootPath = URL(string: buildRequest.rootUri) else {
            throw JSONRPCError.invalidRequest("Invalid root URI: \(buildRequest.rootUri)")
        }

        // Get execution root from Bazel
        let execrootPath = try getExecutionRoot(rootPath: rootPath)

        let handler = RequestHandler(
            logger: logger,
            config: config,
            rootPath: rootPath,
            execrootPath: execrootPath
        )

        // Load targets
        try handler.loadTargets()

        // TODO: - Log loaded targets for verification

        return handler
    }

    /// Handle a BSP request and return appropriate response
    public func handleRequest(_ request: JSONRPCRequest) throws -> BuildServerResponse {
        switch request.method {
        case "build/initialized":
            return .none

        case "workspace/buildTargets":
            let response = try workspaceBuildTargets(request: request)
            return .response(response)

        case "buildTarget/sources":
            let response = try buildTargetSources(request: request)
            return .response(response)

        case "textDocument/sourceKitOptions":
            let response = try sourceKitOptions(request: request)
            return .response(response)

        case "textDocument/registerForChanges":
            let notification = try registerForChanges(request: request)
            return .notification(notification)

        case "workspace/waitForBuildSystemUpdates":
            let response = try waitForBuildSystemUpdates(request: request)
            return .response(response)

        case "workspace/didChangeWatchedFiles":
            let notification = try didChangeWatchedFiles(request: request)
            return .notification(notification)

        case "buildTarget/prepare":
            let response = try buildTargetPrepare(request: request)
            return .response(response)

        case "buildTarget/didChange":
            return .none

        case "build/shutdown":
            let response = try buildShutdown(request: request)
            return .response(response)

        case "build/exit":
            let notification = try buildExit(request: request)
            return .notification(notification)

        case "window/showMessage":
            return .none

        default:
            return .none
        }
    }

    /// Handle build/initialize request
    public func buildInitialize(request: JSONRPCRequest) throws -> JSONRPCResponse {
        let capabilities = BuildServerCapabilities(
            compileProvider: CompileProvider(languageIds: ["swift"]),
            testProvider: nil,
            runProvider: nil,
            debugProvider: nil,
            inverseSourcesProvider: true,
            dependencySourcesProvider: true,
            resourcesProvider: false,
            outputPathsProvider: false,
            buildTargetChangedProvider: true,
            jvmRunEnvironmentProvider: false,
            jvmTestEnvironmentProvider: false,
            canReload: false
        )

        let data = SourceKitInitializeBuildResponseData(
            indexDatabasePath: config.indexDatabasePath,
            indexStorePath: config.indexStorePath,
            outputPathsProvider: false,
            prepareProvider: true,
            sourceKitOptionsProvider: true,
            defaultSettings: config.defaultSettings ?? []
        )

        let response = InitializeBuildResponse(
            displayName: "Bazel SourceKit BSP",
            version: "1.0.0",
            bspVersion: "2.0.0",
            capabilities: capabilities,
            data: data
        )

        return try JSONRPCResponse(
            id: request.id,
            result: response.toJSONValue()
        )
    }

    // MARK: - BSP Method Implementations

    private func workspaceBuildTargets(request: JSONRPCRequest) throws -> JSONRPCResponse {
        let buildTargets = targets.map { BuildTarget.from(bazelTarget: $0) }
        let response = WorkspaceBuildTargetsResponse(targets: buildTargets)

        return try JSONRPCResponse(
            id: request.id,
            result: response.toJSONValue()
        )
    }

    private func buildTargetSources(request: JSONRPCRequest) throws -> JSONRPCResponse {
        guard let params = request.params else {
            throw JSONRPCError.invalidRequest("Missing parameters")
        }

        let buildTargetSourcesRequest = try BuildTargetSourcesRequest.from(jsonValue: params)
        var items: [SourcesItem] = []

        for target in buildTargetSourcesRequest.targets {
            if let bazelTarget = targets.first(where: { $0.uri == target.uri }) {
                let sources = try getSourcesForTarget(bazelTarget)
                let item = SourcesItem(
                    target: target,
                    sources: sources,
                    roots: [rootPath.absoluteString]
                )
                items.append(item)
            }
        }

        let response = BuildTargetSourcesResponse(items: items)
        return try JSONRPCResponse(
            id: request.id,
            result: response.toJSONValue()
        )
    }

    private func sourceKitOptions(request: JSONRPCRequest) throws -> JSONRPCResponse {
        guard let params = request.params else {
            throw JSONRPCError.invalidRequest("Missing parameters")
        }

        let sourceKitRequest = try TextDocumentSourceKitOptionsRequest.from(jsonValue: params)
        let options = try getSourceKitOptions(for: sourceKitRequest.textDocument.uri, target: sourceKitRequest.target)

        let response = TextDocumentSourceKitOptionsResponse(
            compilerArguments: options,
            workingDirectory: rootPath.path
        )

        return try JSONRPCResponse(
            id: request.id,
            result: response.toJSONValue()
        )
    }

    private func registerForChanges(request: JSONRPCRequest) throws -> JSONRPCNotification {
        guard let params = request.params else {
            throw JSONRPCError.invalidRequest("Missing parameters")
        }

        let registerRequest = try RegisterForChanges.from(jsonValue: params)

        // If targets haven't been loaded yet, load them now
        if targets.isEmpty {
            try loadTargets()
        }

        // Find compiler arguments for the specific file
        var options: [String] = []
        for target in targets {
            for inputFile in target.inputFiles {
                if inputFile == registerRequest.uri {
                    options = target.compilerArguments
                    break
                }
            }
            if !options.isEmpty {
                break
            }
        }

        // If no specific options found, use default settings
        if options.isEmpty {
            options = config.defaultSettings ?? []
        }

        let notification = FileOptionsChangedNotification(
            uri: registerRequest.uri,
            updatedOptions: Options(
                options: options,
                workingDirectory: rootPath.path
            )
        )

        return try JSONRPCNotification(
            method: "textDocument/sourceKitOptionsChanged",
            params: notification.toJSONValue()
        )
    }

    private func waitForBuildSystemUpdates(request: JSONRPCRequest) throws -> JSONRPCResponse {
        // For now, just return immediately
        return JSONRPCResponse(
            id: request.id,
            result: .null
        )
    }

    private func didChangeWatchedFiles(request _: JSONRPCRequest) throws -> JSONRPCNotification {
        // For now, just acknowledge
        return JSONRPCNotification(
            method: "buildTarget/didChange",
            params: .null
        )
    }

    private func buildTargetPrepare(request: JSONRPCRequest) throws -> JSONRPCResponse {
        // Build the target using Bazel on a background thread
        var commandArgs = ["build", config.target]
        commandArgs.append(contentsOf: config.aqueryArgs)

        let rootPath = self.rootPath
        let logger = self.logger

        Task {
            let result = ShellCommand(
                executable: "bazel",
                currentDir: rootPath.path(),
                args: commandArgs
            ).run()

            if result.exitCode == 0 {
                logger.info("Build completed successfully")
            } else {
                logger.error("Build failed with exit code \(result.exitCode)")
                if let output = result.output {
                    logger.error("Build output: \(output)")
                }
            }
        }

        // Return immediately without waiting for the build to complete
        return JSONRPCResponse(
            id: request.id,
            result: .null
        )
    }

    private func buildShutdown(request: JSONRPCRequest) throws -> JSONRPCResponse {
        return JSONRPCResponse(
            id: request.id,
            result: .null
        )
    }

    private func buildExit(request _: JSONRPCRequest) throws -> JSONRPCNotification {
        return JSONRPCNotification(
            method: "build/exit",
            params: .null
        )
    }

    // MARK: - Helper Methods

    private static func getExecutionRoot(rootPath: URL) throws -> URL {
        guard let output = ShellCommand(
            executable: "bazel",
            currentDir: rootPath.path(),
            args: ["info", "execution_root"]
        ).run().output else {
            fatalError("Failed to get execution_root")
        }

        let execrootPath = output.trimmingCharacters(in: .whitespacesAndNewlines)
        return URL(fileURLWithPath: execrootPath)
    }

    private func loadTargets() throws {
        targets = try ActionQuery().execute(
            target: config.target,
            rootPath: rootPath,
            execrootPath: execrootPath,
            sdk: config.sdk,
            aqueryArgs: config.aqueryArgs,
            logger: logger
        )
    }

    private func getSourcesForTarget(_ target: BazelTarget) throws -> [SourceItem] {
        // Convert input files to SourceItem objects
        return target.inputFiles.map { filePath in
            SourceItem(
                uri: filePath,
                kind: .file,
                generated: false
            )
        }
    }

    private func getSourceKitOptions(for _: String, target: BuildTargetIdentifier) throws -> [String] {
        // Find the corresponding BazelTarget
        if let bazelTarget = targets.first(where: { $0.uri == target.uri }) {
            return bazelTarget.compilerArguments
        }

        // Return the default settings from config if target not found
        return config.defaultSettings ?? []
    }
}

// MARK: - BuildServerResponse enum

public enum BuildServerResponse {
    case response(JSONRPCResponse)
    case notification(JSONRPCNotification)
    case none
    case exit
}
