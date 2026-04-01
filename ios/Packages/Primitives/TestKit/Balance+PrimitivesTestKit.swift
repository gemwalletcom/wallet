// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives

public extension Balance {
    static func mock(
        available: BigInt = BigInt("1000000000000000000"),
        frozen: BigInt = .zero,
        locked: BigInt = .zero,
        staked: BigInt = .zero,
        pending: BigInt = .zero,
        rewards: BigInt = .zero,
        reserved: BigInt = .zero,
        withdrawable: BigInt = .zero,
        earn: BigInt = .zero,
    ) -> Balance {
        Balance(
            available: available,
            frozen: frozen,
            locked: locked,
            staked: staked,
            pending: pending,
            rewards: rewards,
            reserved: reserved,
            withdrawable: withdrawable,
            earn: earn,
        )
    }
}
