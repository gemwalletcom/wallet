// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public struct KeystoreMigrationFailure: Sendable {
    public let walletId: WalletId
    public let error: any Error

    public init(walletId: WalletId, error: any Error) {
        self.walletId = walletId
        self.error = error
    }
}
