// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

private let hyperliquidName = "Hyperliquid"

public extension Recipient {
    static var hyperliquidProvider: Recipient {
        Recipient(name: hyperliquidName, address: "", memo: .none)
    }

    static var hyperliquidDeposit: Recipient {
        Recipient(name: hyperliquidName, address: PerpetualConfig.depositAddress, memo: .none)
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
