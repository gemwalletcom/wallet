// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation

public extension Balance {
    static let zero = Balance(
        available: .zero,
        frozen: .zero,
        locked: .zero,
        staked: .zero,
        pending: .zero,
        pendingUnconfirmed: .zero,
        rewards: .zero,
        reserved: .zero,
        earn: .zero,
        withdrawable: .zero,
        metadata: nil,
    )
}
