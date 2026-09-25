// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Rewards",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "Rewards",
            targets: ["Rewards"],
        ),
        .library(
            name: "RewardsTestKit",
            targets: ["RewardsTestKit"],
        ),
    ],
    dependencies: [
        .package(name: "Gemstone", path: "../../Packages/Gemstone"),
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
    ],
    targets: [
        .target(
            name: "Rewards",
            dependencies: [
                "Gemstone",
                "Primitives",
                "Components",
                "Style",
                "Localization",
                "PrimitivesComponents",
                "GemstonePrimitives",
            ],
            path: "Sources",
        ),
        .target(
            name: "RewardsTestKit",
            dependencies: [
                "Rewards",
                "Gemstone",
                "Primitives",
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
            ],
            path: "TestKit",
        ),
        .testTarget(
            name: "RewardsTests",
            dependencies: [
                "Rewards",
                "RewardsTestKit",
                "Primitives",
                "Gemstone",
                "Components",
                "Localization",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
            ],
        ),
    ],
)
