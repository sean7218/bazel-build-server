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

        let logURL = FileManager
            .default
            .homeDirectoryForCurrentUser
            .appending(path: ".bazel-sourcekit-bsp/bsp.log")

        // Bootstrap logging to use file handler from the start
        LoggingSystem.bootstrap { label in
            do {
                return try FileLogHandler.file(label: label, fileURL: logURL, logLevel: .debug, truncate: true)
            } catch {
                // Fallback to stdout if file logging fails
                print("⚠️ Failed to setup file logging at startup: \(error). Using stdout instead.")
                return StreamLogHandler.standardOutput(label: label, logLevel: .debug)
            }
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
        let requestHandler = try server.handleInitialization()

        // Main request processing loop
        while true {
            do {
                let request = try server.readRequest()
                let response = try requestHandler.handleRequest(request)

                switch response {
                case let .response(jsonResponse):
                    try server.sendResponse(jsonResponse)

                case let .notification(jsonNotification):
                    try server.sendNotification(jsonNotification)

                case .none:
                    // No response needed for this request
                    break

                case .exit:
                    logger.info("👋 Received exit request, shutting down...")
                }

            } catch JSONRPCError.endOfStream {
                logger.info("📤 Client disconnected")
                break
            } catch {
                logger.error("❌ Error processing request: \(error)")
                // Continue processing other requests
            }
        }

        logger.info("🏁 Build server shutdown complete")
    }
}
