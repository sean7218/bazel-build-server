// swift-tools-version: 6.1
import PackageDescription

let package = Package(
    name: "BazelSourceKitBSP",
    platforms: [
        .macOS(.v13),
    ],
    products: [
        .executable(
            name: "bazel-sourcekit-bsp",
            targets: ["BazelSourceKitBSP"]
        ),
        .library(
            name: "BazelSourceKitBSPLib",
            targets: ["BazelSourceKitBSPLib"]
        ),
    ],
    dependencies: [
        .package(url: "https://github.com/apple/swift-log.git", from: "1.0.0"),
        .package(url: "https://github.com/apple/swift-nio.git", from: "2.0.0"),
        .package(url: "https://github.com/apple/swift-argument-parser.git", from: "1.0.0"),
        .package(url: "https://github.com/JohnSundell/ShellOut.git", from: "2.0.0"),
        .package(url: "https://github.com/apple/swift-system.git", from: "1.0.0"),
    ],
    targets: [
        .executableTarget(
            name: "BazelSourceKitBSP",
            dependencies: [
                "BazelSourceKitBSPLib",
                .product(name: "ArgumentParser", package: "swift-argument-parser"),
                .product(name: "Logging", package: "swift-log"),
            ]
        ),

        .target(
            name: "BazelSourceKitBSPLib",
            dependencies: [
                .product(name: "Logging", package: "swift-log"),
                .product(name: "NIO", package: "swift-nio"),
                .product(name: "NIOHTTP1", package: "swift-nio"),
                .product(name: "NIOFoundationCompat", package: "swift-nio"),
                .product(name: "ShellOut", package: "ShellOut"),
                .product(name: "SystemPackage", package: "swift-system"),
            ]
        ),
    ]
)
