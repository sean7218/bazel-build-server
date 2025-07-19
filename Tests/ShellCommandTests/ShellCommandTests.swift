import Foundation
import Testing

@testable import ShellCommand

final class ShellCommandTests {
    @Test
    func testShellCommand() throws {
        let currentDir = FileManager.default.currentDirectoryPath
        let currentURL = URL(fileURLWithPath: currentDir)
        let examplePath =
            currentURL
            .appendingPathComponent("TestHarness")
            .path()

        let command = ShellCommand(
            executable: "bazel",
            currentDir: examplePath,
            args: ["info", "execution_root"]
        )
        guard let output = command.run().output else {
            Issue.record("Failed to get output from command")
            return
        }
        #expect(output.contains("execroot/_main"))
    }

    @Test
    func tesAqueryCommand() throws {
        let currentDir = FileManager.default.currentDirectoryPath
        let currentURL = URL(fileURLWithPath: currentDir)
        let examplePath =
            currentURL
            .appendingPathComponent("TestHarness")
            .path()

        let command = ShellCommand(
            executable: "bazel",
            currentDir: examplePath,
            args: ["aquery", "mnemonic(\"SwiftCompile\", deps(//App:App))"]
        )
        guard let output = command.run().output else {
            Issue.record("Failed to get output from aquery command")
            return
        }
        #expect(output != "")
    }
}
