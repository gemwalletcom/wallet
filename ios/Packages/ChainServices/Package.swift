// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "ChainServices",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(name: "WalletConnectorService", targets: ["WalletConnectorService"]),
        .library(name: "WalletConnectorServiceTestKit", targets: ["WalletConnectorServiceTestKit"]),
        .library(name: "ChainService", targets: ["ChainService"]),
        .library(name: "ChainServiceTestKit", targets: ["ChainServiceTestKit"]),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../Primitives"),
        .package(name: "GemAPI", path: "../GemAPI"),
        .package(name: "Store", path: "../Store"),
        .package(name: "Blockchain", path: "../Blockchain"),
        .package(name: "Gemstone", path: "../Gemstone"),
        .package(name: "GemstonePrimitives", path: "../GemstonePrimitives"),
        .package(name: "NativeProviderService", path: "../NativeProviderService"),
        .package(name: "reown-swift", path: "../../Submodules/reown-swift"),
    ],
    targets: [
        .target(
            name: "WalletConnectorService",
            dependencies: [
                "Primitives",
                "Gemstone",
                "GemstonePrimitives",
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
        .target(
            name: "ChainService",
            dependencies: [
                "Primitives",
                "Blockchain",
                "NativeProviderService",
            ],
            path: "ChainService",
            exclude: ["TestKit"],
        ),
        .target(
            name: "ChainServiceTestKit",
            dependencies: [
                "ChainService",
                "Primitives",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                "Blockchain",
                .product(name: "BlockchainTestKit", package: "Blockchain"),
            ],
            path: "ChainService/TestKit",
        ),
        .testTarget(
            name: "WalletConnectorServiceTests",
            dependencies: [
                "WalletConnectorService",
            ],
            path: "WalletConnectorService/Tests",
        ),
    ],
)
