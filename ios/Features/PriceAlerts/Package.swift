// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "PriceAlerts",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "PriceAlerts",
            targets: ["PriceAlerts"],
        ),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "Gemstone", path: "../../Packages/Gemstone"),

        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
        .package(name: "Formatters", path: "../../Packages/Formatters"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
    ],
    targets: [
        .target(
            name: "PriceAlerts",
            dependencies: [
                "Primitives",
                "Components",
                "Style",
                "Localization",
                "PrimitivesComponents",
                "Gemstone",
                "Store",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                "Formatters",
            ],
            path: "Sources",
        ),
        .testTarget(
            name: "PriceAlertsTests",
            dependencies: [
                "PriceAlerts",
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                "Gemstone",
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                "Primitives",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                "Store",
            ],
        ),
    ],
)
