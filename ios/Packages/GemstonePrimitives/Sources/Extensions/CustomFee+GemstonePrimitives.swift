// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Gemstone

public struct CustomFeeEstimate: Sendable {
    public let feeAmount: BigInt
    public let maxRate: BigInt
    public let isOverMax: Bool
}

public extension CustomFeeEstimate {
    static func estimate(
        rate: BigInt?,
        loadedFee: BigInt,
        baseTotal: BigInt,
        normalTotal: BigInt,
        maxMultiplier: Int,
    ) -> CustomFeeEstimate {
        let result = try? Gemstone.customFeeEstimate(
            rate: rate?.description,
            loadedFee: loadedFee.description,
            baseTotal: baseTotal.description,
            normalTotal: normalTotal.description,
            maxMultiplier: UInt32(maxMultiplier),
        )
        return CustomFeeEstimate(
            feeAmount: result.flatMap { BigInt($0.feeAmount) } ?? loadedFee,
            maxRate: result.flatMap { BigInt($0.maxRate) } ?? .zero,
            isOverMax: result?.isOverMax ?? false,
        )
    }
}
