// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Gemstone.TpslOrderData {
    func map() throws -> Primitives.TPSLOrderData {
        Primitives.TPSLOrderData(
            direction: direction.map(),
            takeProfit: takeProfit,
            stopLoss: stopLoss,
            size: size,
        )
    }
}

public extension Primitives.TPSLOrderData {
    func map() -> Gemstone.TpslOrderData {
        Gemstone.TpslOrderData(
            direction: direction.map(),
            takeProfit: takeProfit,
            stopLoss: stopLoss,
            size: size,
        )
    }
}
