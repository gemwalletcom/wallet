// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "GemstoneServices",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "GemstoneServices",
            targets: ["GemstoneServices"],
        ),
        .library(
            name: "GemstoneServicesTestKit",
            targets: ["GemstoneServicesTestKit"],
        ),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../Primitives"),
        .package(name: "SwiftHTTPClient", path: "../SwiftHTTPClient"),
        .package(name: "Formatters", path: "../Formatters"),
        .package(name: "SystemServices", path: "../SystemServices"),
        .package(name: "NativeProviderService", path: "../NativeProviderService"),
        .package(name: "Keychain", path: "../Keychain"),
        .package(name: "Gemstone", path: "../Gemstone"),
        .package(name: "GemstonePrimitives", path: "../GemstonePrimitives"),
        .package(name: "Store", path: "../Store"),
        .package(name: "Preferences", path: "../Preferences"),
    ],
    targets: [
        .target(
            name: "GemstoneServices",
            dependencies: [
                "Primitives",
                "Gemstone",
                "GemstonePrimitives",
                "Store",
                "Preferences",
                .product(name: "LocalStore", package: "SystemServices"),
                "Keychain",
                "NativeProviderService",
                .product(name: "WebSocketClient", package: "SwiftHTTPClient"),
                "Formatters",
            ],
            path: "Sources",
        ),
        .target(
            name: "GemstoneServicesTestKit",
            dependencies: [
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "NativeProviderService", package: "NativeProviderService"),
                "Gemstone",
                "GemstoneServices",
            ],
            path: "TestKit",
        ),
        .testTarget(
            name: "GemstoneServicesTests",
            dependencies: [
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                .product(name: "PreferencesTestKit", package: "Preferences"),
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "StoreTestKit", package: "Store"),
                "Primitives",
                .product(name: "Store", package: "Store"),
                .product(name: "WebSocketClientTestKit", package: "SwiftHTTPClient"),
                "Gemstone",
                "GemstoneServices",
                "GemstoneServicesTestKit",
            ],
            path: "Tests",
            resources: [.process("Resources")],
        ),
    ],
)
