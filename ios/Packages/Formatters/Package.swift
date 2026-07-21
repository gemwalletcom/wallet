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
                "GemstonePrimitives",
            ],
        ),
        .testTarget(
            name: "FormattersTests",
            dependencies: [
                "Formatters",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
            ],
        ),
    ],
)
