// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemRecipientServiceProtocol
import Primitives

public extension GemRecipientServiceProtocol {
    func recipientWallets(wallets: [Wallet]) -> [Wallet] {
        recipientWallets(wallets: wallets.map { $0.map() }).map { $0.map() }
    }
}
