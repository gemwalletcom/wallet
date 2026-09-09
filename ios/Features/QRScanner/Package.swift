// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "QRScanner",
    platforms: [.iOS(.v17),
                .macOS(.v15)],
    products: [
        .library(
            name: "QRScanner",
            targets: ["QRScanner"],
        ),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "Style", path: "../../Packages/Style"),
    ],
    targets: [
        .target(
            name: "QRScanner",
            dependencies: [
                "Primitives",
                "Components",
                "Localization",
                "Style",
            ],
            path: "Sources",
        ),
    ],
)
