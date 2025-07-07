import BSPError
import Foundation
import ShellCommand

// MARK: - Bazel Query Functions

package struct ActionQuery: Sendable {
    package init() {}

    /// Executes Bazel aquery and returns processed targets
    package func execute(
        target: String,
        rootPath: URL,
        execrootPath: URL,
        sdk: String,
        aqueryArgs: [String],
        extraIncludes: [String],
        extraFrameworks: [String],
    ) throws -> [BazelTarget] {
        let mnemonic = "'mnemonic(\"SwiftCompile\", deps(\(target)))'"

        var commandArgs: [String] = [
            "aquery",
            mnemonic,
            "--output=jsonproto",
        ]
        commandArgs.append(contentsOf: aqueryArgs)

        // logger.debug("bazel \(commandArgs.joined(separator: " "))")

        guard let output = ShellCommand(
            executable: "bazel",
            currentDir: rootPath.path(),
            args: commandArgs,
        ).run().output else {
            fatalError("Failed to run ActionQuery")
        }

        let queryResult = try parseQueryResult(output: output)

        return try processBazelTargets(
            queryResult: queryResult,
            rootPath: rootPath,
            execrootPath: execrootPath,
            sdk: sdk,
            extraIncludes: extraIncludes,
            extraFrameworks: extraFrameworks,
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
        extraFrameworks: [String]
    ) throws -> [BazelTarget] {
        // Convert arrays to hashmaps for efficient lookup
        let artifacts = Dictionary(uniqueKeysWithValues: queryResult.artifacts.map { ($0.id, $0) })
        let files = Dictionary(uniqueKeysWithValues: queryResult.depSetOfFiles.map { ($0.id, $0) })
        let fragments = Dictionary(
            uniqueKeysWithValues: queryResult.pathFragments.map { ($0.id, $0) })

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
                extraFrameworks: extraFrameworks,
            )

            guard let target = queryResult.targets.first(where: { $0.id == action.targetId }) else {
                print("Target not found for action: \(action.targetId)")
                continue
            }

            let uri = try bazelToUri(rootPath: rootPath, label: target.label, id: target.id)

            let bazelTarget = BazelTarget(
                id: action.targetId,
                uri: uri,
                label: target.label,
                kind: "swift_library",  // TODO: Get from rule class
                tags: [],
                inputFiles: inputFiles,
                compilerArguments: compilerArguments
            )

            bazelTargets.append(bazelTarget)
        }

        // Deduplicate targets
        let uniqueTargets = Array(Set(bazelTargets))
        return uniqueTargets
    }

    /// Builds input files from artifacts
    private func buildInputFiles(
        artifacts: [UInt32: Artifact],
        files: [UInt32: DepSetOfFiles],
        fragments: [UInt32: PathFragment],
        action: Action,
        rootPath: URL,
    ) throws -> [String] {
        var inputFiles: [String] = []
        var validPaths: [String] = []
        var invalidPaths: [String] = []

        for depSetId in action.inputDepSetIds {
            let artifactIds = buildArtifactIds(fileSet: files[depSetId], files: files)

            for artifactId in artifactIds {
                guard let artifact = artifacts[artifactId] else { continue }

                let filePath = buildFilePath(fragments: fragments, leafId: artifact.pathFragmentId)

                // Convert to URL and filter for Swift files
                let fullPath = rootPath.appendingPathComponent(filePath)
                if fullPath.pathExtension == "swift" {
                    // Check if file exists
                    let fileExists = FileManager.default.fileExists(atPath: fullPath.path)

                    if fileExists {
                        inputFiles.append(fullPath.absoluteString)
                        validPaths.append(fullPath.path)
                    } else {
                        invalidPaths.append(fullPath.path)
                    }
                }
            }
        }

        return inputFiles
    }

    /// Recursively builds artifact IDs from dep sets
    private func buildArtifactIds(fileSet: DepSetOfFiles?, files: [UInt32: DepSetOfFiles])
        -> [UInt32]
    {
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
        var validArgPaths: [String] = []
        var invalidArgPaths: [String] = []

        var index = 0
        let count = action.arguments.count

        while index < count {
            let arg = action.arguments[index]

            // Skip swiftc executable and wrapper arguments
            if arg.contains("-Xwrapped-swift") || arg.hasSuffix("worker") || arg.hasPrefix("swiftc")
            {
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
                if nextArg.contains("-const-gather-protocols-file")
                    || nextArg.contains("const_protocols_to_gather.json")
                {
                    index += 2
                    continue
                }
            }

            // Replace SDK placeholder
            if arg.contains("__BAZEL_XCODE_SDKROOT__") {
                let transformedArg = arg.replacingOccurrences(
                    of: "__BAZEL_XCODE_SDKROOT__", with: sdk)
                validateArgumentPath(
                    arg: transformedArg, validPaths: &validArgPaths, invalidPaths: &invalidArgPaths)
                compilerArguments.append(transformedArg)
                index += 1
                continue
            }

            // Transform bazel-out/ paths
            if arg.contains("bazel-out/") {
                let prefix = "\(execrootPath.path)/bazel-out/"
                let transformedArg = arg.replacingOccurrences(of: "bazel-out/", with: prefix)
                validateArgumentPath(
                    arg: transformedArg, validPaths: &validArgPaths, invalidPaths: &invalidArgPaths)
                compilerArguments.append(transformedArg)
                index += 1
                continue
            }

            // Transform external/ paths
            if arg.contains("external/") {
                let prefix = "\(execrootPath.path)/external/"
                let transformedArg = arg.replacingOccurrences(of: "external/", with: prefix)
                validateArgumentPath(
                    arg: transformedArg, validPaths: &validArgPaths, invalidPaths: &invalidArgPaths)
                compilerArguments.append(transformedArg)
                index += 1
                continue
            }

            validateArgumentPath(
                arg: arg, validPaths: &validArgPaths, invalidPaths: &invalidArgPaths)
            compilerArguments.append(arg)
            index += 1
        }

        // Add extra includes
        for include in extraIncludes {
            let includeArg = "-I\(include)"
            validateArgumentPath(
                arg: includeArg, validPaths: &validArgPaths, invalidPaths: &invalidArgPaths)
            compilerArguments.append(includeArg)
        }

        // Add extra frameworks
        for framework in extraFrameworks {
            let frameworkArg = "-F\(framework)"
            validateArgumentPath(
                arg: frameworkArg, validPaths: &validArgPaths, invalidPaths: &invalidArgPaths)
            compilerArguments.append(frameworkArg)
        }

        return compilerArguments
    }

    /// Validates paths in compiler arguments
    private func validateArgumentPath(
        arg: String, validPaths: inout [String], invalidPaths: inout [String]
    ) {
        // Check if argument looks like a file path (contains / and doesn't start with -)
        if arg.contains("/") && !arg.hasPrefix("-") {
            let fileExists = FileManager.default.fileExists(atPath: arg)
            if fileExists {
                validPaths.append(arg)
            } else {
                invalidPaths.append(arg)
            }
        }
        // Check for -I and -F flag paths
        else if arg.hasPrefix("-I") || arg.hasPrefix("-F") {
            let pathPart = String(arg.dropFirst(2))
            if !pathPart.isEmpty {
                let fileExists = FileManager.default.fileExists(atPath: pathPart)
                if fileExists {
                    validPaths.append(pathPart)
                } else {
                    invalidPaths.append(pathPart)
                }
            }
        }
    }

    /// Converts Bazel label to URI
    private func bazelToUri(rootPath _: URL, label: String, id: UInt32) throws -> String {
        // Simple implementation - in practice you might want more sophisticated URI generation
        return "bazel://\(label)#\(id)"
    }

}
