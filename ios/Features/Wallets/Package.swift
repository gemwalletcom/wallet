// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Wallets",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "Wallets",
            targets: ["Wallets"],
        ),
        .library(
            name: "WalletsTestKit",
            targets: ["WalletsTestKit"],
        ),
    ],
    dependencies: [
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "Gemstone", path: "../../Packages/Gemstone"),
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
    ],
    targets: [
        .target(
            name: "Wallets",
            dependencies: [
                "GemstonePrimitives",
                "Gemstone",
                "Primitives",
                "Localization",
                "Style",
                "Components",
                "PrimitivesComponents",
                "Store",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
            ],
            path: "Sources",
        ),
        .target(
            name: "WalletsTestKit",
            dependencies: [
                "Wallets",
                "Gemstone",
                "Primitives",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
            ],
            path: "TestKit",
        ),
        .testTarget(
            name: "WalletsTests",
            dependencies: [
                "Wallets",
                "WalletsTestKit",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                .product(name: "StoreTestKit", package: "Store"),
                "Gemstone",
                "Primitives",
                "Store",
                "GemstonePrimitives",
            ],
        ),
    ],
)
