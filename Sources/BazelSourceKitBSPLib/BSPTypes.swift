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
            id: BuildTargetIdentifier(uri: bazelTarget.label),
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
    public let kind: SourceItemKind
    public let generated: Bool

    public init(uri: String, kind: SourceItemKind, generated: Bool) {
        self.uri = uri
        self.kind = kind
        self.generated = generated
    }
}

public enum SourceItemKind: String, Codable {
    case file
    case directory
}

// MARK: - SourceKit Options

public struct TextDocumentSourceKitOptionsRequest: Codable {
    public let textDocument: TextDocumentIdentifier

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
    public let options: [String]
    public let workingDirectory: String

    public init(options: [String], workingDirectory: String) {
        self.options = options
        self.workingDirectory = workingDirectory
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
    public let workingDirectory: String
    public let flags: [String]

    public init(workingDirectory: String, flags: [String]) {
        self.workingDirectory = workingDirectory
        self.flags = flags
    }
}

// MARK: - Bazel Target Types

public struct BazelTarget: Codable {
    public let label: String
    public let kind: String
    public let tags: [String]

    public init(label: String, kind: String, tags: [String]) {
        self.label = label
        self.kind = kind
        self.tags = tags
    }
}

// MARK: - Build Server Config

public struct BuildServerConfig: Codable {
    public let defaultSettings: [String]
    public let targets: [String]?

    public init(defaultSettings: [String], targets: [String]? = nil) {
        self.defaultSettings = defaultSettings
        self.targets = targets
    }

    public static func parse(rootUri _: String) throws -> BuildServerConfig {
        // For now, return a default config
        // In practice, this would read from a buildServer.json file
        return BuildServerConfig(
            defaultSettings: [
                "-I", ".",
                "-I", "bazel-out",
                "-swift-version", "5",
            ]
        )
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
