// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemRecipientServiceProtocol
import Primitives

public extension GemRecipientServiceProtocol {
    func recipientWallets() -> [Wallet] {
        do {
            return try recipientWallets().map { try Wallet($0) }
        } catch {
            preconditionFailure("Undecodable wallets: \(error)")
        }
    }
}
