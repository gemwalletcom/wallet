// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "GemstoneStore",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "GemstoneStore",
            targets: ["GemstoneStore"],
        ),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../Primitives"),
        .package(name: "Gemstone", path: "../Gemstone"),
        .package(name: "GemstonePrimitives", path: "../GemstonePrimitives"),
        .package(name: "Store", path: "../Store"),
        .package(name: "Preferences", path: "../Preferences"),
        .package(name: "Keychain", path: "../Keychain"),
        .package(name: "Formatters", path: "../Formatters"),
    ],
    targets: [
        .target(
            name: "GemstoneStore",
            dependencies: [
                "Primitives",
                "Gemstone",
                "GemstonePrimitives",
                "Store",
                "Preferences",
                "Keychain",
                "Formatters",
            ],
            path: "Sources",
        ),
    ],
)
