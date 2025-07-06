import Foundation
import Logging
import ShellOut

// MARK: - Bazel Query Result Types

/// Proto scheme based on Bazel analysis_v2.proto
public struct QueryResult: Codable {
    public let artifacts: [Artifact]
    public let actions: [Action]
    public let targets: [Target]
    public let ruleClasses: [RuleClass]
    public let depSetOfFiles: [DepSetOfFiles]
    public let pathFragments: [PathFragment]
}

public struct Artifact: Codable {
    public let id: UInt32
    public let pathFragmentId: UInt32
    public let isTreeArtifact: Bool?
}

public struct Action: Codable {
    public let targetId: UInt32
    public let actionKey: String
    public let mnemonic: String
    public let configurationId: UInt32
    public let arguments: [String]
    public let environmentVariables: [EnvironmentVariable]
    public let inputDepSetIds: [UInt32]
}

public struct Target: Codable {
    public let id: UInt32
    public let label: String
    public let ruleClassId: UInt32
}

public struct RuleClass: Codable {
    public let id: UInt32
    public let name: String
}

public struct EnvironmentVariable: Codable {
    public let key: String
    public let value: String
}

public struct PathFragment: Codable {
    public let id: UInt32
    public let label: String
    public let parentId: UInt32?
}

public struct DepSetOfFiles: Codable {
    public let id: UInt32
    public let directArtifactIds: [UInt32]?
    public let transitiveDepSetIds: [UInt32]?
}

// MARK: - Bazel Query Functions

/// Executes Bazel aquery and returns processed targets
public func executeAquery(
    target: String,
    rootPath: URL,
    execrootPath: URL,
    sdk: String,
    aqueryArgs: [String],
    extraIncludes: [String],
    extraFrameworks: [String],
    logger: Logger
) throws -> [BazelTarget] {
    logger.debug("✨ Executing aquery for target: \(target)")

    let mnemonic = "mnemonic(\"SwiftCompile\", deps(\(target)))"

    var commandArgs: [String] = [
        "aquery",
        mnemonic,
        "--output=jsonproto",
    ]
    commandArgs.append(contentsOf: aqueryArgs)

    logger.debug("🔍 aquery command: bazel \(commandArgs.joined(separator: " "))")

    let output = try shellOut(
        to: "bazel",
        arguments: commandArgs,
        at: rootPath.path
    )

    let queryResult = try parseQueryResult(output: output)

    return try processBazelTargets(
        queryResult: queryResult,
        rootPath: rootPath,
        execrootPath: execrootPath,
        sdk: sdk,
        extraIncludes: extraIncludes,
        extraFrameworks: extraFrameworks,
        logger: logger
    )
}

/// Parses Bazel aquery JSON proto output
private func parseQueryResult(output: String) throws -> QueryResult {
    guard let data = output.data(using: .utf8) else {
        throw BSPError.bazelError("Failed to convert aquery output to UTF-8 data")
    }

    do {
        let decoder = JSONDecoder()
        return try decoder.decode(QueryResult.self, from: data)
    } catch {
        throw BSPError.jsonError(error)
    }
}

/// Processes the query result into BazelTarget objects
private func processBazelTargets(
    queryResult: QueryResult,
    rootPath: URL,
    execrootPath: URL,
    sdk: String,
    extraIncludes: [String],
    extraFrameworks: [String],
    logger: Logger
) throws -> [BazelTarget] {
    // Convert arrays to hashmaps for efficient lookup
    let artifacts = Dictionary(uniqueKeysWithValues: queryResult.artifacts.map { ($0.id, $0) })
    let files = Dictionary(uniqueKeysWithValues: queryResult.depSetOfFiles.map { ($0.id, $0) })
    let fragments = Dictionary(uniqueKeysWithValues: queryResult.pathFragments.map { ($0.id, $0) })

    var bazelTargets: [BazelTarget] = []

    for action in queryResult.actions {
        let inputFiles = try buildInputFiles(
            artifacts: artifacts,
            files: files,
            fragments: fragments,
            action: action,
            rootPath: rootPath
        )

        let compilerArguments = try processCompilerArguments(
            action: action,
            execrootPath: execrootPath,
            sdk: sdk,
            extraIncludes: extraIncludes,
            extraFrameworks: extraFrameworks
        )

        guard let target = queryResult.targets.first(where: { $0.id == action.targetId }) else {
            logger.warning("Target not found for action: \(action.targetId)")
            continue
        }

        let uri = try bazelToUri(rootPath: rootPath, label: target.label, id: target.id)

        let bazelTarget = BazelTarget(
            id: action.targetId,
            uri: uri,
            label: target.label,
            kind: "swift_library", // TODO: Get from rule class
            tags: [],
            inputFiles: inputFiles,
            compilerArguments: compilerArguments
        )

        bazelTargets.append(bazelTarget)
    }

    // Deduplicate targets
    let uniqueTargets = Array(Set(bazelTargets))
    logger.info("📦 Processed \(uniqueTargets.count) unique targets")

    return uniqueTargets
}

/// Builds input files from artifacts
private func buildInputFiles(
    artifacts: [UInt32: Artifact],
    files: [UInt32: DepSetOfFiles],
    fragments: [UInt32: PathFragment],
    action: Action,
    rootPath: URL
) throws -> [String] {
    var inputFiles: [String] = []

    for depSetId in action.inputDepSetIds {
        let artifactIds = buildArtifactIds(fileSet: files[depSetId], files: files)

        for artifactId in artifactIds {
            guard let artifact = artifacts[artifactId] else { continue }

            let filePath = buildFilePath(fragments: fragments, leafId: artifact.pathFragmentId)

            // Convert to URL and filter for Swift files
            let fullPath = rootPath.appendingPathComponent(filePath)
            if fullPath.pathExtension == "swift" {
                inputFiles.append(fullPath.absoluteString)
            }
        }
    }

    return inputFiles
}

/// Recursively builds artifact IDs from dep sets
private func buildArtifactIds(fileSet: DepSetOfFiles?, files: [UInt32: DepSetOfFiles]) -> [UInt32] {
    guard let fileSet = fileSet else { return [] }

    var artifactIds: [UInt32] = []

    // Add direct artifacts
    if let directIds = fileSet.directArtifactIds {
        artifactIds.append(contentsOf: directIds)
    }

    // Add transitive artifacts
    if let transitiveIds = fileSet.transitiveDepSetIds {
        for transitiveId in transitiveIds {
            let transitiveIds = buildArtifactIds(fileSet: files[transitiveId], files: files)
            artifactIds.append(contentsOf: transitiveIds)
        }
    }

    return artifactIds
}

/// Builds file path from fragments
private func buildFilePath(fragments: [UInt32: PathFragment], leafId: UInt32) -> String {
    guard let leaf = fragments[leafId] else { return "" }

    if let parentId = leaf.parentId {
        let parentPath = buildFilePath(fragments: fragments, leafId: parentId)
        return parentPath + "/" + leaf.label
    } else {
        return leaf.label
    }
}

/// Processes compiler arguments with transformations
private func processCompilerArguments(
    action: Action,
    execrootPath: URL,
    sdk: String,
    extraIncludes: [String],
    extraFrameworks: [String]
) throws -> [String] {
    var compilerArguments: [String] = []

    var index = 0
    let count = action.arguments.count

    while index < count {
        let arg = action.arguments[index]

        // Skip swiftc executable and wrapper arguments
        if arg.contains("-Xwrapped-swift") || arg.hasSuffix("worker") || arg.hasPrefix("swiftc") {
            index += 1
            continue
        }

        // Skip batch mode (incompatible with -index-file)
        if arg.contains("-enable-batch-mode") {
            index += 1
            continue
        }

        // Skip index store path arguments
        if arg.contains("-index-store-path") {
            if index + 1 < count, action.arguments[index + 1].contains("indexstore") {
                index += 2
                continue
            }
        }

        // Skip const-gather-protocols arguments
        if arg.contains("-Xfrontend"), index + 1 < count {
            let nextArg = action.arguments[index + 1]
            if nextArg.contains("-const-gather-protocols-file") || nextArg.contains("const_protocols_to_gather.json") {
                index += 2
                continue
            }
        }

        // Replace SDK placeholder
        if arg.contains("__BAZEL_XCODE_SDKROOT__") {
            let transformedArg = arg.replacingOccurrences(of: "__BAZEL_XCODE_SDKROOT__", with: sdk)
            compilerArguments.append(transformedArg)
            index += 1
            continue
        }

        // Transform bazel-out/ paths
        if arg.contains("bazel-out/") {
            let prefix = "\(execrootPath.path)/bazel-out/"
            let transformedArg = arg.replacingOccurrences(of: "bazel-out/", with: prefix)
            compilerArguments.append(transformedArg)
            index += 1
            continue
        }

        // Transform external/ paths
        if arg.contains("external/") {
            let prefix = "\(execrootPath.path)/external/"
            let transformedArg = arg.replacingOccurrences(of: "external/", with: prefix)
            compilerArguments.append(transformedArg)
            index += 1
            continue
        }

        compilerArguments.append(arg)
        index += 1
    }

    // Add extra includes
    for include in extraIncludes {
        compilerArguments.append("-I\(include)")
    }

    // Add extra frameworks
    for framework in extraFrameworks {
        compilerArguments.append("-F\(framework)")
    }

    return compilerArguments
}

/// Converts Bazel label to URI
private func bazelToUri(rootPath _: URL, label: String, id: UInt32) throws -> String {
    // Simple implementation - in practice you might want more sophisticated URI generation
    return "bazel://\(label)#\(id)"
}
