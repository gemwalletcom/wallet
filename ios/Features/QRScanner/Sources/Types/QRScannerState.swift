// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

enum QRScannerState: Equatable {
    case scanning
    case failure(error: QRScannerError)
}
