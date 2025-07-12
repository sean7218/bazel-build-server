import Foundation

/// Bazel ActionQuery generates list of targets
///   - label: bazel [label](https://bazel.build/rules/lib/builtins/Label)
///   - inputFiles: list of source files, these are absolute path.
///   - compilerArguments: list of compiler arguments used by sourcekit-lsp
package struct BazelTarget: Codable, Hashable, Sendable {
    package let id: UInt32
    package let uri: String
    package let label: String
    package let kind: String
    package let tags: [String]
    package let inputFiles: [String]
    package let compilerArguments: [String]

    package init(
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

    package func hash(into hasher: inout Hasher) {
        hasher.combine(id)
        hasher.combine(label)
    }

    package static func == (lhs: BazelTarget, rhs: BazelTarget) -> Bool {
        return lhs.id == rhs.id && lhs.label == rhs.label
    }
}
