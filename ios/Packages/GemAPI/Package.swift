// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "GemAPI",
    platforms: [.iOS(.v17), .macOS(.v15)],
    products: [
        .library(
            name: "GemAPI",
            targets: ["GemAPI"],
        ),
        .library(
            name: "GemAPIDevice",
            targets: ["GemAPIDevice"],
        ),
        .library(
            name: "GemAPITestKit",
            targets: ["GemAPITestKit"],
        ),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../Primitives"),
        .package(name: "SwiftHTTPClient", path: "../SwiftHTTPClient"),
        .package(name: "GemstonePrimitives", path: "../GemstonePrimitives"),
    ],
    targets: [
        .target(
            name: "GemAPI",
            dependencies: [
                "Primitives",
                "SwiftHTTPClient",
            ],
            path: "Sources",
        ),
        .target(
            name: "GemAPIDevice",
            dependencies: [
                "Primitives",
                "GemstonePrimitives",
            ],
            path: "GemAPIDevice",
        ),
        .target(
            name: "GemAPITestKit",
            dependencies: [
                "GemAPI",
                "SwiftHTTPClient",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
            ],
            path: "TestKit",
        ),
        .testTarget(
            name: "GemAPITests",
            dependencies: [
                "GemAPI",
                "GemAPIDevice",
                "Primitives",
                "SwiftHTTPClient",
            ],
        ),
    ],
)
