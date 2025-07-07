import Foundation

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
