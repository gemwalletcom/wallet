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
        .library(
            name: "OnboardingTestKit",
            targets: ["OnboardingTestKit"],
        ),
    ],
    dependencies: [
        .package(name: "Gemstone", path: "../../Packages/Gemstone"),
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
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
                "Gemstone",
                "Primitives",
                "GemstonePrimitives",
                "Components",
                "InfoSheet",
                "Style",
                "Localization",
                "PrimitivesComponents",
                "QRScanner",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
            ],
            path: "Sources",
        ),
        .target(
            name: "OnboardingTestKit",
            dependencies: [
                "Onboarding",
                "Gemstone",
                "Primitives",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
            ],
            path: "TestKit",
        ),
        .testTarget(
            name: "OnboardingTest",
            dependencies: [
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                "Onboarding",
                "OnboardingTestKit",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                .product(name: "StoreTestKit", package: "Store"),
                "Gemstone",
                "GemstonePrimitives",
                "Primitives",
                "Store",
                "Components",
                "Localization",
                "PrimitivesComponents",
            ],
            path: "Tests",
        ),
    ],
)
