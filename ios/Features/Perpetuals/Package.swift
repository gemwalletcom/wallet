// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Perpetuals",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "Perpetuals",
            targets: ["Perpetuals"],
        ),
        .library(
            name: "PerpetualsTestKit",
            targets: ["PerpetualsTestKit"],
        ),
    ],
    dependencies: [
        .package(name: "Gemstone", path: "../../Packages/Gemstone"),
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "Formatters", path: "../../Packages/Formatters"),
        .package(name: "InfoSheet", path: "../InfoSheet"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "Recents", path: "../Recents"),
    ],
    targets: [
        .target(
            name: "Perpetuals",
            dependencies: [
                "Gemstone",
                "Primitives",
                "PrimitivesComponents",
                "GemstonePrimitives",
                "Components",
                "Style",
                "Localization",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                "Store",
                "Formatters",
                "InfoSheet",
                "Recents",
            ],
            path: "Sources",
        ),
        .target(
            name: "PerpetualsTestKit",
            dependencies: [
                "Perpetuals",
                "Primitives",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
            ],
            path: "TestKit",
        ),
        .testTarget(
            name: "PerpetualsTests",
            dependencies: [
                .product(name: "StoreTestKit", package: "Store"),
                "Store",
                "Perpetuals",
                "PerpetualsTestKit",
                "Formatters",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
            ],
            path: "Tests",
        ),
    ],
    swiftLanguageModes: [.v6],
)
