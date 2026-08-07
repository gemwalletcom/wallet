// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Formatters",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "Formatters",
            targets: ["Formatters"],
        ),
        .library(
            name: "GemstoneFormatters",
            targets: ["GemstoneFormatters"],
        ),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../Primitives"),
        .package(name: "GemstonePrimitives", path: "../GemstonePrimitives"),
    ],
    targets: [
        .target(
            name: "Formatters",
            dependencies: [
                "Primitives",
            ],
            path: "Sources",
            exclude: ["GemstoneFormatters"],
        ),
        .target(
            name: "GemstoneFormatters",
            dependencies: [
                "Formatters",
                "Primitives",
                "GemstonePrimitives",
            ],
            path: "Sources/GemstoneFormatters",
        ),
        .testTarget(
            name: "FormattersTests",
            dependencies: [
                "Formatters",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
            ],
        ),
        .testTarget(
            name: "GemstoneFormattersTests",
            dependencies: [
                "GemstoneFormatters",
                "Formatters",
                "Primitives",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
            ],
        ),
    ],
)
