// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Settings",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "Settings",
            targets: ["Settings"],
        ),
    ],
    dependencies: [
        .package(name: "Gemstone", path: "../../Packages/Gemstone"),
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "Preferences", path: "../../Packages/Preferences"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "Keystore", path: "../../Packages/Keystore"),
        .package(name: "FeatureServices", path: "../../Packages/FeatureServices"),
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
        .package(name: "Formatters", path: "../../Packages/Formatters"),
        .package(name: "Validators", path: "../../Packages/Validators"),
        .package(name: "QRScanner", path: "../QRScanner"),
    ],
    targets: [
        .target(
            name: "Settings",
            dependencies: [
                "Gemstone",
                "Primitives",
                "Components",
                "Style",
                "Localization",
                "PrimitivesComponents",
                "Preferences",
                "GemstonePrimitives",
                "Keystore",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                .product(name: "AppService", package: "FeatureServices"),
                "Formatters",
                "Validators",
                "QRScanner",
            ],
            path: "Sources",
        ),
        .testTarget(
            name: "CurrencyTests",
            dependencies: [
                "Settings",
                "Primitives",
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
            ],
        ),
    ],
)
