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

    mutating func run() throws {
        setupLogging()
        setupErrorHandling()

        let logger = Logger(label: "com.bazel.sourcekit.bsp")
        logger.info("🚀 Starting Bazel SourceKit BSP Server")

        do {
            try runBuildServer(logger: logger)
        } catch {
            logger.error("❌ Build server error: \(error)")
            throw error
        }
    }

    private mutating func setupLogging() {
        LoggingSystem.bootstrap { label in
            StreamLogHandler.standardOutput(label: label, logLevel: .debug)
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

        logger.info("🟢 Build Server Initialized")

        // Main request processing loop
        while true {
            do {
                let request = try server.readRequest()
                logger.debug("➡️ Received request: \(request.method)")

                let response = try requestHandler.handleRequest(request)

                switch response {
                case let .response(jsonResponse):
                    try server.sendResponse(jsonResponse)
                    logger.debug("↩️ Sent response for: \(request.method)")

                case let .notification(jsonNotification):
                    try server.sendNotification(jsonNotification)
                    logger.debug("↩️ Sent notification for: \(request.method)")

                case .none:
                    // No response needed for this request
                    logger.debug("ℹ️ No response needed for: \(request.method)")

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
