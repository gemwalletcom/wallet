// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Contacts",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "Contacts",
            targets: ["Contacts"],
        ),
    ],
    dependencies: [
        .package(name: "Gemstone", path: "../../Packages/Gemstone"),
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "Validators", path: "../../Packages/Validators"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "GemstoneServices", path: "../../Packages/GemstoneServices"),
        .package(name: "QRScanner", path: "../../Packages/QRScanner"),
    ],
    targets: [
        .target(
            name: "Contacts",
            dependencies: [
                "Gemstone",
                "Primitives",
                "PrimitivesComponents",
                "Components",
                "Localization",
                "Style",
                "Store",
                "Validators",
                "GemstonePrimitives",
                "QRScanner",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
            ],
            path: "Sources",
        ),
        .testTarget(
            name: "ContactsTests",
            dependencies: [
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                "Contacts",
                "Primitives",
                "PrimitivesComponents",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
            ],
        ),
    ],
)
