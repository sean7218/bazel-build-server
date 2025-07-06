import Analytics
import Foundation
import NetworkStack
import SwiftUI
import Utils

public struct BazelView: View {
    public init() {}
    public var body: some View {
        Text("BazelView")
    }
}

extension BazelView {
    public static func Hello() {
        let network = NetworkStack(host: "test")
        let logger = Logger(level: .debug)
        let analytics = Analytics(tags: ["hello"])
    }
}
