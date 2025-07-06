import Foundation
import Logging
import SystemPackage

/// Main BuildServer class that handles JSON-RPC communication
public class BuildServer {
    public let logger: Logger
    private let stdin: FileHandle
    private let stdout: FileHandle

    public init(logger: Logger) throws {
        self.logger = logger
        stdin = FileHandle.standardInput
        stdout = FileHandle.standardOutput
    }

    /// Handle the initial build/initialize request
    public func handleInitialization() throws -> RequestHandler {
        let request = try readRequest()

        guard request.method == "build/initialize" else {
            throw JSONRPCError.invalidRequest("Expected build/initialize, got \(request.method)")
        }

        let requestHandler = try RequestHandler.initialize(request: request, logger: logger)

        let response = try requestHandler.buildInitialize(request: request)
        try sendResponse(response)

        return requestHandler
    }

    /// Read a JSON-RPC request from stdin
    public func readRequest() throws -> JSONRPCRequest {
        // Read Content-Length header
        guard let headerData = readLine() else {
            throw JSONRPCError.endOfStream
        }

        let headerString = String(data: headerData, encoding: .utf8) ?? ""

        let contentLengthPrefix = "Content-Length: "
        guard headerString.hasPrefix(contentLengthPrefix) else {
            throw JSONRPCError.invalidRequest("Missing Content-Length header")
        }

        let lengthString = String(headerString.dropFirst(contentLengthPrefix.count)).trimmingCharacters(in: .whitespacesAndNewlines)
        guard let contentLength = Int(lengthString) else {
            throw JSONRPCError.invalidRequest("Invalid Content-Length: \(lengthString)")
        }

        // Read empty line
        _ = readLine()

        // Read JSON content
        let jsonData = try readBytes(count: contentLength)

        let jsonString = String(data: jsonData, encoding: .utf8) ?? "invalid UTF-8"
        let prettyJsonString = prettyPrintJSON(jsonString)
        logger.info("📨 Incoming request:\n\(prettyJsonString)")

        // Parse JSON
        do {
            let request = try JSONDecoder().decode(JSONRPCRequest.self, from: jsonData)
            return request
        } catch {
            throw JSONRPCError.parseError("Failed to parse JSON: \(error)")
        }
    }

    /// Send a JSON-RPC response to stdout
    public func sendResponse(_ response: JSONRPCResponse) throws {
        try sendJSON(response)
    }

    /// Send a JSON-RPC notification to stdout
    public func sendNotification(_ notification: JSONRPCNotification) throws {
        try sendJSON(notification)
    }

    // MARK: - Private Methods

    private func readLine() -> Data? {
        var line = Data()

        while true {
            guard let byte = stdin.readData(ofLength: 1).first else {
                break
            }

            if byte == 0x0A { // \n
                break
            }
            if byte != 0x0D { // skip \r
                line.append(byte)
            }
        }

        return line.isEmpty ? nil : line
    }

    private func readBytes(count: Int) throws -> Data {
        var data = Data()

        while data.count < count {
            let remaining = count - data.count
            let chunk = stdin.readData(ofLength: remaining)

            if chunk.isEmpty {
                throw JSONRPCError.endOfStream
            }

            data.append(chunk)
        }

        return data
    }

    private func sendJSON<T: Codable>(_ object: T) throws {
        let jsonData = try JSONEncoder.bspEncoder.encode(object)
        let jsonString = String(data: jsonData, encoding: .utf8) ?? ""

        // Pretty print for logging
        let prettyJsonString = prettyPrintJSON(jsonString)
        logger.info("📤 Outgoing response:\n\(prettyJsonString)")

        let message = "Content-Length: \(jsonData.count)\r\n\r\n\(jsonString)"
        let messageData = message.data(using: .utf8) ?? Data()

        stdout.write(messageData)
    }

    /// Pretty print JSON string for logging
    private func prettyPrintJSON(_ jsonString: String) -> String {
        guard let data = jsonString.data(using: .utf8) else {
            return jsonString
        }

        do {
            let jsonObject = try JSONSerialization.jsonObject(with: data, options: [])
            let encoder = JSONEncoder()
            encoder.outputFormatting = [.prettyPrinted, .sortedKeys, .withoutEscapingSlashes]
            let prettyData = try encoder.encode(JSONValue.from(object: jsonObject))
            return String(data: prettyData, encoding: .utf8) ?? jsonString
        } catch {
            return jsonString
        }
    }
}

// MARK: - JSON-RPC Types

public struct JSONRPCRequest: Codable {
    public let jsonrpc: String
    public let id: JSONRPCId?
    public let method: String
    public let params: JSONValue?

    public init(jsonrpc: String = "2.0", id: JSONRPCId? = nil, method: String, params: JSONValue? = nil) {
        self.jsonrpc = jsonrpc
        self.id = id
        self.method = method
        self.params = params
    }
}

public struct JSONRPCResponse: Codable {
    public let jsonrpc: String
    public let id: JSONRPCId?
    public let result: JSONValue?
    public let error: JSONRPCError?

    public init(jsonrpc: String = "2.0", id: JSONRPCId? = nil, result: JSONValue? = nil, error: JSONRPCError? = nil) {
        self.jsonrpc = jsonrpc
        self.id = id
        self.result = result
        self.error = error
    }
}

public struct JSONRPCNotification: Codable {
    public let jsonrpc: String
    public let method: String
    public let params: JSONValue?

    public init(jsonrpc: String = "2.0", method: String, params: JSONValue? = nil) {
        self.jsonrpc = jsonrpc
        self.method = method
        self.params = params
    }
}

public enum JSONRPCId: Codable, Hashable {
    case string(String)
    case number(Int)
    case null

    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()

        if container.decodeNil() {
            self = .null
        } else if let string = try? container.decode(String.self) {
            self = .string(string)
        } else if let number = try? container.decode(Int.self) {
            self = .number(number)
        } else {
            throw DecodingError.typeMismatch(JSONRPCId.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Invalid JSON-RPC ID: expected string, number, or null"))
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        switch self {
        case let .string(string):
            try container.encode(string)
        case let .number(number):
            try container.encode(number)
        case .null:
            try container.encodeNil()
        }
    }
}

public enum JSONValue: Codable {
    case null
    case bool(Bool)
    case number(Double)
    case string(String)
    case array([JSONValue])
    case object([String: JSONValue])

    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()

        if container.decodeNil() {
            self = .null
        } else if let bool = try? container.decode(Bool.self) {
            self = .bool(bool)
        } else if let number = try? container.decode(Double.self) {
            self = .number(number)
        } else if let string = try? container.decode(String.self) {
            self = .string(string)
        } else if let array = try? container.decode([JSONValue].self) {
            self = .array(array)
        } else if let object = try? container.decode([String: JSONValue].self) {
            self = .object(object)
        } else {
            throw DecodingError.typeMismatch(JSONValue.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Invalid JSON value"))
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        switch self {
        case .null:
            try container.encodeNil()
        case let .bool(bool):
            try container.encode(bool)
        case let .number(number):
            try container.encode(number)
        case let .string(string):
            try container.encode(string)
        case let .array(array):
            try container.encode(array)
        case let .object(object):
            try container.encode(object)
        }
    }
}

// MARK: - Error Types

public enum JSONRPCError: Error, Codable {
    case endOfStream
    case invalidRequest(String)
    case parseError(String)
    case internalError(String)

    public var errorDescription: String? {
        switch self {
        case .endOfStream:
            return "End of input stream"
        case let .invalidRequest(message):
            return "Invalid request: \(message)"
        case let .parseError(message):
            return "Parse error: \(message)"
        case let .internalError(message):
            return "Internal error: \(message)"
        }
    }
}

public enum BSPError: Error, CustomStringConvertible {
    case custom(String)
    case targetNotFound(String)
    case executionRootNotFound(String)
    case jsonError(Error)
    case ioError(Error)
    case configError(String)
    case bazelError(String)

    public var description: String {
        switch self {
        case let .custom(message):
            return "BSPError::Custom -> Reason: \(message)"
        case let .targetNotFound(message):
            return "BSPError::TargetNotFound -> Reason: \(message)"
        case let .executionRootNotFound(message):
            return "BSPError::ExecutionRootNotFound -> Reason: \(message)"
        case let .jsonError(error):
            return "BSPError::JsonError -> Reason: \(error)"
        case let .ioError(error):
            return "BSPError::IoError -> Reason: \(error)"
        case let .configError(message):
            return "BSPError::ConfigError -> Reason: \(message)"
        case let .bazelError(message):
            return "BSPError::BazelError -> Reason: \(message)"
        }
    }

    public var localizedDescription: String {
        return description
    }
}
