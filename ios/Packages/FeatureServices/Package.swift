// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "FeatureServices",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(name: "StreamService", targets: ["StreamService"]),
        .library(name: "StreamServiceTestKit", targets: ["StreamServiceTestKit"]),
        .library(name: "AppService", targets: ["AppService"]),
        .library(name: "AppServiceTestKit", targets: ["AppServiceTestKit"]),
        .library(name: "ConnectionStatusService", targets: ["ConnectionStatusService"]),
        .library(name: "WalletConnectorService", targets: ["WalletConnectorService"]),
        .library(name: "WalletConnectorServiceTestKit", targets: ["WalletConnectorServiceTestKit"]),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../Primitives"),
        .package(name: "GemstoneServices", path: "../GemstoneServices"),
        .package(name: "Store", path: "../Store"),
        .package(name: "reown-swift", path: "../../Submodules/reown-swift"),
        .package(name: "GemstonePrimitives", path: "../GemstonePrimitives"),
        .package(name: "Gemstone", path: "../Gemstone"),
        .package(name: "SwiftHTTPClient", path: "../SwiftHTTPClient"),
        .package(name: "SystemServices", path: "../SystemServices"),
    ],
    targets: [
        .target(
            name: "StreamService",
            dependencies: [
                "Primitives",
                "Store",
                "Gemstone",
                "GemstonePrimitives",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                .product(name: "WebSocketClient", package: "SwiftHTTPClient"),
            ],
            path: "StreamService",
            exclude: ["TestKit"],
        ),
        .target(
            name: "StreamServiceTestKit",
            dependencies: [
                "StreamService",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "WebSocketClientTestKit", package: "SwiftHTTPClient"),
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
            ],
            path: "StreamService/TestKit",
        ),
        .target(
            name: "AppService",
            dependencies: [
                "Primitives",
                "Store",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                "GemstonePrimitives",
                "StreamService",
                "ConnectionStatusService",
                "WalletConnectorService",
                "Gemstone",
            ],
            path: "AppService",
            exclude: ["Tests", "TestKit"],
        ),
        .target(
            name: "AppServiceTestKit",
            dependencies: [
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
                "AppService",
                "Gemstone",
                "Primitives",
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                "StreamServiceTestKit",
                "WalletConnectorService",
                "WalletConnectorServiceTestKit",
                "ConnectionStatusService",
                .product(name: "ConnectivityService", package: "SystemServices"),
                .product(name: "ConnectivityServiceTestKit", package: "SystemServices"),
            ],
            path: "AppService/TestKit",
        ),
        .target(
            name: "ConnectionStatusService",
            dependencies: [
                "Primitives",
                "Gemstone",
                "GemstonePrimitives",
                .product(name: "ConnectivityService", package: "SystemServices"),
            ],
            path: "ConnectionStatusService",
            exclude: ["Tests"],
        ),
        .testTarget(
            name: "ConnectionStatusServiceTests",
            dependencies: [
                "ConnectionStatusService",
                "Gemstone",
                .product(name: "ConnectivityService", package: "SystemServices"),
            ],
            path: "ConnectionStatusService/Tests",
        ),
        .target(
            name: "WalletConnectorService",
            dependencies: [
                "Primitives",
                "Gemstone",
                "GemstonePrimitives",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                .product(name: "WalletConnect", package: "reown-swift"),
                .product(name: "ReownWalletKit", package: "reown-swift"),
                .product(name: "WalletConnectNetworking", package: "reown-swift"),
            ],
            path: "WalletConnectorService",
            exclude: ["TestKit", "Tests"],
        ),
        .target(
            name: "WalletConnectorServiceTestKit",
            dependencies: [
                "WalletConnectorService",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
            ],
            path: "WalletConnectorService/TestKit",
        ),
        .testTarget(
            name: "WalletConnectorServiceTests",
            dependencies: [
                "WalletConnectorService",
            ],
            path: "WalletConnectorService/Tests",
        ),
        .testTarget(
            name: "AppServiceTests",
            dependencies: [
                "AppService",
                "AppServiceTestKit",
                "WalletConnectorService",
                "WalletConnectorServiceTestKit",
                .product(name: "GemstoneServices", package: "GemstoneServices"),
                .product(name: "GemstoneServicesTestKit", package: "GemstoneServices"),
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                "Store",
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "GemstonePrimitivesTestKit", package: "GemstonePrimitives"),
            ],
            path: "AppService/Tests",
        ),
    ],
)
