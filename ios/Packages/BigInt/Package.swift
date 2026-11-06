// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "BigInt",
    platforms: [.iOS(.v17), .macOS(.v15)],
    products: [
        .library(name: "BigInt", targets: ["BigInt"]),
    ],
    targets: [
        .target(name: "BigInt", path: "Sources"),
        .testTarget(name: "BigIntTests", dependencies: ["BigInt"], path: "Tests"),
    ]
)
