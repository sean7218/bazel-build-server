import Foundation
import XCTest

@testable import ShellCommand

final class ShellCommandTests: XCTestCase {
    func testShellCommand() throws {
        let currentDir = FileManager.default.currentDirectoryPath
        let currentURL = URL(fileURLWithPath: currentDir)
        let examplePath =
            currentURL
            .appendingPathComponent("example-app")
            .path()

        let command = ShellCommand(
            executable: "bazel",
            currentDir: examplePath,
            args: ["info", "execution_root"]
        )
        guard let output = command.run().output else {
            XCTFail()
            return
        }
        XCTAssert(output.contains("execroot/_main"))
    }

    func tesAqueryCommand() throws {
        let currentDir = FileManager.default.currentDirectoryPath
        let currentURL = URL(fileURLWithPath: currentDir)
        let examplePath =
            currentURL
            .appendingPathComponent("example-app")
            .path()

        let command = ShellCommand(
            executable: "bazel",
            currentDir: examplePath,
            args: ["aquery", "mnemonic(\"SwiftCompile\", deps(//App:App))"]
        )
        guard let output = command.run().output else {
            XCTFail()
            return
        }
        XCTAssert(output != "")
    }
}
