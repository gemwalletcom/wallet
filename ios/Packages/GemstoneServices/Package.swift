// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "GemstoneServices",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "GemstoneServices",
            targets: ["GemstoneServices"],
        ),
        .library(
            name: "GemstoneServicesTestKit",
            targets: ["GemstoneServicesTestKit"],
        ),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../Primitives"),
        .package(name: "Gemstone", path: "../Gemstone"),
        .package(name: "GemstonePrimitives", path: "../GemstonePrimitives"),
        .package(name: "GemstoneStore", path: "../GemstoneStore"),
        .package(name: "Store", path: "../Store"),
        .package(name: "Preferences", path: "../Preferences"),
    ],
    targets: [
        .target(
            name: "GemstoneServices",
            dependencies: [
                "Primitives",
                "Gemstone",
                "GemstonePrimitives",
                "GemstoneStore",
                "Store",
                "Preferences",
            ],
            path: "Sources",
        ),
        .target(
            name: "GemstoneServicesTestKit",
            dependencies: [
                "GemstoneServices",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "PreferencesTestKit", package: "Preferences"),
            ],
            path: "TestKit",
        ),
    ],
)
