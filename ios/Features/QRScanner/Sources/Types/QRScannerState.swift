// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

enum QRScannerState: Equatable {
    case idle
    case scanning
    case failure(error: QRScannerError)
}
