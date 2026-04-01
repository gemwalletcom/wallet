// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import Primitives

public extension GemBalance {
    func map() throws -> Balance {
        try Balance(
            available: BigInt.from(string: available),
            frozen: BigInt.from(string: frozen),
            locked: BigInt.from(string: locked),
            staked: BigInt.from(string: staked),
            pending: BigInt.from(string: pending),
            pendingUnconfirmed: BigInt.from(string: pendingUnconfirmed),
            rewards: BigInt.from(string: rewards),
            reserved: BigInt.from(string: reserved),
            withdrawable: BigInt.from(string: withdrawable),
            earn: BigInt.from(string: earn),
            metadata: metadata?.map(),
        )
    }
}
