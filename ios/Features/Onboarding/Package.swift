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
        .package(name: "Preferences", path: "../../Packages/Preferences"),
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "Formatters", path: "../../Packages/Formatters"),
        .package(name: "InfoSheet", path: "../InfoSheet"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "QRScanner", path: "../QRScanner"),
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
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                "Formatters",
                "Store",
            ],
            path: "Sources",
        ),
        .testTarget(
            name: "OnboardingTest",
            dependencies: [
                "Preferences",
                .product(name: "PreferencesTestKit", package: "Preferences"),
                "Onboarding",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                .product(name: "StoreTestKit", package: "Store"),
            ],
            path: "Tests",
        ),
    ],
)
