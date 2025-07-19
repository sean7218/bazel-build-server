// swift-tools-version: 6.1
import PackageDescription

let package = Package(
    name: "bazel-build-server",
    platforms: [
        .macOS(.v13),
    ],
    products: [
        .executable(
            name: "bazel-build-server",
            targets: ["BazelBuildServer"]
        ),
        .library(
            name: "BazelBuildServerLib",
            targets: ["BazelBuildServerLib"]
        ),
    ],
    dependencies: [
        .package(url: "https://github.com/apple/swift-log.git", from: "1.0.0"),
        .package(url: "https://github.com/apple/swift-nio.git", from: "2.0.0"),
        .package(url: "https://github.com/apple/swift-argument-parser.git", from: "1.0.0"),
        .package(url: "https://github.com/apple/swift-system.git", from: "1.0.0"),
    ],
    targets: [
        .executableTarget(
            name: "BazelBuildServer",
            dependencies: [
                "BazelBuildServerLib",
                .product(name: "ArgumentParser", package: "swift-argument-parser"),
                .product(name: "Logging", package: "swift-log"),
            ]
        ),

        .target(
            name: "BazelBuildServerLib",
            dependencies: [
                .product(name: "Logging", package: "swift-log"),
                .product(name: "NIO", package: "swift-nio"),
                .product(name: "NIOHTTP1", package: "swift-nio"),
                .product(name: "NIOFoundationCompat", package: "swift-nio"),
                .product(name: "SystemPackage", package: "swift-system"),
                "ActionQuery",
                "BSPError",
                "ShellCommand"
            ]
        ),
        .target(
            name: "ActionQuery",
            dependencies: [
                "BSPError",
                "ShellCommand",
                .product(name: "Logging", package: "swift-log"),
            ]
        ),
        .target(name: "BSPError"),
        .target(name: "ShellCommand"),
        .testTarget(
            name: "ActionQueryTests",
            dependencies: ["ActionQuery"],
            resources: [
                .copy("Resources/aquery.json"),
                .copy("Resources/aquery.txt")
            ]
        ),
        .testTarget(name: "ShellCommandTests", dependencies: ["ShellCommand"]),
    ]
)
