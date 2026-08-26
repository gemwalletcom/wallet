// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Transfer",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "Transfer",
            targets: ["Transfer"],
        ),
        .library(
            name: "TransferTestKit",
            targets: ["TransferTestKit"],
        ),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Gemstone", path: "../../Packages/Gemstone"),
        .package(name: "Formatters", path: "../../Packages/Formatters"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "Keystore", path: "../../Packages/Keystore"),
        .package(name: "Preferences", path: "../../Packages/Preferences"),
        .package(name: "Validators", path: "../../Packages/Validators"),
        .package(name: "Store", path: "../../Packages/Store"),

        .package(name: "Stake", path: "../Stake"),
        .package(name: "WalletConnector", path: "../WalletConnector"),
        .package(name: "InfoSheet", path: "../InfoSheet"),
        .package(name: "Swap", path: "../Swap"),
        .package(name: "Perpetuals", path: "../Perpetuals"),

        .package(name: "FeatureServices", path: "../../Packages/FeatureServices"),
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
        .package(name: "GemAPI", path: "../../Packages/GemAPI"),
    ],
    targets: [
        .target(
            name: "Transfer",
            dependencies: [
                "Gemstone",
                "Primitives",
                "Formatters",
                .product(name: "GemstoneFormatters", package: "Formatters"),
                "Localization",
                "Style",
                "Components",
                "PrimitivesComponents",
                "GemstonePrimitives",
                "Keystore",
                "Preferences",
                "Store",
                "Validators",

                "Stake",
                "WalletConnector",
                "InfoSheet",
                "Swap",
                "Perpetuals",

                .product(name: "WalletSessionService", package: "FeatureServices"),
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                .product(name: "ActivityService", package: "FeatureServices"),
            ],
            path: "Sources",
        ),
        .target(
            name: "TransferTestKit",
            dependencies: [
                "Transfer",
                "Primitives",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
            ],
            path: "TestKit",
        ),
        .testTarget(
            name: "TransferTests",
            dependencies: [
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                "Transfer",
                "TransferTestKit",
                "Gemstone",
                "GemstonePrimitives",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "SwapServiceTestKit", package: "FeatureServices"),
                .product(name: "KeystoreTestKit", package: "Keystore"),
                .product(name: "WalletSessionService", package: "FeatureServices"),
                .product(name: "WalletSessionServiceTestKit", package: "FeatureServices"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                .product(name: "ActivityServiceTestKit", package: "FeatureServices"),
                .product(name: "GemAPITestKit", package: "GemAPI"),
                .product(name: "StoreTestKit", package: "Store"),
            ],
            path: "Tests",
        ),
    ],
)
