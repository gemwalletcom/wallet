// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "AppLock",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "AppLock",
            targets: ["AppLock"],
        ),
    ],
    dependencies: [
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "Gemstone", path: "../../Packages/Gemstone"),
        .package(name: "Primitives", path: "../../Packages/Primitives"),
    ],
    targets: [
        .target(
            name: "AppLock",
            dependencies: [
                "Style",
                "Components",
                "Localization",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                "Gemstone",
                "Primitives",
            ],
            path: "Sources",
        ),
        .testTarget(
            name: "AppLockTests",
            dependencies: [
                "AppLock",
                "Gemstone",
                "GemstoneServices",
                "Primitives",
            ],
        ),
    ],
)
