import Foundation

// MARK: - Bazel Query Result Types

/// Proto Scheme based on the following
/// https://github.com/bazelbuild/bazel/blob/master/src/main/protobuf/analysis_v2.proto
package struct QueryResult: Codable {
    package let artifacts: [Artifact]
    package let actions: [Action]
    package let targets: [Target]
    package let ruleClasses: [RuleClass]
    package let depSetOfFiles: [DepSetOfFiles]
    package let pathFragments: [PathFragment]
}

package struct Artifact: Codable {
    package let id: UInt32
    package let pathFragmentId: UInt32
    package let isTreeArtifact: Bool?
}

package struct Action: Codable {
    package let targetId: UInt32
    package let actionKey: String
    package let mnemonic: String
    package let configurationId: UInt32
    package let arguments: [String]
    package let environmentVariables: [EnvironmentVariable]
    package let inputDepSetIds: [UInt32]
}

package struct Target: Codable {
    package let id: UInt32
    package let label: String
    package let ruleClassId: UInt32
}

package struct RuleClass: Codable {
    package let id: UInt32
    package let name: String
}

package struct EnvironmentVariable: Codable {
    package let key: String
    package let value: String
}

package struct PathFragment: Codable {
    package let id: UInt32
    package let label: String
    package let parentId: UInt32?
}

package struct DepSetOfFiles: Codable {
    package let id: UInt32
    package let directArtifactIds: [UInt32]?
    package let transitiveDepSetIds: [UInt32]?
}
