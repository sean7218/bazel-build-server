import Foundation

public struct AwesomeUtils {
    public let name: String

    public init(name: String) {
        self.name = name
    }

    public func hello() {
        print("say hello");
    }

    public func world() {
        print("say world")
    }
}

extension AwesomeUtils {

}