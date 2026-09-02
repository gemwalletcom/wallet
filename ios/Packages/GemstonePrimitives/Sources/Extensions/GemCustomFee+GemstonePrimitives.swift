// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import Primitives

public extension CustomFeeEstimate {
    init(
        rate: BigInt?,
        loadedFee: BigInt,
        baseTotal: BigInt,
        normalTotal: BigInt,
        maxMultiplier: Int,
    ) throws {
        let fee = GemCustomFee.estimate(
            rate: rate?.description,
            loadedFee: loadedFee.description,
            baseTotal: baseTotal.description,
            normalTotal: normalTotal.description,
            maxMultiplier: UInt32(maxMultiplier),
        )
        try self.init(
            feeAmount: BigInt.from(string: fee.feeValue()),
            maxRate: BigInt.from(string: fee.maxRate()),
            isOverMax: fee.isOverMax(),
        )
    }
}
