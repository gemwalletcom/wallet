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
    ],
    dependencies: [
        .package(name: "Primitives", path: "../Primitives"),
        .package(name: "Gemstone", path: "../Gemstone"),
        .package(name: "GemstonePrimitives", path: "../GemstonePrimitives"),
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
        .testTarget(
            name: "WalletConnectorServiceTests",
            dependencies: [
                "WalletConnectorService",
            ],
            path: "WalletConnectorService/Tests",
        ),
    ],
)
