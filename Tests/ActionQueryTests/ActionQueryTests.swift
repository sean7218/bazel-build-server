import Foundation
import XCTest

@testable import ActionQuery

final class ActionQueryTests: XCTestCase {

    func testQueryCommand() throws {
        guard let aqueryJson = Bundle.module.url(forResource: "aquery", withExtension: "json") else {
            XCTFail()
            return
        }

        guard let data = try? Data(contentsOf: aqueryJson) else {
            XCTFail()
            return
        }

        guard let queryResult = try? JSONDecoder().decode(QueryResult.self, from: data) else {
            XCTFail()
            return
        }

        let expectedTargets = [
            "//Features/BazelView:BazelView",
            "//App:Sources",
            "//Libraries/Utils:Utils",
            "//Libraries/NetworkStack:NetworkStack",
            "//Libraries/Analytics:Analytics",
            "@@+_repo_rules2+SwiftNonEmpty//:SwiftNonEmpty",
            "@@+_repo_rules+JOSESwift//:JOSESwift",
        ]

        let actualTargets = queryResult.targets.map({ $0.label })

        for expected in expectedTargets {
            if actualTargets.contains(expected) {
                continue
            } else {
                XCTFail("target: \(expected) not found in parsed action query result")
            }
        }
    }
}
