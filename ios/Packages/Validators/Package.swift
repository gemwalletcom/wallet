// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Validators",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "Validators",
            targets: ["Validators"],
        ),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../Primitives"),
        .package(name: "Gemstone", path: "../Gemstone"),
        .package(name: "GemstonePrimitives", path: "../GemstonePrimitives"),
        .package(name: "Localization", path: "../Localization"),
        .package(name: "Formatters", path: "../Formatters"),
    ],
    targets: [
        .target(
            name: "Validators",
            dependencies: [
                "Primitives",
                "Gemstone",
                "GemstonePrimitives",
                "Localization",
                "Formatters",
                .product(name: "GemstoneFormatters", package: "Formatters"),
            ],
            path: "Sources",
        ),
        .testTarget(
            name: "ValidatorsTests",
            dependencies: [
                "Validators",
                .product(name: "GemstoneFormatters", package: "Formatters"),
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                .product(name: "PrimitivesTestKit", package: "Primitives"),
            ],
        ),
    ],
)
