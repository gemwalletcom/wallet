// Copyright (c). Gem Wallet. All rights reserved.

import BigInt

public struct BalanceRequirement: Equatable, Sendable {
    public let required: BigInt
    public let available: BigInt
    public let shortfall: BigInt

    public init(required: BigInt, available: BigInt, shortfall: BigInt) {
        self.required = required
        self.available = available
        self.shortfall = shortfall
    }
}
