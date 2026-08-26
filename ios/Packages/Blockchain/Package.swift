// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Blockchain",
    platforms: [.iOS(.v17), .macOS(.v15)],
    products: [
        .library(
            name: "Blockchain",
            targets: ["Blockchain"],
        ),
        .library(
            name: "BlockchainTestKit",
            targets: ["BlockchainTestKit"],
        ),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../Primitives"),
        .package(name: "Gemstone", path: "../Gemstone"),
        .package(name: "GemstonePrimitives", path: "../GemstonePrimitives"),
        .package(name: "Keychain", path: "../Keychain"),
        .package(name: "GemstoneStore", path: "../GemstoneStore"),
        .package(name: "NativeProviderService", path: "../NativeProviderService"),
    ],
    targets: [
        .target(
            name: "Blockchain",
            dependencies: [
                "Primitives",
                "Gemstone",
                "GemstonePrimitives",
                "Keychain",
                "NativeProviderService",
                "GemstoneStore",
            ],
            path: "Sources",
        ),
        .target(
            name: "BlockchainTestKit",
            dependencies: [
                "Blockchain",
                "Primitives",
                "Gemstone",
                "GemstonePrimitives",
                "NativeProviderService",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
            ],
            path: "TestKit",
        ),
        .testTarget(
            name: "BlockchainTests",
            dependencies: [
                "Blockchain",
                "BlockchainTestKit",
                "GemstonePrimitives",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
            ],
        ),
    ],
)
