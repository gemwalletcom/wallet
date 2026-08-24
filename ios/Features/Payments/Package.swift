// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Payments",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "Payments",
            targets: ["Payments"],
        ),
    ],
    dependencies: [
        .package(name: "BigInt", path: "../../Submodules/BigInt"),
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "FeatureServices", path: "../../Packages/FeatureServices"),
        .package(name: "Formatters", path: "../../Packages/Formatters"),
        .package(name: "InfoSheet", path: "../InfoSheet"),
    ],
    targets: [
        .target(
            name: "Payments",
            dependencies: [
                .product(name: "BigInt", package: "BigInt"),
                "Primitives",
                "Style",
                "Components",
                "Localization",
                "GemstonePrimitives",
                "PrimitivesComponents",
                "Formatters",
                "InfoSheet",
                .product(name: "PaymentService", package: "FeatureServices"),
                .product(name: "BalanceService", package: "FeatureServices"),
            ],
            path: "Sources",
        ),
        .testTarget(
            name: "PaymentsTests",
            dependencies: [
                "Payments",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "BalanceServiceTestKit", package: "FeatureServices"),
            ],
            path: "Tests",
        ),
    ],
)
