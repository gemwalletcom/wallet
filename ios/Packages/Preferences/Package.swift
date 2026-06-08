// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Preferences",
    platforms: [
        .iOS(.v17),
        .macOS(.v15),
    ],
    products: [
        .library(
            name: "Preferences",
            targets: ["Preferences"],
        ),
        .library(
            name: "PreferencesTestKit",
            targets: ["PreferencesTestKit"],
        ),
    ],
    dependencies: [
        .package(name: "Primitives", path: "../Primitives"),
        .package(name: "Keychain", path: "../Keychain"),
        .package(name: "GemstonePrimitives", path: "../GemstonePrimitives"),
    ],
    targets: [
        .target(
            name: "Preferences",
            dependencies: [
                "Primitives",
                "Keychain",
                "GemstonePrimitives",
            ],
            path: "Sources",
        ),
        .target(
            name: "PreferencesTestKit",
            dependencies: [
                "Primitives",
                "Preferences",
            ],
            path: "TestKit",
        ),
        .testTarget(
            name: "PreferencesTest",
            dependencies: [
                "Preferences",
                "PreferencesTestKit",
                .product(name: "PrimitivesTestKit", package: "Primitives"),
            ],
            path: "Tests",
        ),
    ],
)
