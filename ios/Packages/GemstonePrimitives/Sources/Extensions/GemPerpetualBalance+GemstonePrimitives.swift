// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemPerpetualBalance {
    func map() -> PerpetualBalance {
        PerpetualBalance(
            available: available,
            reserved: reserved,
            withdrawable: withdrawable,
        )
    }
}
