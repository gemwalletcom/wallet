// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

enum ConfirmationPhase {
    case idle
    case confirming
    case failed(Error)

    var isConfirming: Bool {
        switch self {
        case .confirming: true
        case .idle, .failed: false
        }
    }
}
