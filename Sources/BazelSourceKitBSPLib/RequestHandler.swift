import Foundation
import Logging
import ShellOut
import SystemPackage

/// Handles Build Server Protocol requests
public class RequestHandler {
    public let logger: Logger
    public let config: BuildServerConfig
    public let rootPath: URL
    public let execrootPath: URL
    public var targets: [BazelTarget] = []

    private init(logger: Logger, config: BuildServerConfig, rootPath: URL, execrootPath: URL) {
        self.logger = logger
        self.config = config
        self.rootPath = rootPath
        self.execrootPath = execrootPath
    }

    /// Initialize the request handler from a build/initialize request
    public static func initialize(request: JSONRPCRequest, logger: Logger) throws -> RequestHandler {
        logger.debug("🔧 Initializing RequestHandler...")

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

        logger.info("✅ RequestHandler initialized with \(handler.targets.count) targets")
        return handler
    }

    /// Handle a BSP request and return appropriate response
    public func handleRequest(_ request: JSONRPCRequest) throws -> BuildServerResponse {
        logger.debug("🔍 Handling request: \(request.method)")

        switch request.method {
        case "build/initialized":
            logger.info("🤩 Build server initialized!")
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
            logger.warning("🤷 Unknown request method: \(request.method)")
            return .none
        }
    }

    /// Handle build/initialize request
    public func buildInitialize(request: JSONRPCRequest) throws -> JSONRPCResponse {
        logger.debug("🏗️ Building initialize response...")

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
        logger.debug("📋 Getting workspace build targets...")

        let buildTargets = targets.map { BuildTarget.from(bazelTarget: $0) }
        let response = WorkspaceBuildTargetsResponse(targets: buildTargets)

        return try JSONRPCResponse(
            id: request.id,
            result: response.toJSONValue()
        )
    }

    private func buildTargetSources(request: JSONRPCRequest) throws -> JSONRPCResponse {
        logger.debug("📁 Getting build target sources...")

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
        logger.debug("⚙️ Getting SourceKit options...")

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
        logger.debug("🔔 Registering for changes...")

        guard let params = request.params else {
            throw JSONRPCError.invalidRequest("Missing parameters")
        }

        let registerRequest = try RegisterForChanges.from(jsonValue: params)

        let notification = FileOptionsChangedNotification(
            uri: registerRequest.uri,
            updatedOptions: Options(
                options: config.defaultSettings ?? [],
                workingDirectory: rootPath.path
            )
        )

        return try JSONRPCNotification(
            method: "textDocument/sourceKitOptionsChanged",
            params: notification.toJSONValue()
        )
    }

    private func waitForBuildSystemUpdates(request: JSONRPCRequest) throws -> JSONRPCResponse {
        logger.debug("⏳ Waiting for build system updates...")

        // For now, just return immediately
        return JSONRPCResponse(
            id: request.id,
            result: .null
        )
    }

    private func didChangeWatchedFiles(request _: JSONRPCRequest) throws -> JSONRPCNotification {
        logger.debug("📝 Files changed notification...")

        // For now, just acknowledge
        return JSONRPCNotification(
            method: "buildTarget/didChange",
            params: .null
        )
    }

    private func buildTargetPrepare(request: JSONRPCRequest) throws -> JSONRPCResponse {
        logger.debug("🎯 Preparing build target...")

        // Build the target using Bazel
        var commandArgs = ["build", config.target]
        commandArgs.append(contentsOf: config.aqueryArgs)

        logger.info("🔨 Running bazel build: bazel \(commandArgs.joined(separator: " "))")

        do {
            let output = try shellOut(
                to: "bazel",
                arguments: commandArgs,
                at: rootPath.path
            )
            logger.debug("✅ Build output: \(output)")
        } catch {
            logger.warning("⚠️ Build failed but continuing: \(error)")
            // Don't fail the BSP request even if build fails
        }

        return JSONRPCResponse(
            id: request.id,
            result: .null
        )
    }

    private func buildShutdown(request: JSONRPCRequest) throws -> JSONRPCResponse {
        logger.info("🔌 Shutting down...")

        return JSONRPCResponse(
            id: request.id,
            result: .null
        )
    }

    private func buildExit(request _: JSONRPCRequest) throws -> JSONRPCNotification {
        logger.info("👋 Exiting...")

        return JSONRPCNotification(
            method: "build/exit",
            params: .null
        )
    }

    // MARK: - Helper Methods

    private static func getExecutionRoot(rootPath: URL) throws -> URL {
        let output = try shellOut(
            to: "bazel",
            arguments: ["info", "execution_root"],
            at: rootPath.path
        )

        let execrootPath = output.trimmingCharacters(in: .whitespacesAndNewlines)
        return URL(fileURLWithPath: execrootPath)
    }

    private func loadTargets() throws {
        logger.debug("🎯 Loading Bazel targets using aquery...")

        targets = try executeAquery(
            target: config.target,
            rootPath: rootPath,
            execrootPath: execrootPath,
            sdk: config.sdk,
            aqueryArgs: config.aqueryArgs,
            extraIncludes: config.extraIncludes ?? [],
            extraFrameworks: config.extraFrameworks ?? [],
            logger: logger
        )

        logger.info("📦 Loaded \(targets.count) targets")
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
