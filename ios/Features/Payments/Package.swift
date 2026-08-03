// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Payments",
    platforms: [.iOS(.v17),
                .macOS(.v15)],
    products: [
        .library(
            name: "Payments",
            targets: ["Payments"],
        ),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../../Packages/Primitives"),
        .package(name: "Store", path: "../../Packages/Store"),
        .package(name: "Blockchain", path: "../../Packages/Blockchain"),
        .package(name: "ChainServices", path: "../../Packages/ChainServices"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "FeatureServices", path: "../../Packages/FeatureServices"),
        .package(name: "GemstonePrimitives", path: "../../Packages/GemstonePrimitives"),
        .package(name: "Keystore", path: "../../Packages/Keystore"),
        .package(name: "Signer", path: "../../Packages/Signer"),
        .package(name: "EventPresenterService", path: "../EventPresenterService"),
        .package(name: "Formatters", path: "../../Packages/Formatters"),
    ],
    targets: [
        .target(
            name: "Payments",
            dependencies: [
                "Primitives",
                "Components",
                "Localization",
                "Style",
                "PrimitivesComponents",
                "GemstonePrimitives",
                "Formatters",
                "Keystore",
                "Signer",
                .product(name: "TransferService", package: "FeatureServices"),
                "EventPresenterService",
                "Blockchain",
                .product(name: "SigningRequestService", package: "ChainServices"),
                .product(name: "ChainService", package: "ChainServices"),
                .product(name: "ScanService", package: "ChainServices"),
                .product(name: "AssetsService", package: "FeatureServices"),
                .product(name: "BalanceService", package: "FeatureServices"),
                .product(name: "PaymentService", package: "ChainServices"),
                .product(name: "TransactionStateService", package: "FeatureServices"),
            ],
            path: "Sources/Payments",
        ),
        .testTarget(
            name: "PaymentsTests",
            dependencies: [
                "Payments",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
                .product(name: "SigningRequestServiceTestKit", package: "ChainServices"),
                .product(name: "PaymentServiceTestKit", package: "ChainServices"),
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "TransactionStateServiceTestKit", package: "FeatureServices"),
            ],
            path: "Tests/PaymentsTests",
        ),
    ],
)
