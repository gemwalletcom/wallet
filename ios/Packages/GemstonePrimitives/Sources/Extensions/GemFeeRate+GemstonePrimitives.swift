// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemFeeRate {
    func map() throws -> FeeRate {
        try FeeRate(
            priority: priority.map(),
            gasPriceType: gasPriceType.map(),
        )
    }
}
