// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "MarketInsight",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "MarketInsight",
            targets: ["MarketInsight"],
        ),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Formatters", path: "../../Packages/Formatters"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "Gemstone", path: "../../Packages/Gemstone"),
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
        .package(name: "InfoSheet", path: "../InfoSheet"),
    ],
    targets: [
        .target(
            name: "MarketInsight",
            dependencies: [
                "Gemstone",
                "Primitives",
                "Formatters",
                "GemstonePrimitives",
                "Localization",
                "Store",
                "PrimitivesComponents",
                "Components",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                "InfoSheet",
            ],
            path: "Sources",
        ),
        .testTarget(
            name: "MarketInsightTests",
            dependencies: [
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                "MarketInsight",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
            ],
        ),
    ],
)
