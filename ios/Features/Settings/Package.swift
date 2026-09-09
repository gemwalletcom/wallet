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
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
        .package(name: "Formatters", path: "../../Packages/Formatters"),
        .package(name: "QRScanner", path: "../QRScanner"),
        .package(name: "BigInt", path: "../../Submodules/BigInt"),
        .package(name: "Store", path: "../../Packages/Store"),
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
                "GemstonePrimitives",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                "Formatters",
                "QRScanner",
                .product(name: "BigInt", package: "BigInt"),
                "Store",
            ],
            path: "Sources",
        ),
        .testTarget(
            name: "CurrencyTests",
            dependencies: [
                "Settings",
                "Primitives",
                "GemstonePrimitives",
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                "GemstoneServices",
            ],
        ),
    ],
)
