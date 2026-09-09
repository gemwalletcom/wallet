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
        .package(name: "BigInt", path: "../../Submodules/BigInt"),
    ],
    targets: [
        .target(
            name: "Validators",
            dependencies: [
                "Primitives",
                "Gemstone",
                "Localization",
                "Formatters",
                .product(name: "BigInt", package: "BigInt"),
            ],
            path: "Sources",
        ),
        .testTarget(
            name: "ValidatorsTests",
            dependencies: [
                "Validators",
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "BigInt", package: "BigInt"),
                "Formatters",
                "Gemstone",
                "Primitives",
            ],
        ),
    ],
)
