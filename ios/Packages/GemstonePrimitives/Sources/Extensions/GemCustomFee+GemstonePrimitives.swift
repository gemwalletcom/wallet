// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import Primitives

public extension GemCustomFee {
    func map() throws -> CustomFeeEstimate {
        try CustomFeeEstimate(
            feeAmount: BigInt.from(string: feeValue),
            maxRate: BigInt.from(string: maxRate),
            isOverMax: isOverMax,
        )
    }
}

public extension CustomFeeEstimate {
    static func estimate(
        rate: BigInt?,
        loadedFee: BigInt,
        baseTotal: BigInt,
        normalTotal: BigInt,
        maxMultiplier: Int,
        feeService: GemFeeService,
    ) throws -> CustomFeeEstimate {
        try feeService.customFeeEstimate(
            rate: rate?.description,
            loadedFee: loadedFee.description,
            baseTotal: baseTotal.description,
            normalTotal: normalTotal.description,
            maxMultiplier: UInt32(maxMultiplier),
        ).map()
    }
}
