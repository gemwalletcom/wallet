// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemBalanceRequirement
import Primitives

public extension GemBalanceRequirement {
    func map() -> BalanceRequirement {
        BalanceRequirement(required: required, available: available, shortfall: shortfall)
    }
}
