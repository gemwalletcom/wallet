// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemBalanceMetadata {
    func map() -> BalanceMetadata {
        BalanceMetadata(
            votes: votes,
            energyAvailable: energyAvailable,
            energyTotal: energyTotal,
            bandwidthAvailable: bandwidthAvailable,
            bandwidthTotal: bandwidthTotal,
        )
    }
}
