// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

enum QRScannerError: Error, Equatable {
    case notSupported
    case permissionsNotGranted
}
