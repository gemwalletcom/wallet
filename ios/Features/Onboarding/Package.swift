// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Onboarding",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "Onboarding",
            targets: ["Onboarding"],
        ),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "Formatters", path: "../../Packages/Formatters"),
        .package(name: "InfoSheet", path: "../InfoSheet"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "QRScanner", path: "../QRScanner"),
        .package(name: "Keystore", path: "../../Packages/Keystore"),
        .package(name: "FeatureServices", path: "../../Packages/FeatureServices"),
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
        .package(name: "Store", path: "../../Packages/Store"),
    ],
    targets: [
        .target(
            name: "Onboarding",
            dependencies: [
                "Primitives",
                "GemstonePrimitives",
                "Components",
                "InfoSheet",
                "Style",
                "Localization",
                "PrimitivesComponents",
                "QRScanner",
                "Keystore",
                .product(name: "WalletService", package: "FeatureServices"),
                .product(name: "WalletSessionService", package: "FeatureServices"),
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                .product(name: "AvatarService", package: "FeatureServices"),
                "Formatters",
                "Store",
            ],
            path: "Sources",
        ),
        .testTarget(
            name: "OnboardingTest",
            dependencies: [
                "Onboarding",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "WalletServiceTestKit", package: "FeatureServices"),
                .product(name: "WalletSessionService", package: "FeatureServices"),
                .product(name: "WalletSessionServiceTestKit", package: "FeatureServices"),
                .product(name: "KeystoreTestKit", package: "Keystore"),
                .product(name: "StoreTestKit", package: "Store"),
            ],
            path: "Tests",
        ),
    ],
)
