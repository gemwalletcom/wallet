// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Market",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "Market",
            targets: ["Market"],
        ),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "Gemstone", path: "../../Packages/Gemstone"),
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
        .package(name: "InfoSheet", path: "../../Packages/InfoSheet"),
    ],
    targets: [
        .target(
            name: "Market",
            dependencies: [
                "Gemstone",
                "Primitives",
                "GemstonePrimitives",
                "Store",
                "PrimitivesComponents",
                "Components",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                "InfoSheet",
            ],
            path: "Sources",
        ),
    ],
)
