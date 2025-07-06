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
