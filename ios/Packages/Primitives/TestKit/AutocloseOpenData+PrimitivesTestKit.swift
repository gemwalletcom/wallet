// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension AutocloseOpenData {
    static func mock(
        symbol: String = "BTC",
        direction: PerpetualDirection = .long,
        leverage: UInt8 = 10,
        size: Double = 1.0,
    ) -> AutocloseOpenData {
        AutocloseOpenData(
            assetId: .mock(.bitcoin),
            symbol: symbol,
            direction: direction,
            marketPrice: 100.0,
            leverage: leverage,
            size: size,
            assetDecimals: 8,
            takeProfit: nil,
            stopLoss: nil,
        )
    }
}
