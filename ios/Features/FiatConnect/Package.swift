// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "FiatConnect",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "FiatConnect",
            targets: ["FiatConnect"],
        ),
    ],
    dependencies: [
        .package(name: "Gemstone", path: "../../Packages/Gemstone"),
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Formatters", path: "../../Packages/Formatters"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
        .package(name: "BigInt", path: "../../Submodules/BigInt"),
        .package(name: "Validators", path: "../../Packages/Validators"),
    ],
    targets: [
        .target(
            name: "FiatConnect",
            dependencies: [
                "Gemstone",
                "Primitives",
                "Formatters",
                "Style",
                "Components",
                "Localization",
                "GemstonePrimitives",
                "Store",
                "PrimitivesComponents",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                .product(name: "BigInt", package: "BigInt"),
                "Validators",
            ],
            path: "Sources",
        ),
        .testTarget(
            name: "FiatConnectTests",
            dependencies: [
                "FiatConnect",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                .product(name: "BigInt", package: "BigInt"),
                "Formatters",
                "Gemstone",
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                "Localization",
                "Primitives",
                "Store",
            ],
            path: "Tests",
        ),
    ],
)
