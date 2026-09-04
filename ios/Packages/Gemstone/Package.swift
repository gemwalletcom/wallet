// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Gemstone",
    platforms: [
        .iOS(.v17), .macOS(.v15)
    ],
    products: [
        .library(
            name: "Gemstone",
            targets: ["Gemstone"]
        )
    ],
    dependencies: [
        .package(name: "BigInt", path: "../../Submodules/BigInt"),
    ],
    targets: [
        .target(
            name: "Gemstone",
            dependencies: [
                "GemstoneFFI",
                .product(name: "BigInt", package: "BigInt"),
            ],
            swiftSettings: [
                .swiftLanguageMode(.v5) // TODO: - remove when GemstoneFFI will support swift6 fully
            ]
        ),
        .target(
            name: "GemstoneFFI",
            path: "Sources/GemstoneFFI",
            publicHeadersPath: "include"
        )
    ]
)
