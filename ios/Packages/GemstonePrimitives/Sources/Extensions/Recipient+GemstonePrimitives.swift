// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension Recipient {
    static var hyperliquidDeposit: Recipient {
        Recipient(name: "Hyperliquid", address: PerpetualConfig.depositAddress, memo: .none)
    }
}
