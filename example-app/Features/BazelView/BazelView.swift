import Analytics
import Foundation
import HexColors
import NetworkStack
import SwiftUI
import Utils
import ObjcFunc
import ObjcFuncSwift

public struct BazelView: View {
    public init() {}
    public var body: some View {
        Text("BazelView")
            .foregroundColor(#color("#000000"))
    }
}

public extension BazelView {
    static func Hello() {
        let network = NetworkStack(host: "test")
        let logger = Logger(level: .debug)
        let analytics = Analytics(tags: ["hello"])
        let objcFunc = ObjcFunc(name: "HelloObjc", count: 1)
        let objcFuncSwift = ObjcFuncSwift(name: "HelloObjc", count: 1)
    }
}
