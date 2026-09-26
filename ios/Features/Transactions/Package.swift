// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Transactions",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "Transactions",
            targets: ["Transactions"],
        ),
        .library(
            name: "TransactionsTestKit",
            targets: ["TransactionsTestKit"],
        ),
    ],
    dependencies: [
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "Gemstone", path: "../../Packages/Gemstone"),
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "Formatters", path: "../../Packages/Formatters"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
        .package(name: "InfoSheet", path: "../../Packages/InfoSheet"),
        .package(name: "BigInt", path: "../../Submodules/BigInt"),
    ],
    targets: [
        .target(
            name: "Transactions",
            dependencies: [
                "GemstonePrimitives",
                "Gemstone",
                .product(name: "BigInt", package: "BigInt"),
                "Primitives",
                "Localization",
                "Formatters",
                "Store",
                "Style",
                "Components",
                "PrimitivesComponents",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                "InfoSheet",
            ],
            path: "Sources",
        ),
        .target(
            name: "TransactionsTestKit",
            dependencies: [
                "Transactions",
                "Gemstone",
                "Primitives",
                "Store",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
            ],
            path: "TestKit",
        ),
        .testTarget(
            name: "TransactionsTests",
            dependencies: [
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                "Transactions",
                "TransactionsTestKit",
                "PrimitivesComponents",
                .product(name: "PrimitivesComponentsTestKit", package: "PrimitivesComponents"),
                "Components",
                "Gemstone",
                "GemstonePrimitives",
                "Localization",
                "Primitives",
                "Store",
                "Style",
            ],
        ),
    ],
)
