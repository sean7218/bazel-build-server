import Foundation

// MARK: - BSP Protocol Types

/// Build/Initialize Request
public struct InitializeBuildRequest: Codable {
    public let displayName: String
    public let version: String
    public let bspVersion: String
    public let rootUri: String
    public let capabilities: BuildClientCapabilities
    public let data: InitializeBuildRequestData?

    public static func from(jsonValue: JSONValue) throws -> InitializeBuildRequest {
        let data = try jsonValue.toData()
        return try JSONDecoder().decode(InitializeBuildRequest.self, from: data)
    }
}

public struct BuildClientCapabilities: Codable {
    public let languageIds: [String]

    public init(languageIds: [String]) {
        self.languageIds = languageIds
    }
}

public struct InitializeBuildRequestData: Codable {
    // Additional data for initialization
}

/// Build/Initialize Response
public struct InitializeBuildResponse: Codable {
    public let displayName: String
    public let version: String
    public let bspVersion: String
    public let capabilities: BuildServerCapabilities
    public let data: SourceKitInitializeBuildResponseData?

    public func toJSONValue() throws -> JSONValue {
        let data = try JSONEncoder().encode(self)
        return try JSONValue.from(data: data)
    }
}

public struct BuildServerCapabilities: Codable {
    public let compileProvider: CompileProvider?
    public let testProvider: TestProvider?
    public let runProvider: RunProvider?
    public let debugProvider: DebugProvider?
    public let inverseSourcesProvider: Bool?
    public let dependencySourcesProvider: Bool?
    public let resourcesProvider: Bool?
    public let outputPathsProvider: Bool?
    public let buildTargetChangedProvider: Bool?
    public let jvmRunEnvironmentProvider: Bool?
    public let jvmTestEnvironmentProvider: Bool?
    public let canReload: Bool?

    public init(
        compileProvider: CompileProvider? = nil,
        testProvider: TestProvider? = nil,
        runProvider: RunProvider? = nil,
        debugProvider: DebugProvider? = nil,
        inverseSourcesProvider: Bool? = nil,
        dependencySourcesProvider: Bool? = nil,
        resourcesProvider: Bool? = nil,
        outputPathsProvider: Bool? = nil,
        buildTargetChangedProvider: Bool? = nil,
        jvmRunEnvironmentProvider: Bool? = nil,
        jvmTestEnvironmentProvider: Bool? = nil,
        canReload: Bool? = nil
    ) {
        self.compileProvider = compileProvider
        self.testProvider = testProvider
        self.runProvider = runProvider
        self.debugProvider = debugProvider
        self.inverseSourcesProvider = inverseSourcesProvider
        self.dependencySourcesProvider = dependencySourcesProvider
        self.resourcesProvider = resourcesProvider
        self.outputPathsProvider = outputPathsProvider
        self.buildTargetChangedProvider = buildTargetChangedProvider
        self.jvmRunEnvironmentProvider = jvmRunEnvironmentProvider
        self.jvmTestEnvironmentProvider = jvmTestEnvironmentProvider
        self.canReload = canReload
    }
}

public struct CompileProvider: Codable {
    public let languageIds: [String]

    public init(languageIds: [String]) {
        self.languageIds = languageIds
    }
}

public struct TestProvider: Codable {
    public let languageIds: [String]

    public init(languageIds: [String]) {
        self.languageIds = languageIds
    }
}

public struct RunProvider: Codable {
    public let languageIds: [String]

    public init(languageIds: [String]) {
        self.languageIds = languageIds
    }
}

public struct DebugProvider: Codable {
    public let languageIds: [String]

    public init(languageIds: [String]) {
        self.languageIds = languageIds
    }
}

public struct SourceKitInitializeBuildResponseData: Codable {
    public let defaultSettings: [String]

    public init(defaultSettings: [String]) {
        self.defaultSettings = defaultSettings
    }
}

// MARK: - Build Target Types

public struct BuildTarget: Codable {
    public let id: BuildTargetIdentifier
    public let displayName: String?
    public let baseDirectory: String?
    public let tags: [String]
    public let capabilities: BuildTargetCapabilities
    public let languageIds: [String]
    public let dependencies: [BuildTargetIdentifier]
    public let dataKind: String?
    public let data: BuildTargetData?

    public static func from(bazelTarget: BazelTarget) -> BuildTarget {
        return BuildTarget(
            id: BuildTargetIdentifier(uri: bazelTarget.uri),
            displayName: bazelTarget.label,
            baseDirectory: nil,
            tags: bazelTarget.tags,
            capabilities: BuildTargetCapabilities(
                canCompile: true,
                canTest: bazelTarget.kind.contains("test"),
                canRun: bazelTarget.kind.contains("binary"),
                canDebug: false
            ),
            languageIds: ["swift"],
            dependencies: [],
            dataKind: bazelTarget.kind,
            data: nil
        )
    }
}

public struct BuildTargetIdentifier: Codable {
    public let uri: String

    public init(uri: String) {
        self.uri = uri
    }
}

public struct BuildTargetCapabilities: Codable {
    public let canCompile: Bool
    public let canTest: Bool
    public let canRun: Bool
    public let canDebug: Bool

    public init(canCompile: Bool, canTest: Bool, canRun: Bool, canDebug: Bool) {
        self.canCompile = canCompile
        self.canTest = canTest
        self.canRun = canRun
        self.canDebug = canDebug
    }
}

public struct BuildTargetData: Codable {
    // Additional target-specific data
}

// MARK: - Workspace Build Targets

public struct WorkspaceBuildTargetsResponse: Codable {
    public let targets: [BuildTarget]

    public init(targets: [BuildTarget]) {
        self.targets = targets
    }

    public func toJSONValue() throws -> JSONValue {
        let data = try JSONEncoder().encode(self)
        return try JSONValue.from(data: data)
    }
}

// MARK: - Build Target Sources

public struct BuildTargetSourcesRequest: Codable {
    public let targets: [BuildTargetIdentifier]

    public static func from(jsonValue: JSONValue) throws -> BuildTargetSourcesRequest {
        let data = try jsonValue.toData()
        return try JSONDecoder().decode(BuildTargetSourcesRequest.self, from: data)
    }
}

public struct BuildTargetSourcesResponse: Codable {
    public let items: [SourcesItem]

    public init(items: [SourcesItem]) {
        self.items = items
    }

    public func toJSONValue() throws -> JSONValue {
        let data = try JSONEncoder().encode(self)
        return try JSONValue.from(data: data)
    }
}

public struct SourcesItem: Codable {
    public let target: BuildTargetIdentifier
    public let sources: [SourceItem]
    public let roots: [String]?

    public init(target: BuildTargetIdentifier, sources: [SourceItem], roots: [String]?) {
        self.target = target
        self.sources = sources
        self.roots = roots
    }
}

public struct SourceItem: Codable {
    public let uri: String
    public let kind: Int
    public let generated: Bool
    public let dataKind: String
    public let data: SourceKitSourceItemData

    public init(uri: String, kind: SourceItemKind, generated: Bool = false) {
        self.uri = uri
        self.kind = kind.rawValue
        self.generated = generated
        dataKind = "sourceKit"
        data = SourceKitSourceItemData(
            language: nil,
            kind: .source,
            outputPath: nil
        )
    }
}

public enum SourceItemKind: Int, Codable {
    case file = 1
    case directory = 2
}

public struct SourceKitSourceItemData: Codable {
    public let language: String?
    public let kind: SourceKitSourceItemKind?
    public let outputPath: String?

    public init(language: String? = nil, kind: SourceKitSourceItemKind? = nil, outputPath: String? = nil) {
        self.language = language
        self.kind = kind
        self.outputPath = outputPath
    }
}

public enum SourceKitSourceItemKind: String, Codable {
    case source
    case header
    case doccCatalog
}

// MARK: - SourceKit Options

public struct TextDocumentSourceKitOptionsRequest: Codable {
    public let textDocument: TextDocumentIdentifier
    public let target: BuildTargetIdentifier
    public let language: String

    public static func from(jsonValue: JSONValue) throws -> TextDocumentSourceKitOptionsRequest {
        let data = try jsonValue.toData()
        return try JSONDecoder().decode(TextDocumentSourceKitOptionsRequest.self, from: data)
    }
}

public struct TextDocumentIdentifier: Codable {
    public let uri: String

    public init(uri: String) {
        self.uri = uri
    }
}

public struct TextDocumentSourceKitOptionsResponse: Codable {
    public let compilerArguments: [String]
    public let workingDirectory: String?
    public let data: JSONValue?

    public init(compilerArguments: [String], workingDirectory: String? = nil, data: JSONValue? = nil) {
        self.compilerArguments = compilerArguments
        self.workingDirectory = workingDirectory
        self.data = data
    }

    public func toJSONValue() throws -> JSONValue {
        let data = try JSONEncoder().encode(self)
        return try JSONValue.from(data: data)
    }
}

// MARK: - Register for Changes

public struct RegisterForChanges: Codable {
    public let uri: String
    public let action: String

    public static func from(jsonValue: JSONValue) throws -> RegisterForChanges {
        let data = try jsonValue.toData()
        return try JSONDecoder().decode(RegisterForChanges.self, from: data)
    }
}

public struct FileOptionsChangedNotification: Codable {
    public let uri: String
    public let updatedOptions: Options

    public init(uri: String, updatedOptions: Options) {
        self.uri = uri
        self.updatedOptions = updatedOptions
    }

    public func toJSONValue() throws -> JSONValue {
        let data = try JSONEncoder().encode(self)
        return try JSONValue.from(data: data)
    }
}

public struct Options: Codable {
    public let options: [String]
    public let workingDirectory: String?

    public init(options: [String], workingDirectory: String? = nil) {
        self.options = options
        self.workingDirectory = workingDirectory
    }
}

// MARK: - Bazel Target Types

public struct BazelTarget: Codable, Hashable {
    public let id: UInt32
    public let uri: String
    public let label: String
    public let kind: String
    public let tags: [String]
    public let inputFiles: [String]
    public let compilerArguments: [String]

    public init(
        id: UInt32,
        uri: String,
        label: String,
        kind: String,
        tags: [String],
        inputFiles: [String],
        compilerArguments: [String]
    ) {
        self.id = id
        self.uri = uri
        self.label = label
        self.kind = kind
        self.tags = tags
        self.inputFiles = inputFiles
        self.compilerArguments = compilerArguments
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(id)
        hasher.combine(label)
    }

    public static func == (lhs: BazelTarget, rhs: BazelTarget) -> Bool {
        return lhs.id == rhs.id && lhs.label == rhs.label
    }
}

// MARK: - Build Server Config

public struct BuildServerConfig: Codable {
    public let name: String
    public let argv: [String]
    public let version: String
    public let bspVersion: String
    public let languages: [String]
    public let target: String
    public let sdk: String
    public let indexStorePath: String
    public let indexDatabasePath: String
    public let aqueryArgs: [String]
    public let extraIncludes: [String]?
    public let extraFrameworks: [String]?
    public let logPath: String?
    public let truncateLogOnStartup: Bool?

    // Legacy support for old format
    public let defaultSettings: [String]?

    public init(
        name: String,
        argv: [String],
        version: String,
        bspVersion: String,
        languages: [String],
        target: String,
        sdk: String,
        indexStorePath: String,
        indexDatabasePath: String,
        aqueryArgs: [String],
        extraIncludes: [String]? = nil,
        extraFrameworks: [String]? = nil,
        logPath: String? = nil,
        truncateLogOnStartup: Bool? = nil
    ) {
        self.name = name
        self.argv = argv
        self.version = version
        self.bspVersion = bspVersion
        self.languages = languages
        self.target = target
        self.sdk = sdk
        self.indexStorePath = indexStorePath
        self.indexDatabasePath = indexDatabasePath
        self.aqueryArgs = aqueryArgs
        self.extraIncludes = extraIncludes
        self.extraFrameworks = extraFrameworks
        self.logPath = logPath
        self.truncateLogOnStartup = truncateLogOnStartup
        defaultSettings = nil
    }

    public static func parse(rootUri: String) throws -> BuildServerConfig {
        guard let rootURL = URL(string: rootUri) else {
            throw BSPError.configError("Invalid root URI: \(rootUri)")
        }

        let configPath = rootURL.appendingPathComponent("buildServer.json")

        guard let configData = try? Data(contentsOf: configPath) else {
            throw BSPError.configError("Could not read buildServer.json from: \(configPath.path)")
        }

        do {
            let config = try JSONDecoder().decode(BuildServerConfig.self, from: configData)
            return config
        } catch {
            throw BSPError.jsonError(error)
        }
    }
}

// MARK: - Additional Response Types

public struct WaitForBuildSystemUpdatesResponse: Codable {
    public init() {}

    public func toJSONValue() throws -> JSONValue {
        return .null
    }
}

public struct BuildTargetPrepareResponse: Codable {
    public init() {}

    public func toJSONValue() throws -> JSONValue {
        return .null
    }
}

public struct BuildShutdownResponse: Codable {
    public init() {}

    public func toJSONValue() throws -> JSONValue {
        return .null
    }
}

public struct DidChangeWatchedFilesNotification: Codable {
    public let method: String
    public let params: JSONValue?

    public init(params: JSONValue? = nil) {
        method = "workspace/didChangeWatchedFiles"
        self.params = params
    }

    public func toJSONValue() throws -> JSONValue {
        let data = try JSONEncoder().encode(self)
        return try JSONValue.from(data: data)
    }
}

public struct BuildExitNotification: Codable {
    public let method: String
    public let params: JSONValue?

    public init(params: JSONValue? = nil) {
        method = "build/exit"
        self.params = params
    }

    public func toJSONValue() throws -> JSONValue {
        let data = try JSONEncoder().encode(self)
        return try JSONValue.from(data: data)
    }
}

// MARK: - JSONValue Extensions

public extension JSONValue {
    static func from(data: Data) throws -> JSONValue {
        let json = try JSONSerialization.jsonObject(with: data)
        return try JSONValue.from(object: json)
    }

    static func from(object: Any) throws -> JSONValue {
        switch object {
        case is NSNull:
            return .null
        case let bool as Bool:
            return .bool(bool)
        case let number as NSNumber:
            return .number(number.doubleValue)
        case let string as String:
            return .string(string)
        case let array as [Any]:
            let jsonArray = try array.map { try JSONValue.from(object: $0) }
            return .array(jsonArray)
        case let dict as [String: Any]:
            let jsonDict = try dict.mapValues { try JSONValue.from(object: $0) }
            return .object(jsonDict)
        default:
            throw JSONRPCError.parseError("Unsupported JSON type: \(type(of: object))")
        }
    }

    func toData() throws -> Data {
        return try JSONEncoder().encode(self)
    }
}
