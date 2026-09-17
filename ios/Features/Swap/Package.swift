// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Swap",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "Swap",
            targets: ["Swap"],
        ),
        .library(
            name: "SwapTestKit",
            targets: ["SwapTestKit"],
        ),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Formatters", path: "../../Packages/Formatters"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "Gemstone", path: "../../Packages/Gemstone"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "InfoSheet", path: "../InfoSheet"),
        .package(name: "BigInt", path: "../../Submodules/BigInt"),
        .package(name: "Style", path: "../../Packages/Style"),
    ],
    targets: [
        .target(
            name: "Swap",
            dependencies: [
                "Primitives",
                "Formatters",
                "Components",
                "GemstonePrimitives",
                "Gemstone",
                "Localization",
                "Store",
                "PrimitivesComponents",
                "InfoSheet",
                .product(name: "BigInt", package: "BigInt"),
                "Style",
            ],
            path: "Sources",
        ),
        .target(
            name: "SwapTestKit",
            dependencies: [
                "Swap",
                .product(name: "BigInt", package: "BigInt"),
                "Gemstone",
                "GemstonePrimitives",
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                "Primitives",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                "Store",
            ],
            path: "TestKit",
        ),
        .testTarget(
            name: "SwapTests",
            dependencies: [
                "GemstonePrimitives",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                .product(name: "StoreTestKit", package: "Store"),
                "PrimitivesComponents",
                "Swap",
                "SwapTestKit",
                .product(name: "BigInt", package: "BigInt"),
                "Components",
                "Formatters",
                "Gemstone",
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                "Localization",
                "Primitives",
                "Store",
                "Style",
            ],
        ),
    ],
)
