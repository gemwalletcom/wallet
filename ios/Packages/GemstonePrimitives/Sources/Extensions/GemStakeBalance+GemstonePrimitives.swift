// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemStakeBalance {
    init(_ balance: Primitives.Balance) {
        self.init(
            frozen: balance.frozen.description,
            locked: balance.locked.description,
            staked: balance.staked.description,
            pending: balance.pending.description,
            rewards: balance.rewards.description,
        )
    }
}
