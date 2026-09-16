// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemBalanceRequirement
import Primitives

public extension GemBalanceRequirement {
    func toPrimitives() -> BalanceRequirement {
        BalanceRequirement(required: required, available: available, shortfall: shortfall)
    }
}
