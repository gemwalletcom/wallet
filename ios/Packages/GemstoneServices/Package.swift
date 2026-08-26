// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "GemstoneServices",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "GemstoneServices",
            targets: ["GemstoneServices"],
        ),
        .library(
            name: "GemstoneServicesTestKit",
            targets: ["GemstoneServicesTestKit"],
        ),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../Primitives"),
        .package(name: "ChainServices", path: "../ChainServices"),
        .package(name: "Blockchain", path: "../Blockchain"),
        .package(name: "SystemServices", path: "../SystemServices"),
        .package(name: "NativeProviderService", path: "../NativeProviderService"),
        .package(name: "Keystore", path: "../Keystore"),
        .package(name: "GemAPI", path: "../GemAPI"),
        .package(name: "Gemstone", path: "../Gemstone"),
        .package(name: "GemstonePrimitives", path: "../GemstonePrimitives"),
        .package(name: "GemstoneStore", path: "../GemstoneStore"),
        .package(name: "Store", path: "../Store"),
        .package(name: "Preferences", path: "../Preferences"),
    ],
    targets: [
        .target(
            name: "GemstoneServices",
            dependencies: [
                "Primitives",
                "Gemstone",
                "GemstonePrimitives",
                "GemstoneStore",
                "Store",
                "Preferences",
                .product(name: "LocalStore", package: "SystemServices"),
                "Keystore",
                .product(name: "ChainService", package: "ChainServices"),
            ],
            path: "Sources",
        ),
        .target(
            name: "GemstoneServicesTestKit",
            dependencies: [
                "GemstoneServices",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "PreferencesTestKit", package: "Preferences"),
                .product(name: "GemAPITestKit", package: "GemAPI"),
                .product(name: "NativeProviderService", package: "NativeProviderService"),
                "Gemstone",
                "GemstoneStore",
                .product(name: "ChainServiceTestKit", package: "ChainServices"),
            ],
            path: "TestKit",
        ),
        .testTarget(
            name: "GemstoneServicesTests",
            dependencies: [
                "GemstoneServices",
                "GemstoneServicesTestKit",
                .product(name: "GemAPITestKit", package: "GemAPI"),
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                .product(name: "PreferencesTestKit", package: "Preferences"),
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "StoreTestKit", package: "Store"),
                "Primitives",
                .product(name: "BlockchainTestKit", package: "Blockchain"),
                .product(name: "ChainServiceTestKit", package: "ChainServices"),
                .product(name: "Store", package: "Store"),
            ],
            path: "Tests",
        ),
    ],
)
