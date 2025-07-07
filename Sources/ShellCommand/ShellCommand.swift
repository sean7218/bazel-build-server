import Foundation

package struct ShellCommand {
    var executable: String
    var currentDir: String
    var args: [String]

    package init(
        executable: String,
        currentDir: String,
        args: [String]
    ) {
        self.executable = executable
        self.currentDir = currentDir
        self.args = args
    }

    package func run() -> (output: String, error: String, exitCode: Int32) {
        let task = Process()
        let outputPipe = Pipe()
        let errorPipe = Pipe()

        task.standardOutput = outputPipe
        task.standardError = errorPipe
        task.arguments = self.args
        task.executableURL = URL(fileURLWithPath: executable)

        do {
            try task.run()
            task.waitUntilExit()

            let outputData = outputPipe.fileHandleForReading.readDataToEndOfFile()
            let errorData = errorPipe.fileHandleForReading.readDataToEndOfFile()

            let output = String(data: outputData, encoding: .utf8) ?? ""
            let error = String(data: errorData, encoding: .utf8) ?? ""

            return (output, error, task.terminationStatus)
        } catch {
            return ("", "Failed to execute: \(error)", -1)
        }
    }
}

