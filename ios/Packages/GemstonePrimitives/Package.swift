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
    ],
    targets: [
        .target(
            name: "GemstonePrimitives",
            dependencies: [
                "Gemstone",
                "Primitives",
            ],
            path: "Sources",
        ),
        .target(
            name: "GemstonePrimitivesTestKit",
            dependencies: [
                "Gemstone",
                "GemstonePrimitives",
                "Primitives",
            ],
            path: "TestKit",
        ),
        .testTarget(
            name: "GemstonePrimitivesTests",
            dependencies: [
                "GemstonePrimitives",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
            ],
        ),
    ],
)
