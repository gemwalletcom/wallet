// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

enum QRScannerError: Error {
    case notSupported
    case permissionsNotGranted
    case decoding
    case unknown(Error)
}

extension QRScannerError: Equatable {
    static func == (lhs: Self, rhs: Self) -> Bool {
        switch (lhs, rhs) {
        case (.notSupported, .notSupported),
             (.permissionsNotGranted, .permissionsNotGranted),
             (.decoding, .decoding):
            true
        case (.unknown, .unknown):
            true
        default:
            false
        }
    }
}
