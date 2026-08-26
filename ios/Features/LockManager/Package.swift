// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "LockManager",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "LockManager",
            targets: ["LockManager"],
        ),
    ],
    dependencies: [
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "Localization", path: "../../Packages/Localization"),
    ],
    targets: [
        .target(
            name: "LockManager",
            dependencies: [
                "Style",
                "Components",
                "Localization",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
            ],
            path: "Sources",
        ),
        .testTarget(
            name: "LockManagerTests",
            dependencies: ["LockManager"],
        ),
    ],
)
