// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "ManageWallets",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "ManageWallets",
            targets: ["ManageWallets"],
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
        .package(name: "FeatureServices", path: "../../Packages/FeatureServices"),
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
        .package(name: "Onboarding", path: "../Onboarding"),
    ],
    targets: [
        .target(
            name: "ManageWallets",
            dependencies: [
                "GemstonePrimitives",
                "Gemstone",
                "Primitives",
                "Localization",
                "Style",
                "Components",
                "PrimitivesComponents",
                "Store",
                .product(name: "WalletService", package: "FeatureServices"),
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                "Onboarding",
            ],
            path: "Sources",
        ),
        .testTarget(
            name: "ManageWalletsTests",
            dependencies: [
                "ManageWallets",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "WalletServiceTestKit", package: "FeatureServices"),
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                .product(name: "StoreTestKit", package: "Store"),
            ],
        ),
    ],
)
