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

    public func testing() throws {
        let btn = BazelView()
        print(btn.body)
        let logger = Logger(level: .debug)
        print(logger.level == LogLeverl.debug)
        let network = NetworkStack(host: "Hello")
        try network.request(url: "www.google.com")
        let level = LogLeverl.error
    }
}

#Preview {
    ContentView()
}
