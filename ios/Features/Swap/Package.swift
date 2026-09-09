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
        .package(name: "Validators", path: "../../Packages/Validators"),
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
                "Validators",
                "InfoSheet",
                .product(name: "BigInt", package: "BigInt"),
                "Style",
            ],
            path: "Sources",
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
