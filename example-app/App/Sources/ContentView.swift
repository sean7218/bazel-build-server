import Analytics
import BazelView
import Foundation
import NetworkStack
import SwiftUI
import Utils

public struct ContentView: View {
    public var body: some View {
        VStack {
            Image(systemName: "globe")
                .imageScale(.large)
                .foregroundStyle(.tint)
            Text("Hello, world!")
        }
        .padding()
    }

    public func testing() {
        let btn = BazelView()
        print(btn.body)
        let logger = Logger(level: .debug)
        print(logger.level == LogLeverl.debug)
        let network = NetworkStack.init(host: "hello")
        print(network.host)
    }
}

#Preview {
    ContentView()
}

