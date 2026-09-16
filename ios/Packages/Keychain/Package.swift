// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "Keychain",
    platforms: [.iOS(.v17), .macOS(.v12)],
    products: [
        .library(
            name: "Keychain",
            targets: ["Keychain"],
        ),
    ],
    targets: [
        .target(
            name: "Keychain",
            path: "Sources",
        ),
    ],
)
