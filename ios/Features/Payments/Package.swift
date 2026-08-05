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
        .package(name: "ChainServices", path: "../../Packages/ChainServices"),
        .package(name: "Components", path: "../../Packages/Components"),
        .package(name: "Localization", path: "../../Packages/Localization"),
        .package(name: "Style", path: "../../Packages/Style"),
        .package(name: "PrimitivesComponents", path: "../../Packages/PrimitivesComponents"),
        .package(name: "FeatureServices", path: "../../Packages/FeatureServices"),
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
                "Formatters",
                "EventPresenterService",
                .product(name: "SigningRequestService", package: "ChainServices"),
                .product(name: "PaymentService", package: "FeatureServices"),
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
                .product(name: "PaymentServiceTestKit", package: "FeatureServices"),
                .product(name: "StoreTestKit", package: "Store"),
                .product(name: "TransactionStateServiceTestKit", package: "FeatureServices"),
            ],
            path: "Tests/PaymentsTests",
        ),
    ],
)
