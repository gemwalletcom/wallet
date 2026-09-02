// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemRecipientServiceProtocol
import Primitives

public extension GemRecipientServiceProtocol {
    func otherWallets(wallet: Wallet) -> [Wallet] {
        do {
            return try otherWallets(walletId: wallet.id.id).map { try Wallet($0) }
        } catch {
            preconditionFailure("Undecodable wallets: \(error)")
        }
    }
}
