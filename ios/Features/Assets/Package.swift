// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Assets",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "Assets",
            targets: ["Assets"],
        ),
        .library(
            name: "AssetsTestKit",
            targets: ["AssetsTestKit"],
        ),
    ],
    dependencies: [
        .package(name: "Gemstone", path: "../../Packages/Gemstone"),
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Formatters", path: "../../Packages/Formatters"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "Preferences", path: "../../Packages/Preferences"),
        .package(name: "InfoSheet", path: "../InfoSheet"),
        .package(name: "QRScanner", path: "../QRScanner"),
        .package(name: "Recents", path: "../Recents"),
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
    ],
    targets: [
        .target(
            name: "Assets",
            dependencies: [
                "Gemstone",
                "Primitives",
                "Formatters",
                "Localization",
                "Style",
                "Components",
                "PrimitivesComponents",
                "GemstonePrimitives",
                "Store",
                "Preferences",
                "InfoSheet",
                "QRScanner",
                "Recents",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
            ],
            path: "Sources",
        ),
        .target(
            name: "AssetsTestKit",
            dependencies: [
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                "Components",
                "Assets",
            ],
            path: "TestKit",
        ),
        .testTarget(
            name: "AssetsTests",
            dependencies: [
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                .product(name: "PreferencesTestKit", package: "Preferences"),
                .product(name: "StoreTestKit", package: "Store"),
                "Store",
                "AssetsTestKit",
            ],
        ),
    ],
)
