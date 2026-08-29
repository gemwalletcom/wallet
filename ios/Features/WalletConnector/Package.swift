// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "WalletConnector",
    platforms: [.iOS(.v17),
                .macOS(.v15)],
    products: [
        .library(
            name: "WalletConnector",
            targets: ["WalletConnector"],
        ),
    ],
    dependencies: [
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "Preferences", path: "../../Packages/Preferences"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "QRScanner", path: "../QRScanner"),
        .package(name: "FeatureServices", path: "../../Packages/FeatureServices"),
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
        .package(name: "Gemstone", path: "../../Packages/Gemstone"),
        .package(name: "Formatters", path: "../../Packages/Formatters"),
    ],
    targets: [
        .target(
            name: "WalletConnector",
            dependencies: [
                "GemstonePrimitives",
                "Primitives",
                .product(name: "WalletConnectorService", package: "FeatureServices"),
                "Components",
                "Localization",
                "Style",
                "Store",
                "Preferences",
                "PrimitivesComponents",
                "QRScanner",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                "Gemstone",
                "Formatters",
            ],
            path: "Sources",
        ),
        .testTarget(
            name: "WalletConnectorTests",
            dependencies: [
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "PreferencesTestKit", package: "Preferences"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                .product(name: "WalletConnectorService", package: "FeatureServices"),
                .product(name: "WalletConnectorServiceTestKit", package: "FeatureServices"),
                "WalletConnector",
                "Gemstone",
            ],
            resources: [.process("Resources")],
        ),
    ],
)
