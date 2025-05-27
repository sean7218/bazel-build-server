import Foundation
import NetworkStack
import Utils

public struct Analytics {
    public enum Event {
        case One
        case Two
    }

    public var tags: [String]
    public init(tags: [String]) {
        self.tags = tags
    }
}

extension Analytics {
    public func call() {
        let logger = Logger(level: .debug)
        let network = NetworkStack(host: "test")
    }
}
