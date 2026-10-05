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
        .library(
            name: "FiatConnectTestKit",
            targets: ["FiatConnectTestKit"],
        ),
    ],
    dependencies: [
        .package(name: "InfoSheet", path: "../../Packages/InfoSheet"),
        .package(name: "Gemstone", path: "../../Packages/Gemstone"),
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
        .package(name: "BigInt", path: "../../Submodules/BigInt"),
    ],
    targets: [
        .target(
            name: "FiatConnect",
            dependencies: [
                "InfoSheet",
                "Gemstone",
                "Primitives",
                "Style",
                "Components",
                "Localization",
                "GemstonePrimitives",
                "Store",
                "PrimitivesComponents",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                .product(name: "BigInt", package: "BigInt"),
            ],
            path: "Sources",
        ),
        .target(
            name: "FiatConnectTestKit",
            dependencies: [
                "FiatConnect",
                "Primitives",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
            ],
            path: "TestKit",
        ),
        .testTarget(
            name: "FiatConnectTests",
            dependencies: [
                "FiatConnect",
                "FiatConnectTestKit",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                .product(name: "BigInt", package: "BigInt"),
                "Gemstone",
                "GemstonePrimitives",
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                "Localization",
                "Primitives",
                "Store",
            ],
            path: "Tests",
        ),
    ],
)
