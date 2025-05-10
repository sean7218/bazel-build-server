import Foundation
import Utils

public struct Button {
    public let title: String
    public var onClick: () -> Void 
    public init(title: String, onClick: @escaping () -> Void) {
        self.title = title
        self.onClick = onClick
    }

    public func pressed() {
        self.onClick()
        let utils = AwesomeUtils(name: "Cool")
        let ut = AwesomeUtils(name: "Another")
        utils.hello()
        ut.hello()
    }
}