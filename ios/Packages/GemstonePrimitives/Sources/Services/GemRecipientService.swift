// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemRecipientServiceProtocol
import Primitives

public extension GemRecipientServiceProtocol {
    func recipientWallets(wallets: [Wallet]) -> [Wallet] {
        do {
            return try recipientWallets(wallets: wallets.map { $0.json() }).map { try Wallet($0) }
        } catch {
            preconditionFailure("Undecodable wallets: \(error)")
        }
    }
}
