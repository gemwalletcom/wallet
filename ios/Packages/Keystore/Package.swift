// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Keystore",
    platforms: [.iOS(.v17), .macOS(.v15)],
    products: [
        .library(
            name: "Keystore",
            targets: ["Keystore"],
        ),
        .library(
            name: "KeystoreTestKit",
            targets: ["KeystoreTestKit"],
        ),
    ],
    dependencies: [
        .package(name: "Gemstone", path: "../Gemstone"),
        .package(name: "Primitives", path: "../Primitives"),
        .package(name: "Keychain", path: "../Keychain"),
        .package(name: "GemstonePrimitives", path: "../GemstonePrimitives"),
    ],
    targets: [
        .target(
            name: "Keystore",
            dependencies: [
                "Gemstone",
                "Primitives",
                "Keychain",
                "GemstonePrimitives",
            ],
            path: "Sources",
        ),
        .target(
            name: "KeystoreTestKit",
            dependencies: [
                "Keystore",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
            ],
            path: "TestKit",
        ),
        .testTarget(
            name: "KeystoreTests",
            dependencies: [
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                "KeystoreTestKit",
                "Keystore",
                "GemstonePrimitives",
            ],
            resources: [.process("Resources")],
        ),
    ],
)
