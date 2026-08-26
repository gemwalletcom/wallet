// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "WalletTab",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "WalletTab",
            targets: ["WalletTab"],
        ),
        .library(
            name: "WalletTabTestKit",
            targets: ["WalletTabTestKit"],
        ),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "InfoSheet", path: "../InfoSheet"),

        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "Preferences", path: "../../Packages/Preferences"),
        .package(name: "FeatureServices", path: "../../Packages/FeatureServices"),
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
        .package(name: "GemAPI", path: "../../Packages/GemAPI"),
        .package(name: "Perpetuals", path: "../Perpetuals"),
        .package(name: "Recents", path: "../Recents"),
        .package(name: "NFT", path: "../NFT"),
    ],
    targets: [
        .target(
            name: "WalletTab",
            dependencies: [
                "Primitives",
                "Localization",
                "Style",
                "Components",
                "PrimitivesComponents",
                "GemstonePrimitives",
                "InfoSheet",
                "Store",
                "Preferences",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                .product(name: "WalletSessionService", package: "FeatureServices"),
                .product(name: "ActivityService", package: "FeatureServices"),
                "Perpetuals",
                "Recents",
                "NFT",
            ],
            path: "Sources",
        ),
        .target(
            name: "WalletTabTestKit",
            dependencies: [
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                .product(name: "PreferencesTestKit", package: "Preferences"),
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "WalletSessionService", package: "FeatureServices"),
                .product(name: "WalletSessionServiceTestKit", package: "FeatureServices"),
                .product(name: "ActivityServiceTestKit", package: "FeatureServices"),
                "WalletTab",
            ],
            path: "TestKit",
        ),
        .testTarget(
            name: "WalletTabTests",
            dependencies: [
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                .product(name: "PreferencesTestKit", package: "Preferences"),
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "WalletSessionServiceTestKit", package: "FeatureServices"),
                .product(name: "GemAPITestKit", package: "GemAPI"),
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "ActivityServiceTestKit", package: "FeatureServices"),
                "NFT",
                "WalletTab",
                "WalletTabTestKit",
            ],
        ),
    ],
)
