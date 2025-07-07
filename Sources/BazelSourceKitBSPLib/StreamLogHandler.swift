import BSPError
import Foundation
import Logging

// MARK: - Custom Stream Log Handler

public struct StreamLogHandler: LogHandler, @unchecked Sendable {
    private let stream: TextOutputStream
    private let label: String

    public var logLevel: Logger.Level = .info
    public var metadata = Logger.Metadata()

    public init(label: String, stream: TextOutputStream) {
        self.label = label
        self.stream = stream
    }

    public static func standardOutput(label: String, logLevel: Logger.Level = .info) -> StreamLogHandler {
        var handler = StreamLogHandler(label: label, stream: StdoutOutputStream())
        handler.logLevel = logLevel
        return handler
    }

    public func log(level: Logger.Level, message: Logger.Message, metadata _: Logger.Metadata?, source _: String, file _: String, function _: String, line _: UInt) {
        let timestamp = ISO8601DateFormatter().string(from: Date())
        let levelString = level.rawValue.uppercased()

        var output = stream
        output.write("[\(timestamp)] [\(levelString)] [\(label)] \(message)\n")
    }

    public subscript(metadataKey key: String) -> Logger.Metadata.Value? {
        get { metadata[key] }
        set { metadata[key] = newValue }
    }
}

public struct StdoutOutputStream: TextOutputStream {
    public func write(_ string: String) {
        print(string, terminator: "")
    }
}

// MARK: - File Log Handler

public struct FileLogHandler: LogHandler, @unchecked Sendable {
    private let fileURL: URL
    private let label: String
    private let fileHandle: FileHandle

    public var logLevel: Logger.Level = .info
    public var metadata = Logger.Metadata()

    public init(label: String, fileURL: URL, truncate: Bool = false) throws {
        self.label = label
        self.fileURL = fileURL

        // Create directory if it doesn't exist
        let directory = fileURL.deletingLastPathComponent()
        if !FileManager.default.fileExists(atPath: directory.path) {
            try FileManager.default.createDirectory(
                at: directory,
                withIntermediateDirectories: true,
                attributes: nil
            )
        }

        // Create file if it doesn't exist, or truncate if requested
        if !FileManager.default.fileExists(atPath: fileURL.path) {
            FileManager.default.createFile(atPath: fileURL.path, contents: nil, attributes: nil)
        } else if truncate {
            // Truncate existing file
            try Data().write(to: fileURL)
        }

        // Open file for writing
        guard let handle = try? FileHandle(forWritingTo: fileURL) else {
            throw BSPError.ioError(NSError(domain: "FileLogHandler", code: 1, userInfo: [NSLocalizedDescriptionKey: "Cannot open log file for writing: \(fileURL.path)"]))
        }

        fileHandle = handle
        if !truncate {
            try handle.seekToEnd()
        }
    }

    public static func file(label: String, fileURL: URL, logLevel: Logger.Level = .info, truncate: Bool = false) throws -> FileLogHandler {
        var handler = try FileLogHandler(label: label, fileURL: fileURL, truncate: truncate)
        handler.logLevel = logLevel
        return handler
    }

    public func log(level: Logger.Level, message: Logger.Message, metadata _: Logger.Metadata?, source _: String, file _: String, function _: String, line _: UInt) {
        let timestamp = DateFormatter.logTimestamp.string(from: Date())
        let levelString = level.rawValue.uppercased()

        let logMessage = "[\(timestamp)] [\(levelString)] [\(label)] \(message)\n"

        if let data = logMessage.data(using: .utf8) {
            fileHandle.write(data)
            // Ensure logs are flushed to disk immediately
            fileHandle.synchronizeFile()
        }
    }

    public subscript(metadataKey key: String) -> Logger.Metadata.Value? {
        get { metadata[key] }
        set { metadata[key] = newValue }
    }
}

// MARK: - DateFormatter Extension

private extension DateFormatter {
    static let logTimestamp: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy-MM-dd HH:mm:ss"
        formatter.timeZone = TimeZone.current
        return formatter
    }()
}
