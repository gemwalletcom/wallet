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
        .package(name: "Gemstone", path: "../../Packages/Gemstone"),
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "InfoSheet", path: "../InfoSheet"),

        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "Preferences", path: "../../Packages/Preferences"),
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
        .package(name: "Perpetuals", path: "../Perpetuals"),
        .package(name: "Recents", path: "../Recents"),
        .package(name: "NFT", path: "../NFT"),
    ],
    targets: [
        .target(
            name: "WalletTab",
            dependencies: [
                "Gemstone",
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
                "Perpetuals",
                "Recents",
                "NFT",
            ],
            path: "Sources",
        ),
        .target(
            name: "WalletTabTestKit",
            dependencies: [
                "Gemstone",
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                .product(name: "PreferencesTestKit", package: "Preferences"),
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "GemstoneServices", package: "GemstoneServices"),
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
                .product(name: "StoreTestKit", package: "Store"),
                "NFT",
                "WalletTab",
                "WalletTabTestKit",
            ],
        ),
    ],
)
