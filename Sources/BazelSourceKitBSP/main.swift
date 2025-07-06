import ArgumentParser
import BazelSourceKitBSPLib
import Foundation
import Logging

@main
struct BazelSourceKitBSP: ParsableCommand {
    static let configuration = CommandConfiguration(
        commandName: "bazel-sourcekit-bsp",
        abstract: "Bazel Build Server Protocol implementation for Swift and iOS projects",
        version: "1.0.0"
    )

    func run() throws {
        setupErrorHandling()

        // We need to read the config first to get the log path
        // Start with stdout logging initially
        LoggingSystem.bootstrap { label in
            StreamLogHandler.standardOutput(label: label, logLevel: .debug)
        }

        let logger = Logger(label: "com.bazel.sourcekit.bsp")
        logger.info("🚀 Starting Bazel SourceKit BSP Server")

        do {
            try runBuildServer(logger: logger)
        } catch {
            logger.error("❌ Build server error: \(error)")
            throw error
        }
    }

    private func setupFileLogging(config: BuildServerConfig) throws {
        let logPath = config.logPath ?? "~/.sourcekit-bsp/bsp.log"

        // Expand tilde (~) to home directory
        let expandedLogPath = NSString(string: logPath).expandingTildeInPath
        let logURL = URL(fileURLWithPath: expandedLogPath)

        // Default to truncating logs on startup for fresh logs each session
        let truncateOnStartup = config.truncateLogOnStartup ?? true

        // Re-bootstrap logging to use file handler
        LoggingSystem.bootstrap { label in
            do {
                return try FileLogHandler.file(label: label, fileURL: logURL, logLevel: .debug, truncate: truncateOnStartup)
            } catch {
                // Fallback to stdout if file logging fails
                print("⚠️ Failed to setup file logging: \(error). Using stdout instead.")
                return StreamLogHandler.standardOutput(label: label, logLevel: .debug)
            }
        }
    }

    private func setupErrorHandling() {
        // Set up signal handlers for graceful shutdown
        signal(SIGINT) { _ in
            Logger(label: "signal").info("🛑 Received SIGINT, shutting down...")
            Foundation.exit(0)
        }

        signal(SIGTERM) { _ in
            Logger(label: "signal").info("🛑 Received SIGTERM, shutting down...")
            Foundation.exit(0)
        }
    }

    private func runBuildServer(logger: Logger) throws {
        let server = try BuildServer(logger: logger)

        // Handle initialization request first
        logger.debug("📥 Waiting for initialization request...")
        let requestHandler = try server.handleInitialization()

        // Switch to file logging now that we have the config
        try setupFileLogging(config: requestHandler.config)

        // Create a new logger instance after switching to file logging
        let fileLogger = Logger(label: "com.bazel.sourcekit.bsp")
        fileLogger.info("🟢 Build Server Initialized - switched to file logging")

        // Main request processing loop
        while true {
            do {
                let request = try server.readRequest()
                fileLogger.debug("➡️ Received request: \(request.method)")

                let response = try requestHandler.handleRequest(request)

                switch response {
                case let .response(jsonResponse):
                    try server.sendResponse(jsonResponse)
                    fileLogger.debug("↩️ Sent response for: \(request.method)")

                case let .notification(jsonNotification):
                    try server.sendNotification(jsonNotification)
                    fileLogger.debug("↩️ Sent notification for: \(request.method)")

                case .none:
                    // No response needed for this request
                    fileLogger.debug("ℹ️ No response needed for: \(request.method)")

                case .exit:
                    fileLogger.info("👋 Received exit request, shutting down...")
                }

            } catch JSONRPCError.endOfStream {
                fileLogger.info("📤 Client disconnected")
                break
            } catch {
                fileLogger.error("❌ Error processing request: \(error)")
                // Continue processing other requests
            }
        }

        fileLogger.info("🏁 Build server shutdown complete")
    }
}
