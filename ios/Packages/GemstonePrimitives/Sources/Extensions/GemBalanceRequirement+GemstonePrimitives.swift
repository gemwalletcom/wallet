// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import struct Gemstone.GemBalanceRequirement
import Primitives

public extension GemBalanceRequirement {
    func map() -> BalanceRequirement {
        BalanceRequirement(required: BigInt(core: required), available: BigInt(core: available))
    }
}
