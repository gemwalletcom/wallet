// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "GemstonePrimitives",
    platforms: [.iOS(.v17), .macOS(.v15)],
    products: [
        .library(
            name: "GemstonePrimitives",
            targets: ["GemstonePrimitives"],
        ),
        .library(
            name: "GemstonePrimitivesTestKit",
            targets: ["GemstonePrimitivesTestKit"],
        ),
    ],
    dependencies: [
        .package(name: "Gemstone", path: "../Gemstone"),
        .package(name: "Primitives", path: "../Primitives"),
        .package(name: "BigInt", path: "../../Submodules/BigInt"),
    ],
    targets: [
        .target(
            name: "GemstonePrimitives",
            dependencies: [
                "Gemstone",
                "Primitives",
                .product(name: "BigInt", package: "BigInt"),
            ],
            path: "Sources",
        ),
        .target(
            name: "GemstonePrimitivesTestKit",
            dependencies: [
                "Gemstone",
                "GemstonePrimitives",
                "Primitives",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "BigInt", package: "BigInt"),
            ],
            path: "TestKit",
        ),
        .testTarget(
            name: "GemstonePrimitivesTests",
            dependencies: [
                "GemstonePrimitives",
                "GemstonePrimitivesTestKit",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                "Gemstone",
                "Primitives",
            ],
        ),
    ],
)
