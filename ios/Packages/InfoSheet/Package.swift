// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "InfoSheet",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "InfoSheet",
            targets: ["InfoSheet"],
        ),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../Primitives"),
        .package(name: "Style", path: "../Style"),
        .package(name: "Localization", path: "../Localization"),
        .package(name: "Components", path: "../Components"),
        .package(name: "Formatters", path: "../Formatters"),
        .package(name: "Gemstone", path: "../Gemstone"),
        .package(name: "GemstonePrimitives", path: "../GemstonePrimitives"),
        .package(name: "PrimitivesComponents", path: "../PrimitivesComponents"),
        .package(name: "BigInt", path: "../../Submodules/BigInt"),
    ],
    targets: [
        .target(
            name: "InfoSheet",
            dependencies: [
                "Primitives",
                "Style",
                "Localization",
                "Components",
                "Formatters",
                "Gemstone",
                "GemstonePrimitives",
                "PrimitivesComponents",
                .product(name: "BigInt", package: "BigInt"),
            ],
            path: "Sources",
        ),
    ],
)
