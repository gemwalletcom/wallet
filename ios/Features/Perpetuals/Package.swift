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
        .package(name: "Validators", path: "../../Packages/Validators"),
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
                "Validators",
            ],
            path: "Sources",
        ),
        .target(
            name: "PerpetualsTestKit",
            dependencies: [
                "Perpetuals",
                "Primitives",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                "Formatters",
                "Gemstone",
                "GemstonePrimitives",
            ],
            path: "TestKit",
        ),
        .testTarget(
            name: "PerpetualsTests",
            dependencies: [
                .product(name: "StoreTestKit", package: "Store"),
                "Perpetuals",
                "PerpetualsTestKit",
                "Formatters",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                "Components",
                "Gemstone",
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                "GemstoneServices",
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                "Localization",
                "Primitives",
                "Style",
            ],
            path: "Tests",
        ),
    ],
    swiftLanguageModes: [.v6],
)
