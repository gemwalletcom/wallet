// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Signer",
    platforms: [.iOS(.v17), .macOS(.v15)],
    products: [
        .library(
            name: "Signer",
            targets: ["Signer"],
        ),
        .library(
            name: "SignerTestKit",
            targets: ["SignerTestKit"],
        ),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../Primitives"),
        .package(name: "Keystore", path: "../Keystore"),
        .package(name: "Gemstone", path: "../Gemstone"),
        .package(name: "GemstonePrimitives", path: "../GemstonePrimitives"),
    ],
    targets: [
        .target(
            name: "Signer",
            dependencies: [
                "Primitives",
                "Keystore",
                "Gemstone",
                "GemstonePrimitives",
            ],
            path: "Sources",
        ),
        .target(
            name: "SignerTestKit",
            dependencies: [
                "Signer",
                "Primitives",
            ],
            path: "TestKit",
        ),
    ],
)
