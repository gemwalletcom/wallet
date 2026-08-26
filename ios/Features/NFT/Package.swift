// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "NFT",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "NFT",
            targets: ["NFT"],
        ),
    ],
    dependencies: [
        .package(name: "Gemstone", path: "../../Packages/Gemstone"),
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "SystemServices", path: "../../Packages/SystemServices"),
        .package(name: "FeatureServices", path: "../../Packages/FeatureServices"),
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "Formatters", path: "../../Packages/Formatters"),
        .package(name: "InfoSheet", path: "../InfoSheet"),
    ],
    targets: [
        .target(
            name: "NFT",
            dependencies: [
                "Gemstone",
                "Primitives",
                "Components",
                "PrimitivesComponents",
                "Style",
                "Localization",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                "Store",
                .product(name: "ImageGalleryService", package: "SystemServices"),
                "GemstonePrimitives",
                .product(name: "AvatarService", package: "FeatureServices"),
                "Formatters",
                "InfoSheet",
            ],
            path: "Sources",
        ),
        .testTarget(
            name: "NFTTests",
            dependencies: [
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                "NFT",
                "PrimitivesComponents",
                "Formatters",
                .product(name: "AvatarService", package: "FeatureServices"),
                "Store",
            ],
        ),
    ],
)
