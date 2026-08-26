// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Assets",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "Assets",
            targets: ["Assets"],
        ),
        .library(
            name: "AssetsTestKit",
            targets: ["AssetsTestKit"],
        ),
    ],
    dependencies: [
        .package(name: "Gemstone", path: "../../Packages/Gemstone"),
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Formatters", path: "../../Packages/Formatters"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "Preferences", path: "../../Packages/Preferences"),
        .package(name: "Blockchain", path: "../../Packages/Blockchain"),
        .package(name: "InfoSheet", path: "../InfoSheet"),
        .package(name: "QRScanner", path: "../QRScanner"),
        .package(name: "Recents", path: "../Recents"),
        .package(name: "ChainServices", path: "../../Packages/ChainServices"),
        .package(name: "FeatureServices", path: "../../Packages/FeatureServices"),
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
        .package(name: "GemAPI", path: "../../Packages/GemAPI"),
    ],
    targets: [
        .target(
            name: "Assets",
            dependencies: [
                "Gemstone",
                "Primitives",
                "Formatters",
                "Localization",
                "Style",
                "Components",
                "PrimitivesComponents",
                "GemstonePrimitives",
                "Store",
                "Preferences",
                "Blockchain",
                "InfoSheet",
                "QRScanner",
                "Recents",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                .product(name: "ChainService", package: "ChainServices"),
                .product(name: "ActivityService", package: "FeatureServices"),
            ],
            path: "Sources",
        ),
        .target(
            name: "AssetsTestKit",
            dependencies: [
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                .product(name: "ActivityServiceTestKit", package: "FeatureServices"),
                "Components",
                "Assets",
            ],
            path: "TestKit",
        ),
        .testTarget(
            name: "AssetsTests",
            dependencies: [
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                .product(name: "GemAPITestKit", package: "GemAPI"),
                .product(name: "StoreTestKit", package: "Store"),
                "Store",
                "AssetsTestKit",
            ],
        ),
    ],
)
