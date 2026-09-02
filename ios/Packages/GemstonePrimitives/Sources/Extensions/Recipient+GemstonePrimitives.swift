// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemRecipient
import Primitives

private let hyperliquidName = "Hyperliquid"

extension GemRecipient: @retroactive Identifiable {
    public var id: String {
        [name ?? "", address, memo ?? ""].joined(separator: "_")
    }
}

public extension GemRecipient {
    static var hyperliquidProvider: GemRecipient {
        GemRecipient(address: "", name: hyperliquidName)
    }

    static var hyperliquidDeposit: GemRecipient {
        GemRecipient(address: PerpetualConfig.depositAddress, name: hyperliquidName)
    }
}

public extension RecipientData {
    static func hyperliquid() -> RecipientData {
        RecipientData(recipient: .hyperliquidProvider, amount: .none)
    }

    static var hyperliquidDeposit: RecipientData {
        RecipientData(recipient: .hyperliquidDeposit, amount: .none)
    }
}
