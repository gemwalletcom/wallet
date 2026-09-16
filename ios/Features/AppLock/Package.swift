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
        .library(
            name: "AppLockTestKit",
            targets: ["AppLockTestKit"],
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
        .target(
            name: "AppLockTestKit",
            dependencies: [
                "AppLock",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
            ],
            path: "TestKit",
        ),
        .testTarget(
            name: "AppLockTests",
            dependencies: [
                "AppLock",
                "AppLockTestKit",
                "GemstoneServices",
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                "Primitives",
            ],
        ),
    ],
)
