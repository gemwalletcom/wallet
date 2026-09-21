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
        .library(
            name: "WalletConnectorTestKit",
            targets: ["WalletConnectorTestKit"],
        ),
    ],
    dependencies: [
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "QRScanner", path: "../QRScanner"),
        .package(name: "FeatureServices", path: "../../Packages/FeatureServices"),
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
        .package(name: "Gemstone", path: "../../Packages/Gemstone"),
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
                "PrimitivesComponents",
                "QRScanner",
                "Gemstone",
            ],
            path: "Sources",
        ),
        .target(
            name: "WalletConnectorTestKit",
            dependencies: [
                "WalletConnector",
                "Gemstone",
                "Primitives",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                .product(name: "WalletConnectorService", package: "FeatureServices"),
                .product(name: "WalletConnectorServiceTestKit", package: "FeatureServices"),
            ],
            path: "TestKit",
        ),
        .testTarget(
            name: "WalletConnectorTests",
            dependencies: [
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "WalletConnectorService", package: "FeatureServices"),
                .product(name: "WalletConnectorServiceTestKit", package: "FeatureServices"),
                "WalletConnector",
                "WalletConnectorTestKit",
                "Gemstone",
                "GemstonePrimitives",
                "Primitives",
                "PrimitivesComponents",
                "Store",
            ],
            resources: [.process("Resources")],
        ),
    ],
)
