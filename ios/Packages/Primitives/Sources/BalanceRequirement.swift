// Copyright (c). Gem Wallet. All rights reserved.

import BigInt

public struct BalanceRequirement: Equatable, Sendable {
    public let required: BigInt
    public let available: BigInt

    public var shortfall: BigInt {
        max(required - available, .zero)
    }

    public init(required: BigInt, available: BigInt) {
        self.required = required
        self.available = available
    }
}
