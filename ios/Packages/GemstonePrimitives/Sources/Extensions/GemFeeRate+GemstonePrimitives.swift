// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemFeeRate {
    func map() throws -> FeeRate {
        try FeeRate(
            priority: FeePriority(id: priority),
            gasPriceType: gasPriceType.map(),
        )
    }
}
