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
        let logger = Logger(level: .debug)
    }
}

#Preview {
    ContentView()
}

