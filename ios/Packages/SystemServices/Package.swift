// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "SystemServices",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(name: "ImageGalleryService", targets: ["ImageGalleryService"]),
        .library(name: "ConnectivityService", targets: ["ConnectivityService"]),
        .library(name: "ConnectivityServiceTestKit", targets: ["ConnectivityServiceTestKit"]),
    ],
    dependencies: [],
    targets: [
        .target(
            name: "ImageGalleryService",
            dependencies: [],
            path: "ImageGalleryService",
            exclude: ["Tests", "TestKit"],
        ),
        .target(
            name: "ConnectivityService",
            dependencies: [],
            path: "ConnectivityService",
            exclude: ["Tests", "TestKit"],
        ),
        .target(
            name: "ConnectivityServiceTestKit",
            dependencies: [
                "ConnectivityService",
            ],
            path: "ConnectivityService/TestKit",
        ),
        .testTarget(
            name: "ConnectivityServiceTests",
            dependencies: [
                "ConnectivityService",
                "ConnectivityServiceTestKit",
            ],
            path: "ConnectivityService/Tests",
        ),
    ],
)
