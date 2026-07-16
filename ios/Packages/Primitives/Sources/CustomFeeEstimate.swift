// Copyright (c). Gem Wallet. All rights reserved.

import BigInt

public struct CustomFeeEstimate: Equatable, Sendable {
    public let feeAmount: BigInt
    public let maxRate: BigInt
    public let isOverMax: Bool

    public init(
        feeAmount: BigInt,
        maxRate: BigInt,
        isOverMax: Bool,
    ) {
        self.feeAmount = feeAmount
        self.maxRate = maxRate
        self.isOverMax = isOverMax
    }
}
