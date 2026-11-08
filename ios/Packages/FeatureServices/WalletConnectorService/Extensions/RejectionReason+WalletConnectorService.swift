// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemWalletConnectError
import enum Gemstone.GemWalletConnectRejectionReason

extension GemWalletConnectRejectionReason {
    init(from error: any Error) {
        guard let walletConnectError = error as? GemWalletConnectError else {
            self = .userRejected
            return
        }
        self = walletConnectError.rejectionReason()
    }
}
