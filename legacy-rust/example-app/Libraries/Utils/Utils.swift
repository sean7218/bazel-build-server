import Foundation

public struct Logger {
    public let level: LogLeverl

    public init(level: LogLeverl) {
        self.level = level
    }
}

public enum LogLeverl {
    case debug
    case info
    case error
}
