// swift-tools-version: 6.0
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "PrimitivesComponents",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "PrimitivesComponents",
            targets: ["PrimitivesComponents"],
        ),
        .library(
            name: "PrimitivesComponentsTestKit",
            targets: ["PrimitivesComponentsTestKit"],
        ),
    ],
    dependencies: [
        .package(name: "Gemstone", path: "../Gemstone"),
        .package(name: "Primitives", path: "../Primitives"),
        .package(name: "GemstonePrimitives", path: "../GemstonePrimitives"),
        .package(name: "Localization", path: "../Localization"),
        .package(name: "Components", path: "../Components"),
        .package(name: "Style", path: "../Style"),
        .package(name: "Validators", path: "../Validators"),
        .package(name: "Formatters", path: "../Formatters"),
        .package(name: "BigInt", path: "../../Submodules/BigInt"),
    ],
    targets: [
        .target(
            name: "PrimitivesComponents",
            dependencies: [
                "Gemstone",
                "Primitives",
                "GemstonePrimitives",
                "Localization",
                "Components",
                "Style",
                "Validators",
                "Formatters",
                .product(name: "BigInt", package: "BigInt"),
                .product(name: "PrimitivesTestKit", package: "Primitives"),
            ],
            path: "Sources",
        ),
        .target(
            name: "PrimitivesComponentsTestKit",
            dependencies: [
                "PrimitivesComponents",
                "Formatters",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
            ],
            path: "TestKit",
        ),
        .testTarget(
            name: "PrimitivesComponentsTests",
            dependencies: [
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                "PrimitivesComponents",
                "PrimitivesComponentsTestKit",
                "GemstonePrimitives",
                "Formatters",
            ],
        ),
    ],
)
