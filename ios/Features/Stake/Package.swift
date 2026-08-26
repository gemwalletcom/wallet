// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Stake",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "Stake",
            targets: ["Stake"],
        ),
        .library(
            name: "StakeTestKit",
            targets: ["StakeTestKit"],
        ),
    ],
    dependencies: [
        .package(name: "Gemstone", path: "../../Packages/Gemstone"),
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
        .package(name: "Preferences", path: "../../Packages/Preferences"),
        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "InfoSheet", path: "../InfoSheet"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "Formatters", path: "../../Packages/Formatters"),
        .package(name: "Style", path: "../../Packages/Style"),
    ],
    targets: [
        .target(
            name: "Stake",
            dependencies: [
                "Gemstone",
                "Primitives",
                "Components",
                "GemstonePrimitives",
                "Localization",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                "Preferences",
                "Store",
                "InfoSheet",
                "PrimitivesComponents",
                "Formatters",
                "Style",
            ],
            path: "Sources",
        ),
        .target(
            name: "StakeTestKit",
            dependencies: [
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                "Stake",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
            ],
            path: "TestKit",
        ),
        .testTarget(
            name: "StakeTests",
            dependencies: [
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                "StakeTestKit",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                "Stake",
            ],
            path: "Tests",
        ),
    ],
)
