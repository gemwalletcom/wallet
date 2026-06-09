// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Support",
    platforms: [.iOS(.v17)],
    products: [
        .library(
            name: "Support",
            targets: ["Support"],
        ),
    ],
    dependencies: [
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "Preferences", path: "../../Packages/Preferences"),
        .package(name: "FeatureServices", path: "../../Packages/FeatureServices"),
        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "GemAPI", path: "../../Packages/GemAPI"),
    ],
    targets: [
        .target(
            name: "Support",
            dependencies: [
                "Style",
                "Components",
                "Primitives",
                "Localization",
                "GemstonePrimitives",
                "PrimitivesComponents",
                "Preferences",
                .product(name: "NotificationService", package: "FeatureServices"),
                .product(name: "DeviceService", package: "FeatureServices"),
                .product(name: "SupportChatService", package: "FeatureServices"),
                "Store",
                .product(name: "GemAPI", package: "GemAPI"),
            ],
            path: "Sources",
        ),
        .testTarget(
            name: "SupportTests",
            dependencies: [
                "Support",
                "Primitives",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                "Store",
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "SupportChatService", package: "FeatureServices"),
            ],
        ),
    ],
)
