// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Support",
    platforms: [.iOS(.v17)],
    products: [
        .library(
            name: "Support",
            targets: ["Support"],
        ),
    ],
    dependencies: [
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "Gemstone", path: "../../Packages/Gemstone"),
    ],
    targets: [
        .target(
            name: "Support",
            dependencies: [
                "Style",
                "Components",
                "Primitives",
                "Localization",
                "PrimitivesComponents",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                "Store",
                "Gemstone",
            ],
            path: "Sources",
        ),
        .testTarget(
            name: "SupportTests",
            dependencies: [
                "Support",
                "Primitives",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
            ],
        ),
    ],
)
