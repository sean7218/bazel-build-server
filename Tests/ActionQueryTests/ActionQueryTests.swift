import Foundation
import Testing

@testable import ActionQuery

final class ActionQueryTests {

    @Test
    func testQueryCommand() throws {
        guard let aqueryJson = Bundle.module.url(forResource: "aquery", withExtension: "json") else {
            Issue.record("Failed to find aquery.json resource")
            return
        }

        guard let data = try? Data(contentsOf: aqueryJson) else {
            Issue.record("Failed to read data from aquery.json")
            return
        }

        guard let queryResult = try? JSONDecoder().decode(QueryResult.self, from: data) else {
            Issue.record("Failed to decode QueryResult from JSON data")
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
                Issue.record("target: \(expected) not found in parsed action query result")
            }
        }
    }

    @Test
    func testBuildFilePath() throws {
        // Setup fragments: root/foo/bar.txt
        let fragRoot = PathFragment(id: 1, label: "root", parentId: nil)
        let fragFoo = PathFragment(id: 2, label: "foo", parentId: 1)
        let fragBar = PathFragment(id: 3, label: "bar.txt", parentId: 2)
        let fragments: [UInt32: PathFragment] = [1: fragRoot, 2: fragFoo, 3: fragBar]

        // Use ActionQuery's buildFilePath (exposed via extension for test)
        let actionQuery = ActionQuery()
        let path = actionQuery.buildFilePath(fragments: fragments, leafId: 3)
        #expect(path == "root/foo/bar.txt")
    }
}
